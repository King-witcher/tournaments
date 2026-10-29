//! O que a chave emite no outbox. Depois de toda mudança (`settle`), compara
//! cada partida com como estava antes (`Snapshot`) e, para as que mudaram,
//! avisa a página da partida e a chave, "Seus torneios" de quem joga nelas e,
//! na que passou a ter os dois lados sem estar decidida, os dois (`match.ready`;
//! a que vira W.O. na hora já chega decidida; no suíço, as partidas de cada
//! rodada nova). No round robin, em que todas as partidas ficam prontas no
//! início, `match.ready` vai para cada time só quando a próxima partida dele
//! (a primeira pronta, pela ordem das rodadas) muda: a primeira no início e a
//! seguinte quando ele termina uma série (`ready_notices`). Se o torneio
//! encerra ou reabre,
//! avisa o torneio inteiro; se os mapas contestados mudam, a home da
//! organização (`tenants::organizing_changed`). Os avisos de mapa enviado
//! (para o adversário) e de contestação (para a organização) saem de quem
//! envia e de quem contesta; os do veto (adversário pronto, veto começou), de
//! quem fica pronto ou força o começo.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{DateTime, Utc};
use engine::{
    core::{MapId, Match, MatchId, MatchState, ParticipantId},
    formats::{self as ef, StageBracket},
};
use sqlx::PgConnection;

use super::{announce, games::GameStatus, key, result::Confirmation, stage::Stage};
use crate::{
    outbox::{self, Event, Notice, NoticeKind},
    shared::ApiError,
    teams::StoredTeam,
    tenants,
    tournaments::{self, Audience, StoredTournament, TournamentStatus},
};

/// Como cada partida da fase estava antes de uma mudança.
pub struct Snapshot(HashMap<MatchId, Mark>);

impl Snapshot {
    pub fn of(stage: &Stage) -> Self {
        Self(
            stage
                .bracket
                .matches()
                .iter()
                .map(|m| (m.id, Mark::of(stage, m)))
                .collect(),
        )
    }

    /// Os mapas contestados (pelo id).
    fn contested(&self) -> BTreeSet<i64> {
        let games = self.0.values().flat_map(|m| &m.games);
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
/// veto (quantos eventos e o último).
#[derive(PartialEq)]
struct Mark {
    state: MatchState,
    games: Vec<GameMark>,
    result: Option<ResultMark>,
    veto: (usize, Option<i64>),
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
        Self {
            state: m.state,
            games,
            result,
            veto,
        }
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
    let audience = tournaments::audience(conn, tournament.id).await?;
    let mut players = BTreeSet::new();
    for m in after.bracket.matches() {
        let now = Mark::of(after, m);
        let was = before.0.get(&m.id);
        if was == Some(&now) {
            continue;
        }
        let key = key::format(m.id);
        let changed = Event::Tournament {
            tenant: audience.tenant.clone(),
            tournament: audience.slug.clone(),
            match_key: Some(key.clone()),
        };
        outbox::emit(conn, changed).await?;
        let was_sides = was.and_then(|w| sides(w.state));
        for (a, b) in [sides(m.state), was_sides].into_iter().flatten() {
            players.extend(
                [a, b]
                    .iter()
                    .filter_map(|p| after.entrant(*p))
                    .flat_map(StoredTeam::user_ids),
            );
        }
        let decided = |mark: &Mark| mark.result.as_ref().map(|r| (r.1, r.2));
        if decided(&now).is_some() && was.map(decided) != Some(decided(&now)) {
            announce::series_decided(conn, &audience, after, m).await?;
        }
    }
    for (id, pair, to) in ready_notices(Some(before), &after.bracket) {
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
    outbox::emit(conn, Event::me(players)).await?;
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
    let audience = tournaments::audience(conn, tournament.id).await?;
    for (id, pair, to) in ready_notices(None, bracket) {
        match_ready(conn, &audience, entrants, &key::format(id), pair, &to).await?;
    }
    announce::started(conn, &audience).await?;
    Ok(())
}

/// Quem recebe `match.ready` de qual partida (com os lados dela) depois de
/// uma mudança de `before` (`None`: o início) para `after`. Na eliminação e
/// no suíço, os dois lados da partida que passou a estar pronta com eles. No
/// round robin, todas ficam prontas no início: cada time recebe só a próxima
/// dele, quando ela muda (a primeira no início; a seguinte quando ele termina
/// uma série; a reaberta por uma correção, se vem antes).
fn ready_notices(
    before: Option<&Snapshot>,
    after: &StageBracket,
) -> Vec<(MatchId, (ParticipantId, ParticipantId), Vec<ParticipantId>)> {
    let was = |id: MatchId| before.and_then(|b| b.0.get(&id)).map(|m| m.state);
    if !matches!(after.format(), ef::StageFormat::RoundRobin(_)) {
        return after
            .matches()
            .iter()
            .filter_map(|m| {
                let pair = ready(m.state)?;
                let fresh = was(m.id).and_then(ready) != Some(pair);
                fresh.then(|| (m.id, pair, vec![pair.0, pair.1]))
            })
            .collect();
    }
    let old = next_ready(
        before
            .into_iter()
            .flat_map(|b| b.0.iter().map(|(id, m)| (*id, m.state))),
    );
    let new = next_ready(after.matches().iter().map(|m| (m.id, m.state)));
    let mut notices: Vec<(MatchId, (ParticipantId, ParticipantId), Vec<ParticipantId>)> =
        Vec::new();
    for (team, (id, pair)) in new {
        if old.get(&team).map(|(id, _)| *id) == Some(id) {
            continue;
        }
        match notices.iter_mut().find(|n| n.0 == id) {
            Some(n) => n.2.push(team),
            None => notices.push((id, pair, vec![team])),
        }
    }
    notices
}

/// A próxima partida de cada time: a primeira pronta, pela ordem das rodadas.
fn next_ready(
    states: impl Iterator<Item = (MatchId, MatchState)>,
) -> BTreeMap<ParticipantId, (MatchId, (ParticipantId, ParticipantId))> {
    let mut ready: Vec<(MatchId, (ParticipantId, ParticipantId))> = states
        .filter_map(|(id, state)| ready(state).map(|pair| (id, pair)))
        .collect();
    ready.sort_by_key(|(id, _)| *id);
    let mut next = BTreeMap::new();
    for (id, (a, b)) in ready {
        next.entry(a).or_insert((id, (a, b)));
        next.entry(b).or_insert((id, (a, b)));
    }
    next
}

/// `match.ready` da partida `key`, de lados `(a, b)`, para os jogadores dos
/// times `to` (um ou os dois lados), com o nome do outro lado.
async fn match_ready(
    conn: &mut PgConnection,
    audience: &Audience,
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

fn match_path(audience: &Audience, key: &str) -> String {
    format!("{}/matches/{key}", audience.path())
}

/// O time `from` enviou o mapa `game` da partida `key`: aviso para os
/// jogadores de `to`, o adversário, com o prazo para confirmar ou contestar.
pub async fn game_reported(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    key: &str,
    game: i16,
    (from, to): (&StoredTeam, &StoredTeam),
    confirm_by: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let audience = tournaments::audience(conn, tournament.id).await?;
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
    outbox::emit(conn, Event::notice(to.user_ids(), notice)).await
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
    let audience = tournaments::audience(conn, tournament.id).await?;
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

/// O veto da partida `key` começou: aviso para os jogadores dos dois lados,
/// com o nome do outro.
pub async fn veto_started(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    key: &str,
    (a, b): (ParticipantId, ParticipantId),
) -> Result<(), ApiError> {
    let audience = tournaments::audience(conn, tournament.id).await?;
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
        outbox::emit(conn, Event::notice(to.user_ids(), notice)).await?;
    }
    Ok(())
}

/// `opponent` (o nome do time) está pronto para o veto da partida `key`:
/// aviso para os jogadores de `to`, o outro lado.
pub async fn opponent_ready(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    key: &str,
    to: ParticipantId,
    opponent: &str,
) -> Result<(), ApiError> {
    let Some(to) = stage.entrant(to) else {
        return Ok(());
    };
    let audience = tournaments::audience(conn, tournament.id).await?;
    let notice = Notice {
        kind: NoticeKind::OpponentReady {
            tournament: audience.name.clone(),
            match_key: key.to_owned(),
            opponent: opponent.to_owned(),
        },
        path: match_path(&audience, key),
    };
    outbox::emit(conn, Event::notice(to.user_ids(), notice)).await?;
    Ok(())
}
