//! O capitão do time adversário de quem contestou aceita a contestação, sem
//! esperar a organização: como o comentário não traz resultado, ele dá o placar
//! certo, e a partida fica confirmada com ele. Quem discordar ainda contesta de
//! novo (`contest::late`), e aí a organização é avisada como sempre. Só o capitão,
//! qualquer que seja a regra do torneio para o resultado (`acting`).

use axum::{Json, extract::State};
use engine::series;

use super::{
    actions, changes, contest as rules, db, detail, due,
    games::{GameStatus, series_games},
    history::NewEvent,
    outcome,
    stage::Wrote,
    types::{EventKind, GameInput, GamePath, MatchDetail},
    veto,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::TenantMember,
};

const NOT_ACCEPTER: ApiError = ApiError::forbidden("match.not_accepter");
const NOT_CONTESTED: ApiError = ApiError::conflict("match.not_contested");
/// Aceitar com o mesmo resultado seria recusar a contestação sem a organização.
const UNCHANGED: ApiError = ApiError::invalid("match.contest_unchanged");

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/contest/accept",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Série: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2"),
        ("game" = i16, Path, description = "Número da partida na série, a partir de 1"),
    ),
    request_body = GameInput,
    responses(
        (status = OK, body = MatchDetail, description = "O capitão do time adversário de quem contestou aceita a contestação e dá o resultado certo: a partida fica confirmada com ele, e quem contestou é avisado e ainda pode contestar de novo. Se as partidas confirmadas dão a maioria, a série fecha"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_accepter (só o capitão do time adversário de quem contestou)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found, match.game_not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.series_decided, match.not_ready, match.not_contested"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "match.contest_unchanged (o resultado é igual ao enviado), match.map_required, match.map_unknown, match.map_repeated, match.map_mismatch (outro mapa que o do veto), match.frags_invalid {min, max}, match.game_tied {game}, request.invalid_body"),
    )
)]
pub async fn accept_contest(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<GamePath>,
    JsonBody(body): JsonBody<GameInput>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    actions::in_progress(&tournament)?;
    let m = detail::find(&stage, &path.key)?;
    let games = stage.games_of(m.id);
    let g = actions::game(&games, path.game)?;
    if g.status != GameStatus::Contested {
        return Err(NOT_CONTESTED);
    }
    if rules::accepter(&stage, g) != Some(member.user.id) {
        return Err(NOT_ACCEPTER);
    }
    // Contestada, a série está aberta: quem a fecha é o `settle`.
    let sides = actions::ready(m)?;
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_ACCEPTER)?;
    let index = actions::game_index(path.game, games.len())?;
    let maps = actions::map_ids(&mut tx, &body).await?;
    let new = outcome::game(path.game, sides, &body, &maps)?;
    veto::check_report(&stage, m, path.game, Some(new.map))?;
    if g.frags == Some(new.frags) && g.map.as_ref().map(|map| map.id) == Some(new.map) {
        return Err(UNCHANGED);
    }
    let candidate = new.series_game(series::GameStatus::Confirmed);
    series::put(m.best_of, &series_games(&games), index, candidate)?;

    let at = actions::at(&member, &tournament, &stage, &path.key);
    let minutes = tournament.contest_minutes;
    db::accept_contest(&mut tx, g.id, &new, side, member.user.id, minutes).await?;
    let event = NewEvent::game(
        EventKind::Resolved,
        Some(member.user.id),
        path.game,
        Some(new.winner_id()),
    );
    db::insert_event(&mut tx, &at, &event).await?;
    if let Some(contester) = g.contested_by {
        let by = member.nickname.as_str();
        changes::contest_accepted(&mut tx, &tournament, &path.key, g.number, contester, by).await?;
    }
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Games, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}
