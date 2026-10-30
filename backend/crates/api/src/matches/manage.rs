//! O que o organizador (organizer ou owner, inclusive em partida que ele
//! joga) faz numa partida: lança um mapa, já confirmado (o próximo, a
//! resolução de um contestado com outro resultado, ou a correção de um
//! confirmado), e dá W.O. na série. A correção que muda a série decidida
//! refaz a chave por replay e é recusada se contradiz uma partida decidida
//! depois (`correction`). Tudo fica no histórico, com autor e horário.

use axum::{Json, extract::State};
use engine::{
    core::{MatchState, ParticipantId},
    series,
};

use super::{
    actions, correction, db, detail, due,
    games::{GameStatus, StoredGame, series_games},
    history::NewEvent,
    outcome, settle,
    types::{EventKind, GamePath, MatchDetail, MatchPath, ResultInput, SetGame, SetResult},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
};

#[utoipa::path(
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
        ("game" = i16, Path, description = "Número do mapa na série, a partir de 1"),
    ),
    request_body = SetGame,
    responses(
        (status = OK, body = MatchDetail, description = "Mapa definido pelo organizador, já confirmado: o próximo da série, ou no lugar de um enviado (resolve o contestado, corrige o confirmado, também com o torneio encerrado). Os mapas que ficam depois da decisão da série são anulados. Se a série decidida muda, a chave é refeita; se fica sem decisão, reabre"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "match.not_ready, match.series_decided, match.no_games_left {best_of}, match.game_out_of_order {next}, match.correction_conflicts (uma partida decidida depois deixaria de fechar)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "Os do envio (match.map_unknown, match.map_repeated, match.frags_invalid {min, max}, match.game_tied {game}, match.frags_disagree {game}, match.map_required: frags sem mapa), match.comment_invalid, request.invalid_body"),
    )
)]
pub async fn put_game(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<GamePath>,
    JsonBody(body): JsonBody<SetGame>,
) -> Result<Json<MatchDetail>, ApiError> {
    let comment = body.comment.as_deref().map(actions::comment).transpose()?;
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let m = detail::find(&stage, &path.key)?;
    let sides = playable(m.state)?;
    let games = stage.games_of(m.id);
    let index = actions::game_index(path.game, games.len())?;
    let maps = actions::map_ids(&mut tx, &body.game).await?;
    let new = outcome::game(path.game, sides, &body.game, &maps, false)?;
    let candidate = series::SeriesGame {
        winner: body.game.winner.into(),
        status: series::GameStatus::Confirmed,
        map: new.map,
    };
    let after = series::put(m.best_of, &series_games(&games), index, candidate)?;
    let kind = match games.get(index).map(|g| g.status) {
        Some(GameStatus::Contested) => EventKind::Resolved,
        Some(GameStatus::Confirmed) => EventKind::Corrected,
        Some(GameStatus::Pending) | None => EventKind::Entered,
    };
    let change = correction::series_change(stage.result(m.id), after.decided, sides);
    if let Some(change) = &change {
        correction::void_stale(&mut tx, &member, &tournament, &stage, m.id, change).await?;
    }

    let at = actions::at(&member, &tournament, &stage, &path.key);
    db::put_game(&mut tx, &at, &new, member.user.id).await?;
    let event = NewEvent::game(kind, Some(member.user.id), path.game, Some(new.winner))
        .with_comment(comment.as_deref());
    db::insert_event(&mut tx, &at, &event).await?;
    let beyond: Vec<&StoredGame> = games
        .iter()
        .enumerate()
        .filter(|(i, _)| *i >= after.standing && *i != index)
        .map(|(_, g)| *g)
        .collect();
    settle::void(&mut tx, &at, &beyond, Some(member.user.id)).await?;
    if let Some(change) = change {
        let series = (path.key.as_str(), m.best_of);
        correction::write(&mut tx, &member, &tournament, &stage, series, sides, change).await?;
    }
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/result",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    request_body = SetResult,
    responses(
        (status = OK, body = MatchDetail, description = "W.O. na série inteira, dado pelo organizador: numa partida pronta ou no lugar da série decidida (também com o torneio encerrado). Os mapas confirmados ficam; os pendentes e contestados são anulados. A correção refaz a chave; mapas de partidas cujos lados mudaram são anulados; o torneio encerra ou volta para em andamento conforme a chave"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "match.not_ready, match.correction_conflicts (uma partida decidida depois deixaria de fechar)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "match.comment_invalid, request.invalid_body"),
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
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let m = detail::find(&stage, &path.key)?;
    let sides = playable(m.state)?;
    let ResultInput::Walkover { winner } = body.result;
    let outcome = outcome::walkover(sides, winner);
    if stage.result(m.id).is_some() {
        let change = correction::Change::Put {
            winner: outcome.winner(),
            score: None,
        };
        correction::void_stale(&mut tx, &member, &tournament, &stage, m.id, &change).await?;
    }

    let at = actions::at(&member, &tournament, &stage, &path.key);
    let open: Vec<&StoredGame> = stage
        .games_of(m.id)
        .into_iter()
        .filter(|g| g.status != GameStatus::Confirmed)
        .collect();
    settle::void(&mut tx, &at, &open, Some(member.user.id)).await?;
    db::put_by_organizer(
        &mut tx,
        &at,
        m.best_of,
        sides,
        outcome.winner(),
        member.user.id,
    )
    .await?;
    let event = NewEvent::series(EventKind::Walkover, Some(member.user.id), Some(outcome))
        .with_comment(comment.as_deref());
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

/// Os lados da partida pronta ou decidida (o organizador também corrige).
/// 409 `match.not_ready` antes de os lados existirem.
fn playable(state: MatchState) -> Result<(ParticipantId, ParticipantId), ApiError> {
    match state {
        MatchState::Ready { a, b } | MatchState::Completed { a, b, .. } => Ok((a, b)),
        _ => Err(ApiError::conflict("match.not_ready")),
    }
}
