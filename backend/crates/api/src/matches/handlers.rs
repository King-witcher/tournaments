use axum::{Json, extract::State};

use super::{
    detail, due,
    types::{MatchDetail, MatchPath, TournamentBracket},
    view::{self, Viewer},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, PathParams},
    tenants::TenantMember,
    tournaments::TournamentPath,
};

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/bracket",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = TournamentBracket, description = "A chave, refeita dos resultados confirmados (as confirmações por tempo vencidas são aplicadas antes), com as partidas de quem vê e o que esperam dele"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found (inclusive privado para quem não vê), bracket.not_found (ainda não começou)"),
    )
)]
pub async fn get_bracket(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<TournamentBracket>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let (tournament, stage) = due::current(&mut conn, &member, &path.tournament).await?;
    let viewer = Viewer::new(&member, &stage);
    Ok(Json(view::bracket(&tournament, &stage, &viewer)?))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "A partida, com lados, resultado, detalhe por mapa e histórico"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
    )
)]
pub async fn get_match(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let (tournament, stage) = due::current(&mut conn, &member, &path.tournament).await?;
    let viewer = Viewer::new(&member, &stage);
    let m = detail::find(&stage, &path.key)?;
    Ok(Json(
        detail::load(&mut conn, &tournament, &stage, &viewer, m).await?,
    ))
}
