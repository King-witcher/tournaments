//! O que a chave emite no outbox. Depois de toda mudança (`settle`), compara
//! cada partida com como estava antes (`Snapshot`) e, para as que mudaram,
//! avisa a página da partida e a chave, "Seus torneios" de quem joga nelas e,
//! na que passou a ter os dois lados sem estar decidida, os dois (`match.ready`;
//! a que vira W.O. na hora já chega decidida). Se o torneio encerra ou reabre,
//! avisa o torneio inteiro; se os mapas contestados mudam, a home da
//! organização (`tenants::organizing_changed`). Os avisos de mapa enviado
//! (para o adversário) e de contestação (para a organização) saem de quem
//! envia e de quem contesta.

use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, Utc};
use engine::{
    core::{MapId, Match, MatchId, MatchState, ParticipantId},
    formats::elimination::EliminationBracket,
};
use sqlx::PgConnection;

use super::{games::GameStatus, key, result::Confirmation, stage::Stage};
use crate::{
    outbox::{self, Event, Notice, NoticeKind},
    participants::Seeded,
    shared::ApiError,
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

/// O que se vê de uma partida: estado na chave, mapas e série decidida.
#[derive(PartialEq)]
struct Mark {
    state: MatchState,
    games: Vec<GameMark>,
    result: Option<ResultMark>,
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
        Self {
            state: m.state,
            games,
            result,
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
                    .map(|e| e.user_id),
            );
        }
        if let Some(pair) = ready(m.state)
            && was.and_then(|w| ready(w.state)) != Some(pair)
        {
            match_ready(conn, &audience, &after.entrants, &key, pair).await?;
        }
    }
    outbox::emit(conn, Event::me(players)).await?;
    if status != tournament.status {
        tournaments::changed(conn, tournament.id).await?;
    }
    // Contestou, resolveu, corrigiu, anulou, deu W.O.: a lista de mapas
    // contestados da home da organização mudou.
    if before.contested() != contested(after) {
        tenants::organizing_changed(conn, tournament.tenant).await?;
    }
    Ok(())
}

/// Ao iniciar: os dois lados de cada partida que já nasce pronta.
pub async fn started(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    bracket: &EliminationBracket,
    entrants: &[Seeded],
) -> Result<(), ApiError> {
    let audience = tournaments::audience(conn, tournament.id).await?;
    for m in bracket.matches() {
        if let Some(pair) = ready(m.state) {
            match_ready(conn, &audience, entrants, &key::format(m.id), pair).await?;
        }
    }
    Ok(())
}

/// `match.ready` para cada lado, com o apelido do outro.
async fn match_ready(
    conn: &mut PgConnection,
    audience: &Audience,
    entrants: &[Seeded],
    key: &str,
    (a, b): (ParticipantId, ParticipantId),
) -> Result<(), sqlx::Error> {
    let entrant = |p: ParticipantId| entrants.iter().find(|e| e.id == p.0);
    let (Some(a), Some(b)) = (entrant(a), entrant(b)) else {
        return Ok(());
    };
    for (to, opponent) in [(a, b), (b, a)] {
        let notice = Notice {
            kind: NoticeKind::MatchReady {
                tournament: audience.name.clone(),
                match_key: key.to_owned(),
                opponent: opponent.nickname.clone(),
            },
            path: match_path(audience, key),
        };
        outbox::emit(conn, Event::notice([to.user_id], notice)).await?;
    }
    Ok(())
}

fn match_path(audience: &Audience, key: &str) -> String {
    format!("{}/matches/{key}", audience.path())
}

/// `from` enviou o mapa `game` da partida `key`: aviso para `to`, o
/// adversário, com o prazo para confirmar ou contestar.
pub async fn game_reported(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    key: &str,
    game: i16,
    (from, to): (&Seeded, &Seeded),
    confirm_by: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let audience = tournaments::audience(conn, tournament.id).await?;
    let notice = Notice {
        kind: NoticeKind::GameReported {
            tournament: audience.name.clone(),
            match_key: key.to_owned(),
            game,
            opponent: from.nickname.clone(),
            confirm_by,
        },
        path: match_path(&audience, key),
    };
    outbox::emit(conn, Event::notice([to.user_id], notice)).await
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
