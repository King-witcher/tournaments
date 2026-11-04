//! O que a chave emite no outbox. Depois de toda mudança (`settle`), compara
//! cada partida com como estava antes (`Snapshot`) e, para as que mudaram,
//! avisa a página da partida e a chave, "Seus torneios" de quem joga nelas e,
//! na que passou a ter os dois lados sem estar decidida, os dois (`match.ready`;
//! a que vira W.O. na hora já chega decidida; no suíço, as partidas de cada
//! rodada nova). No round robin, em que todas as partidas ficam prontas no
//! início, `match.ready` vai para cada time só com a série sugerida a ele: a
//! primeira no início e a seguinte quando ele termina uma série, junto com o
//! adversário livre que a esperava (`progress::notices`); como a sugestão e
//! quem está jogando dependem das séries dos outros, quando mudam avisa "Seus
//! torneios" de todos os times e, para as páginas das séries, o torneio
//! inteiro. Se o torneio encerra ou reabre,
//! avisa o torneio inteiro; se os mapas contestados mudam, a home da
//! organização (`tenants::organizing_changed`). Os avisos de mapa enviado
//! (para o adversário) e de contestação (para a organização) saem de quem
//! envia e de quem contesta; os do veto (adversário pronto, veto começou), de
//! quem fica pronto ou força o começo. A abertura dos picks por jogador
//! (`match.team_pick_open`, para os jogadores dos dois times, sem dizer nada do
//! que foi escolhido) sai daqui, porque ela abre por qualquer caminho (o último
//! ban, o tempo esgotado, o jogo seguinte, o desfazer). O aviso que pede ação vai
//! só para quem pode agir pela regra do torneio (`acting`); `match.ready` só diz
//! que a série está pronta, e vai para o time.

use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, Utc};
use engine::{
    core::{MapId, Match, MatchId, MatchState, ParticipantId},
    formats::{self as ef, StageBracket},
    veto::{Action, Turn},
};
use sqlx::PgConnection;

use super::{
    acting, announce,
    games::GameStatus,
    key,
    progress::{self, Notices, Suggested},
    result::Confirmation,
    stage::Stage,
};
use crate::{
    outbox::{self, Event, Notice, NoticeKind},
    shared::ApiError,
    teams::StoredTeam,
    tenants,
    tournaments::{self, Address, StoredTournament, TeamAction, TournamentStatus},
};

/// Como cada partida da fase estava antes de uma mudança e, no round robin, a
/// série sugerida a cada time e quem estava jogando.
pub struct Snapshot {
    marks: HashMap<MatchId, Mark>,
    suggested: Suggested,
    playing: BTreeSet<ParticipantId>,
}

impl Snapshot {
    pub fn of(stage: &Stage) -> Self {
        Self {
            marks: stage
                .bracket
                .matches()
                .iter()
                .map(|m| (m.id, Mark::of(stage, m)))
                .collect(),
            suggested: progress::suggestions(stage),
            playing: progress::playing(stage),
        }
    }

    /// Os mapas contestados (pelo id).
    fn contested(&self) -> BTreeSet<i64> {
        let games = self.marks.values().flat_map(|m| &m.games);
        games
            .filter(|g| g.2 == GameStatus::Contested)
            .map(|g| g.0)
            .collect()
    }
}

/// Os mapas contestados da fase (pelo id): o que ela deixa esperando a
/// organização.
fn contested(stage: &Stage) -> BTreeSet<i64> {
    stage
        .games
        .iter()
        .filter(|g| g.status == GameStatus::Contested)
        .map(|g| g.id)
        .collect()
}

/// O que se vê de uma partida: estado na chave, mapas, série decidida e o
/// veto (quantos eventos e o último, e o jogo com os picks por jogador abertos).
#[derive(PartialEq)]
struct Mark {
    state: MatchState,
    games: Vec<GameMark>,
    result: Option<ResultMark>,
    veto: (usize, Option<i64>),
    team_pick: Option<u8>,
}

type GameMark = (
    i64,
    i16,
    GameStatus,
    ParticipantId,
    Option<MapId>,
    Option<(i16, i16)>,
);
type ResultMark = (i64, ParticipantId, Option<(u8, u8)>, Confirmation);

impl Mark {
    fn of(stage: &Stage, m: &Match) -> Self {
        let games = stage
            .games_of(m.id)
            .iter()
            .map(|g| {
                let map = g.map.as_ref().map(|map| map.id);
                (g.id, g.number, g.status, g.winner, map, g.frags)
            })
            .collect();
        let result = stage
            .result(m.id)
            .map(|r| (r.id, r.winner, r.score, r.confirmed_via));
        let veto = stage
            .veto_of(m.id)
            .map_or((0, None), |v| (v.rows.len(), v.rows.last().map(|r| r.id)));
        let team_pick = stage
            .veto_of(m.id)
            .filter(|_| matches!(m.state, MatchState::Ready { .. }))
            .and_then(|v| match v.state.turn() {
                Turn::Act {
                    game,
                    action: Action::TeamPick,
                    ..
                } => Some(game),
                _ => None,
            });
        Self {
            state: m.state,
            games,
            result,
            veto,
            team_pick,
        }
    }

    /// Ninguém enviou mapa nem mexeu no veto.
    const fn untouched(&self) -> bool {
        self.games.is_empty() && self.veto.0 == 0
    }
}

const fn sides(state: MatchState) -> Option<(ParticipantId, ParticipantId)> {
    match state {
        MatchState::Ready { a, b } | MatchState::Completed { a, b, .. } => Some((a, b)),
        _ => None,
    }
}

const fn ready(state: MatchState) -> Option<(ParticipantId, ParticipantId)> {
    match state {
        MatchState::Ready { a, b } => Some((a, b)),
        _ => None,
    }
}

/// Depois de `settle`: emite o que mudou de `before` para `after`. `status`
/// é o do torneio depois da mudança.
pub async fn settled(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    before: &Snapshot,
    after: &Stage,
    status: TournamentStatus,
) -> Result<(), ApiError> {
    let audience = tournament.address();
    let mut players = BTreeSet::new();
    // As mudanças de cada partida e a de "Seus torneios" vão juntas, numa ida
    // ao banco (os avisos saem no meio, cada um na sua).
    let mut changed = Vec::new();
    for m in after.bracket.matches() {
        let now = Mark::of(after, m);
        let was = before.marks.get(&m.id);
        if was == Some(&now) {
            continue;
        }
        let key = key::format(m.id);
        changed.push(Event::Tournament {
            tenant: audience.tenant.clone(),
            tournament: audience.slug.clone(),
            match_key: Some(key.clone()),
        });
        let was_sides = was.and_then(|w| sides(w.state));
        for (a, b) in [sides(m.state), was_sides].into_iter().flatten() {
            players.extend(
                [a, b]
                    .iter()
                    .filter_map(|p| after.entrant(*p))
                    .flat_map(StoredTeam::user_ids),
            );
        }
        if let (Some(game), Some((a, b))) = (now.team_pick, ready(m.state))
            && was.and_then(|w| w.team_pick) != Some(game)
        {
            team_pick_open(conn, &audience, after, &key, (a, b), game).await?;
        }
        let decided = |mark: &Mark| mark.result.as_ref().map(|r| (r.1, r.2));
        if decided(&now).is_some() && was.map(decided) != Some(decided(&now)) {
            announce::series_decided(conn, &audience, after, m).await?;
        }
    }
    let suggested = progress::suggestions(after);
    let playing = progress::playing(after);
    let notices = if is_league(&after.bracket) {
        progress::notices(
            after,
            &before.suggested,
            |id| before.marks.get(&id).map(|m| (m.state, m.untouched())),
            &suggested,
            &playing,
        )
    } else {
        ready_notices(Some(before), &after.bracket)
    };
    for (id, pair, to) in notices {
        match_ready(
            conn,
            &audience,
            &after.entrants,
            &key::format(id),
            pair,
            &to,
        )
        .await?;
    }
    if before.suggested != suggested || before.playing != playing {
        players.extend(after.entrants.iter().flat_map(StoredTeam::user_ids));
    }
    if before.playing != playing {
        changed.push(Event::Tournament {
            tenant: audience.tenant.clone(),
            tournament: audience.slug.clone(),
            match_key: None,
        });
    }
    changed.push(Event::me(players));
    outbox::emit_all(conn, changed).await?;
    if status != tournament.status {
        tournaments::changed(conn, tournament.id).await?;
        if status == TournamentStatus::Finished {
            announce::finished(conn, &audience, after).await?;
        }
    }
    // Contestou, resolveu, corrigiu, anulou, deu W.O.: a lista de mapas
    // contestados da home da organização mudou.
    if before.contested() != contested(after) {
        tenants::organizing_changed(conn, tournament.tenant).await?;
    }
    Ok(())
}

/// Ao iniciar: os lados das partidas que já nascem prontas (no round robin,
/// a primeira de cada time).
pub async fn started(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    bracket: &StageBracket,
    entrants: &[StoredTeam],
) -> Result<(), ApiError> {
    let audience = tournament.address();
    let notices = if is_league(bracket) {
        progress::first_notices(bracket)
    } else {
        ready_notices(None, bracket)
    };
    for (id, pair, to) in notices {
        match_ready(conn, &audience, entrants, &key::format(id), pair, &to).await?;
    }
    announce::started(conn, &audience).await?;
    Ok(())
}

fn is_league(bracket: &StageBracket) -> bool {
    matches!(bracket.format(), ef::StageFormat::RoundRobin(_))
}

/// Na eliminação e no suíço, quem recebe `match.ready` de qual partida (com os
/// lados dela) depois de uma mudança de `before` (`None`: o início) para
/// `after`: os dois lados da partida que passou a estar pronta com eles. O
/// round robin, com todas prontas desde o início, é `progress::notices`.
fn ready_notices(before: Option<&Snapshot>, after: &StageBracket) -> Notices {
    let was = |id: MatchId| before.and_then(|b| b.marks.get(&id)).map(|m| m.state);
    after
        .matches()
        .iter()
        .filter_map(|m| {
            let pair = ready(m.state)?;
            let fresh = was(m.id).and_then(ready) != Some(pair);
            fresh.then(|| (m.id, pair, vec![pair.0, pair.1]))
        })
        .collect()
}

/// `match.ready` da partida `key`, de lados `(a, b)`, para os jogadores dos
/// times `to` (um ou os dois lados), com o nome do outro lado.
async fn match_ready(
    conn: &mut PgConnection,
    audience: &Address,
    entrants: &[StoredTeam],
    key: &str,
    (a, b): (ParticipantId, ParticipantId),
    to: &[ParticipantId],
) -> Result<(), sqlx::Error> {
    let entrant = |p: ParticipantId| entrants.iter().find(|e| e.id == p.0);
    let (Some(team_a), Some(team_b)) = (entrant(a), entrant(b)) else {
        return Ok(());
    };
    let sides = [(a, team_a, team_b), (b, team_b, team_a)];
    for (_, to, opponent) in sides.into_iter().filter(|(p, ..)| to.contains(p)) {
        let notice = Notice {
            kind: NoticeKind::MatchReady {
                tournament: audience.name.clone(),
                match_key: key.to_owned(),
                opponent: opponent.display_name(),
            },
            path: match_path(audience, key),
        };
        outbox::emit(conn, Event::notice(to.user_ids(), notice)).await?;
    }
    Ok(())
}

/// Os picks por jogador da partida `game` abriram: aviso para quem escolhe nos
/// dois times e está na comunidade (todos, ou só o capitão, pela regra).
async fn team_pick_open(
    conn: &mut PgConnection,
    audience: &Address,
    stage: &Stage,
    key: &str,
    (a, b): (ParticipantId, ParticipantId),
    game: u8,
) -> Result<(), sqlx::Error> {
    let rule_open = stage.team_rules.open_to_all(TeamAction::Pick);
    let players = [a, b]
        .into_iter()
        .filter_map(|p| stage.entrant(p))
        .flat_map(|t| {
            t.members
                .iter()
                .filter(move |m| !m.left && (rule_open || m.user_id == t.captain))
                .map(|m| m.user_id)
        });
    let notice = Notice {
        kind: NoticeKind::TeamPickOpen {
            tournament: audience.name.clone(),
            match_key: key.to_owned(),
            game: i16::from(game),
        },
        path: match_path(audience, key),
    };
    outbox::emit(conn, Event::notice(players, notice)).await
}

fn match_path(audience: &Address, key: &str) -> String {
    format!("{}/matches/{key}", audience.path())
}

/// O time `from` enviou o mapa `game` da partida `key`: aviso para quem responde
/// por `to`, o adversário, com o prazo para confirmar ou contestar.
pub async fn game_reported(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    key: &str,
    game: i16,
    (from, to): (&StoredTeam, &StoredTeam),
    confirm_by: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let audience = tournament.address();
    let notice = Notice {
        kind: NoticeKind::GameReported {
            tournament: audience.name.clone(),
            match_key: key.to_owned(),
            game,
            opponent: from.display_name(),
            confirm_by,
        },
        path: match_path(&audience, key),
    };
    let actors = acting::audience(to, tournament.setup.team_rules, TeamAction::Report);
    outbox::emit(conn, Event::notice(actors, notice)).await
}

/// `player` (apelido; `by`, o usuário) contestou o mapa `game` da partida
/// `key`: aviso para organizers e owner, menos quem contestou.
pub async fn game_contested(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    key: &str,
    game: i16,
    (by, player): (i64, &str),
) -> Result<(), sqlx::Error> {
    let audience = tournament.address();
    let staff = tenants::staff(conn, tournament.tenant).await?;
    let notice = Notice {
        kind: NoticeKind::GameContested {
            tournament: audience.name.clone(),
            match_key: key.to_owned(),
            game,
            player: player.to_owned(),
        },
        path: match_path(&audience, key),
    };
    let to = staff.into_iter().filter(|u| *u != by);
    outbox::emit(conn, Event::notice(to, notice)).await
}

/// O veto da partida `key` começou: aviso para quem age nele pelos dois lados,
/// com o nome do outro.
pub async fn veto_started(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    key: &str,
    (a, b): (ParticipantId, ParticipantId),
) -> Result<(), ApiError> {
    let audience = tournament.address();
    let (Some(a), Some(b)) = (stage.entrant(a), stage.entrant(b)) else {
        return Ok(());
    };
    for (to, opponent) in [(a, b), (b, a)] {
        let notice = Notice {
            kind: NoticeKind::VetoStarted {
                tournament: audience.name.clone(),
                match_key: key.to_owned(),
                opponent: opponent.display_name(),
            },
            path: match_path(&audience, key),
        };
        outbox::emit(
            conn,
            Event::notice(acting::veto_audience(stage, to), notice),
        )
        .await?;
    }
    Ok(())
}

/// `opponent` (o nome do time) está pronto para o veto da partida `key`:
/// aviso para `to`, quem ainda falta marcar "Pronto" do outro lado.
pub async fn opponent_ready(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    key: &str,
    to: &[i64],
    opponent: &str,
) -> Result<(), ApiError> {
    let audience = tournament.address();
    let notice = Notice {
        kind: NoticeKind::OpponentReady {
            tournament: audience.name.clone(),
            match_key: key.to_owned(),
            opponent: opponent.to_owned(),
        },
        path: match_path(&audience, key),
    };
    outbox::emit(conn, Event::notice(to.iter().copied(), notice)).await?;
    Ok(())
}
