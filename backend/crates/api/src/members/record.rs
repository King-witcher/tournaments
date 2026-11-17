//! As abas da página de um membro além do cabeçalho: os torneios em que se
//! inscreveu, a carreira (séries, mapas, frags, títulos, sequência e as
//! últimas séries), os mapas paginados com filtro e o confronto direto por
//! adversário. Contam todos os torneios da comunidade, com rating ligado ou
//! não; o efeito no rating só aparece nos mapas que contaram.

use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgConnection;

use super::{
    career, db, history,
    types::{
        GamesQuery, MemberCareer, MemberGames, MemberOpponents, MemberPath, MemberTournament,
        OpponentsQuery,
    },
};
use crate::{
    AppState, ratings,
    shared::{ApiError, ErrorBody, PathParams, QueryParams},
    teams,
    tenants::{TenantId, TenantMember},
};

/// As últimas séries na Visão geral.
const RECENT_SERIES: usize = 5;
/// O que é pódio: a faixa começa entre os três primeiros (o 3º–4º da
/// eliminação simples também).
const PODIUM: u16 = 3;
/// Mapas por página na aba Partidas.
const GAMES_PAGE: i64 = 25;

/// 404 `member.not_found` se `user` não é membro ativo do tenant.
async fn active_member(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
) -> Result<(), ApiError> {
    db::role_of(&mut *conn, tenant, user)
        .await?
        .map(|_| ())
        .ok_or(ApiError::not_found("member.not_found"))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members/{user_id}/tournaments",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    responses(
        (status = OK, body = Vec<MemberTournament>, description = "Os torneios da comunidade em que o membro se inscreveu e que quem vê vê na lista dela (player não vê rascunho), do início mais recente ao mais antigo (sem data, só rascunhos, primeiro; no empate, o criado por último), com a situação e a colocação dele. As confirmações por tempo vencidas são aplicadas antes"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo)"),
    )
)]
pub async fn list_member_tournaments(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MemberPath>,
) -> Result<Json<Vec<MemberTournament>>, ApiError> {
    let mut conn = state.db.acquire().await?;
    active_member(&mut conn, member.tenant, path.user_id).await?;
    history::of(&mut conn, &member, path.user_id)
        .await
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members/{user_id}/career",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    responses(
        (status = OK, body = MemberCareer, description = "A carreira do membro em todos os torneios da comunidade, com rating ou não: séries decididas (W.O. incluído; a série é de quem jogou algum mapa confirmado dela ou, sem nenhum, do time), mapas confirmados, frags dos mapas com placar, títulos, pódios e a melhor colocação (dos torneios que quem vê vê; o empate a decidir não conta), a sequência atual e as 5 últimas séries"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo)"),
    )
)]
pub async fn get_member_career(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MemberPath>,
) -> Result<Json<MemberCareer>, ApiError> {
    let user = path.user_id;
    let mut conn = state.db.acquire().await?;
    active_member(&mut conn, member.tenant, user).await?;
    let entries = history::of(&mut conn, &member, user).await?;
    let placements = career::placements(&entries, PODIUM);
    let series = db::series(&mut conn, member.tenant, user, None).await?;
    let totals = db::game_totals(&mut conn, member.tenant, user).await?;
    let recent = &series[..series.len().min(RECENT_SERIES)];
    let ids: Vec<i64> = recent.iter().map(|s| s.opponent_team).collect();
    let opponents = teams::by_ids(&mut conn, member.tenant, &ids).await?;
    let recent_series = career::with_opponents(recent, &opponents)?;
    Ok(Json(MemberCareer {
        series: career::record(&series),
        games: totals.record,
        frags_for: totals.frags_for,
        frags_against: totals.frags_against,
        titles: placements.titles,
        podiums: placements.podiums,
        best_placement: placements.best,
        streak: career::streak(&series),
        recent_series,
    }))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members/{user_id}/games",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
        GamesQuery,
    ),
    responses(
        (status = OK, body = MemberGames, description = "Uma página (25) dos mapas confirmados do membro em todos os torneios da comunidade, do mais recente, com o efeito no rating nos que contaram (recalculado antes de ler, se algo mudou), quantos o filtro acha e as opções de cada filtro. Página além do fim: vazia, com o total"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path, request.invalid_query (página abaixo de 1)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo)"),
    )
)]
pub async fn list_member_games(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MemberPath>,
    QueryParams(query): QueryParams<GamesQuery>,
) -> Result<Json<MemberGames>, ApiError> {
    let page = query.page.unwrap_or(1);
    if page < 1 {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "request.invalid_query",
        ));
    }
    let user = path.user_id;
    let mut conn = state.db.acquire().await?;
    active_member(&mut conn, member.tenant, user).await?;
    ratings::fresh_on(&mut conn, member.tenant).await?;
    let filter = db::GamesFilter {
        tournament: query.tournament.as_deref(),
        map: query.map.as_deref(),
        opponent: query.opponent,
    };
    let offset = (page - 1).saturating_mul(GAMES_PAGE);
    let (games, total) =
        db::games_page(&mut conn, member.tenant, user, &filter, GAMES_PAGE, offset).await?;
    let filters = db::game_filters(&mut conn, member.tenant, user).await?;
    Ok(Json(MemberGames {
        total,
        page,
        page_size: GAMES_PAGE,
        games,
        filters,
    }))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members/{user_id}/opponents",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
        OpponentsQuery,
    ),
    responses(
        (status = OK, body = MemberOpponents, description = "O confronto direto do membro com cada pessoa que esteve do outro lado, em todos os torneios da comunidade (com rating ou não; duelo e TDM juntos, ou só a `modality`): séries decididas (no TDM, quem jogou algum mapa confirmado dela; no W.O., os times) e mapas confirmados em lados opostos, e o último confronto. Inclui quem saiu da comunidade (`active` falso)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path, request.invalid_query"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo)"),
    )
)]
pub async fn list_member_opponents(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MemberPath>,
    QueryParams(query): QueryParams<OpponentsQuery>,
) -> Result<Json<MemberOpponents>, ApiError> {
    let user = path.user_id;
    let mut conn = state.db.acquire().await?;
    active_member(&mut conn, member.tenant, user).await?;
    let series = db::series(&mut conn, member.tenant, user, query.modality).await?;
    let games = db::games_by_opponent(&mut conn, member.tenant, user, query.modality).await?;
    let mut ids: Vec<i64> = series
        .iter()
        .flat_map(|s| s.opponents.iter().copied())
        .chain(games.iter().map(|g| g.user_id))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let people = db::people(&mut conn, member.tenant, &ids).await?;
    Ok(Json(MemberOpponents {
        opponents: career::opponents(&series, &games, people),
    }))
}
