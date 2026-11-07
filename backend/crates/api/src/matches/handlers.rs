use axum::{Json, extract::State};

use super::{
    detail, due, stage,
    types::{BracketQuery, MatchDetail, MatchPath, TournamentBracket},
    view::{self, Viewer},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, PathParams, QueryParams},
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
        ("stage" = Option<i16>, Query, description = "A fase, de 1 em diante; ausente: a atual (a última que começou). A fase que não começou é 404 `stage.not_started` (o que se sabe dela antes, em `GET .../stages/{stage}`)"),
    ),
    responses(
        (status = OK, body = TournamentBracket, description = "A chave da fase, refeita dos resultados confirmados (as confirmações por tempo vencidas são aplicadas antes), com as partidas de quem vê e o que esperam dele, e a classificação final do torneio até aqui"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found (ainda não começou), stage.not_started (a fase pedida não começou ou não existe)"),
    )
)]
pub async fn get_bracket(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
    QueryParams(query): QueryParams<BracketQuery>,
) -> Result<Json<TournamentBracket>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let (tournament, stage) =
        due::current(&mut conn, &member, &path.tournament, query.stage).await?;
    let viewer = Viewer::new(&member, &stage);
    let chain = stage::chain(&mut conn, &tournament, &stage).await?;
    Ok(Json(view::bracket(&tournament, &stage, &viewer, &chain)?))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2; da fase 2 em diante, com o prefixo da fase (s2-pl-1-1)"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "A partida, com lados, resultado, detalhe por mapa e histórico"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, stage.not_started, match.not_found"),
    )
)]
pub async fn get_match(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let at = Some(path.key.stage);
    let (tournament, stage) = due::current(&mut conn, &member, &path.tournament, at).await?;
    let viewer = Viewer::new(&member, &stage);
    let m = detail::find(&stage, &path.key)?;
    Ok(Json(
        detail::load(&mut conn, &tournament, &stage, &viewer, m).await?,
    ))
}
