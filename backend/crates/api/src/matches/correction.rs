//! A série decidida que muda depois de mexer numa partida dela: a correção
//! do organizador (`manage`) e a contestação tardia do jogador (`report`).
//! A mudança é conferida refazendo a chave com ela (na posição da antiga) e
//! recusada se uma série decidida depois deixa de fechar; os mapas das
//! séries ainda abertas cujos lados mudaram são anulados.

use chrono::Utc;
use engine::{
    core::{MatchId, MatchState, ParticipantId},
    series::Decision,
};
use sqlx::PgConnection;

use super::{
    actions, db,
    history::NewEvent,
    key,
    result::{Side, StoredResult, played_or_walkover},
    settle,
    stage::{self, Stage},
    types::EventKind,
};
use crate::{shared::ApiError, tenants::TenantMember, tournaments::StoredTournament};

pub const CONFLICTS: ApiError = ApiError::conflict("match.correction_conflicts");

/// O que acontece com a série decidida depois de mudar uma partida.
pub enum Change {
    /// Passa a ser esta (`score`: partidas de `a` e de `b`; `None` no W.O.).
    Put {
        winner: ParticipantId,
        score: Option<(u8, u8)>,
    },
    /// Fica sem decisão: a série reabre.
    Remove,
}

/// A mudança na série decidida `current` quando as partidas passam a decidir
/// `decided`. `None`: nada muda (a mesma decisão; série ainda aberta, que o
/// `settle` fecha se for o caso; W.O. que as partidas não desfazem).
pub fn series_change(
    current: Option<&StoredResult>,
    decided: Option<Decision>,
    sides: (ParticipantId, ParticipantId),
) -> Option<Change> {
    let current = current?;
    match decided {
        Some(d) => {
            let winner = Side::from(d.winner).of(sides);
            let score = Some((d.a_wins, d.b_wins));
            (current.winner != winner || current.score != score)
                .then_some(Change::Put { winner, score })
        }
        None if current.score.is_none() => None,
        None => Some(Change::Remove),
    }
}

/// A chave com a mudança `change` na série decidida da partida `id`. Erro
/// `CONFLICTS` se uma série decidida depois deixa de fechar (outro vencedor,
/// outros lados, reset que deixa de existir).
pub fn check(
    stage: &Stage,
    id: MatchId,
    change: &Change,
) -> Result<engine::formats::StageBracket, ApiError> {
    let mut corrected = stage.results.clone();
    match change {
        Change::Put { winner, score } => {
            let current = corrected
                .iter_mut()
                .find(|r| r.match_id == id)
                .ok_or_else(ApiError::internal)?;
            current.winner = *winner;
            current.score = *score;
        }
        Change::Remove => corrected.retain(|r| r.match_id != id),
    }
    stage::replay(stage.format, &stage.history(&corrected)).map_err(|_| CONFLICTS)
}

/// Confere a mudança (`check`) e anula os mapas das séries ainda abertas
/// cujos lados mudaram, com `member` como autor.
pub async fn void_stale<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    tournament: &StoredTournament,
    stage: &Stage,
    id: MatchId,
    change: &Change,
) -> Result<(), ApiError> {
    let bracket = check(stage, id, change)?;
    let mut open: Vec<MatchId> = stage
        .games
        .iter()
        .map(|g| g.match_id)
        .filter(|m| *m != id && stage.result(*m).is_none())
        .collect();
    open.dedup();
    for m in open {
        let games = stage.games_of(m);
        let keeps = matches!(
            bracket.get(m).map(|x| x.state),
            Some(MatchState::Ready { a, b }) if games.iter().all(|g| g.sides() == (a, b))
        );
        if keeps {
            continue;
        }
        let key = key::format(m);
        let at = actions::at(member, tournament, stage, &key);
        settle::void(conn, &at, &games, Some(member.user.id)).await?;
    }
    Ok(())
}

/// Grava a mudança `change` na série decidida da partida `key` (lados
/// `sides`, md`best_of`), com o evento: `decided` com o placar novo, ou
/// `reopened` com `member` como autor. Quem chama já passou por `void_stale`.
pub async fn write<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    tournament: &StoredTournament,
    stage: &Stage,
    (key, best_of): (&str, u8),
    sides: (ParticipantId, ParticipantId),
    change: Change,
) -> Result<(), ApiError> {
    let at = actions::at(member, tournament, stage, key);
    let id = key::parse(key).ok_or_else(ApiError::internal)?;
    match change {
        Change::Put { winner, score } => {
            let score = score.ok_or_else(ApiError::internal)?;
            db::put_decided(conn, &at, best_of, sides, winner, score, Utc::now()).await?;
            let outcome = played_or_walkover(winner, Some(score));
            let event = NewEvent::series(EventKind::Decided, None, Some(outcome));
            db::insert_event(conn, &at, &event).await?;
        }
        Change::Remove => {
            let id = stage.result(id).ok_or_else(ApiError::internal)?.id;
            db::remove_result(conn, id).await?;
            let event = NewEvent::series(EventKind::Reopened, Some(member.user.id), None);
            db::insert_event(conn, &at, &event).await?;
        }
    }
    Ok(())
}
