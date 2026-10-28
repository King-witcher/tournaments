//! O organizador (organizer ou owner, inclusive em partida que ele joga)
//! anula um mapa ainda sem confirmar, pendente ou contestado, para os
//! jogadores o jogarem de novo: o mapa sai da série, o histórico guarda quem
//! anulou e a nota, e o mesmo número volta a poder ser enviado. Só o último
//! mapa da série: anular um do meio deixaria um buraco na numeração.

use axum::{Json, extract::State};

use super::{
    actions, db, detail, due,
    games::GameStatus,
    history::NewEvent,
    stage,
    types::{EventKind, GamePath, MatchDetail, ReplayGame},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
};

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/replay",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
        ("game" = i16, Path, description = "Número do mapa na série, a partir de 1"),
    ),
    request_body = ReplayGame,
    responses(
        (status = OK, body = MatchDetail, description = "Mapa anulado pelo organizador para ser jogado de novo: sai da série (evento `replay` no histórico, com a nota) e o mesmo número volta a poder ser enviado. Só o último mapa da série, pendente ou contestado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found, match.game_not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, match.already_confirmed (inclusive pela janela, que já acabou), match.game_not_last {last}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "match.comment_invalid, request.invalid_body"),
    )
)]
pub async fn replay(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<GamePath>,
    JsonBody(body): JsonBody<ReplayGame>,
) -> Result<Json<MatchDetail>, ApiError> {
    let comment = body.comment.as_deref().map(actions::comment).transpose()?;
    let mut tx = state.db.begin().await?;
    let tournament = due::lock_current(&mut tx, &member, &path.tournament).await?;
    let stage = stage::load(&mut tx, &tournament).await?;
    actions::in_progress(&tournament)?;
    let m = detail::find(&stage, &path.key)?;
    actions::ready(m)?;
    let games = stage.games_of(m.id);
    let g = actions::game(&games, path.game)?;
    if g.status == GameStatus::Confirmed {
        return Err(ApiError::conflict("match.already_confirmed"));
    }
    // Os números vão de 1 a n, sem buracos: o último é o n.
    let last = games.len();
    if usize::try_from(g.number).ok() != Some(last) {
        return Err(ApiError::conflict("match.game_not_last").with_params([("last", last as i64)]));
    }

    let at = actions::at(&member, &tournament, &stage, &path.key);
    db::void_games(&mut tx, &[g.id]).await?;
    let event = NewEvent::game(EventKind::Replay, Some(member.user.id), g.number, None)
        .with_comment(comment.as_deref());
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}
