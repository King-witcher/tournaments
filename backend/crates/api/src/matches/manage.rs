//! O que o organizador (organizer ou owner, inclusive em partida que ele
//! joga) faz numa partida: define o resultado, que já nasce confirmado. Numa
//! partida pronta, lança o placar ou resolve a contestação com outro; numa
//! jogada, corrige (a chave é refeita por replay com o histórico corrigido, e
//! a correção que contradiz uma partida confirmada depois é recusada). W.O.
//! só por aqui. Tudo fica no histórico, com autor e horário.

use axum::{Json, extract::State};
use engine::core::{MatchId, MatchState};

use super::{
    actions, db,
    db::{MatchRef, NewResult},
    detail, due,
    history::NewEvent,
    key,
    outcome::{self, Proposed},
    result::{ResultStatus, StoredResult},
    stage::{self, Stage},
    types::{EventKind, MatchDetail, MatchPath, ResultInput, SetResult},
};
use crate::{
    AppState, maps,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
};

#[utoipa::path(
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/result",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
    ),
    request_body = SetResult,
    responses(
        (status = OK, body = MatchDetail, description = "Resultado definido pelo organizador, já confirmado: lança, resolve a contestação, dá W.O. ou corrige um resultado confirmado (também com o torneio encerrado). A correção refaz a chave; resultados pendentes de partidas cujos lados mudaram são anulados; o torneio encerra ou volta para em andamento conforme a chave"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "match.not_ready, match.correction_conflicts (uma partida confirmada depois deixaria de fechar)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "Os do envio (match.invalid_score {best_of}, detalhe por mapa...), match.comment_invalid, request.invalid_body"),
    )
)]
pub async fn set_result(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<MatchPath>,
    JsonBody(body): JsonBody<SetResult>,
) -> Result<Json<MatchDetail>, ApiError> {
    let comment = body.comment.as_deref().map(actions::comment).transpose()?;
    let mut tx = state.db.begin().await?;
    let tournament = due::lock_current(&mut tx, &member, &path.tournament).await?;
    let stage = stage::load(&mut tx, &tournament).await?;
    let m = detail::find(&stage, &path.key)?;
    let (sides, correcting) = match m.state {
        MatchState::Ready { a, b } => ((a, b), false),
        MatchState::Completed { a, b, .. } => ((a, b), true),
        _ => return Err(ApiError::conflict("match.not_ready")),
    };
    let proposed = match &body.result {
        ResultInput::Played {
            a_wins,
            b_wins,
            games,
        } => {
            let maps = maps::active_ids(&mut tx, &actions::map_slugs(games)).await?;
            outcome::played(m.best_of, sides, (*a_wins, *b_wins), games, &maps)?
        }
        ResultInput::Walkover { winner } => outcome::walkover(sides, *winner),
    };
    let previous = stage.result(m.id);
    let kind = if proposed.score.is_none() {
        EventKind::Walkover
    } else if correcting {
        EventKind::Corrected
    } else if previous.is_some_and(|r| r.status == ResultStatus::Contested) {
        EventKind::Resolved
    } else {
        EventKind::Entered
    };

    if correcting {
        for stale in correction_voids(&stage, m.id, &proposed)? {
            let stale_key = key::format(stale.match_id);
            let at = actions::at(&member, &tournament, &stage, &stale_key);
            void(&mut tx, &at, stale, member.user.id).await?;
        }
    }
    let at = actions::at(&member, &tournament, &stage, &path.key);
    let new = NewResult {
        best_of: m.best_of,
        sides,
        outcome: proposed.outcome,
        score: proposed.score,
        by: member.user.id,
    };
    let id = db::put_by_organizer(&mut tx, &at, &new).await?;
    db::replace_games(&mut tx, id, &proposed.games).await?;
    let event = NewEvent {
        kind,
        actor: Some(member.user.id),
        outcome: Some(proposed.outcome),
        comment: comment.as_deref(),
    };
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

/// Confere a correção da partida `id` refazendo a chave com o resultado novo
/// na posição do antigo. 409 `match.correction_conflicts` se uma partida
/// confirmada depois deixa de fechar (outro vencedor, outros lados, reset que
/// deixa de existir). Devolve os pendentes e contestados que ficam sem
/// partida (os lados mudaram), para anular.
fn correction_voids<'a>(
    stage: &'a Stage,
    id: MatchId,
    proposed: &Proposed,
) -> Result<Vec<&'a StoredResult>, ApiError> {
    let mut corrected = stage.results.clone();
    let current = corrected
        .iter_mut()
        .find(|r| r.match_id == id)
        .ok_or_else(ApiError::internal)?;
    current.winner = proposed.outcome.winner();
    current.score = proposed.score;
    let bracket = stage::replay(&stage.format, &stage.entrants, &corrected)
        .map_err(|_| ApiError::conflict("match.correction_conflicts"))?;
    Ok(stage
        .results
        .iter()
        .filter(|r| r.status != ResultStatus::Confirmed)
        .filter(|r| {
            !matches!(
                bracket.get(r.match_id).map(|m| m.state),
                Some(MatchState::Ready { a, b }) if (a, b) == (r.side_a, r.side_b)
            )
        })
        .collect())
}

/// Anula o resultado pendente ou contestado `stale`, com o evento.
async fn void(
    conn: &mut sqlx::PgConnection,
    at: &MatchRef<'_>,
    stale: &StoredResult,
    by: i64,
) -> Result<(), ApiError> {
    db::void(conn, stale.id).await?;
    let event = NewEvent {
        kind: EventKind::Voided,
        actor: Some(by),
        outcome: None,
        comment: None,
    };
    db::insert_event(conn, at, &event).await?;
    Ok(())
}
