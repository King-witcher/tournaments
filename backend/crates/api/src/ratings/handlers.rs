use axum::{Json, extract::State, http::StatusCode};
use engine::rating::{MAX_ADJUSTMENT, PROVISIONAL_GAMES};

use super::{
    db,
    refresh::{elo, fresh, refresh},
    settings,
    types::{MemberRating, NewAdjustment, Ranking, RatingAdjustment, RatingPath, RatingSettings},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams, text::clean_text},
    tenants::{self, Organizer, Owner, Role, TenantMember},
};

/// Mapas na página de um membro.
const RECENT_GAMES: i64 = 20;
/// Tamanho do motivo de um ajuste (o mesmo check está na tabela).
const MAX_REASON: usize = 280;

#[utoipa::path(
    get,
    path = "/tenants/{slug}/ratings",
    tag = "ratings",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Ranking, description = "O ranking da comunidade: Elo por mapa confirmado dos torneios que contam, mais os ajustes da organização. Recalculado antes de ler se algo mudou"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn get_ranking(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<Json<Ranking>, ApiError> {
    fresh(&state.db, member.tenant).await?;
    let initial = elo(&mut *state.db.acquire().await?, member.tenant)
        .await?
        .config()
        .initial;
    let players = db::ranking(&state.db, member.tenant).await?;
    Ok(Json(Ranking {
        initial,
        provisional_games: PROVISIONAL_GAMES,
        players,
    }))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/ratings/{user_id}",
    tag = "ratings",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    responses(
        (status = OK, body = MemberRating, description = "O rating do membro e os últimos 20 mapas que contaram (só dos torneios que quem vê vê); para organizer e owner, também os ajustes"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo)"),
    )
)]
pub async fn get_member_rating(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<RatingPath>,
) -> Result<Json<MemberRating>, ApiError> {
    let user = path.user_id;
    active_member(&state, member.tenant, user).await?;
    fresh(&state.db, member.tenant).await?;
    let initial = elo(&mut *state.db.acquire().await?, member.tenant)
        .await?
        .config()
        .initial;
    let standing = db::ranking(&state.db, member.tenant)
        .await?
        .into_iter()
        .find(|e| e.user_id == user);
    let staff = member.role >= Role::Organizer;
    let games = db::recent_games(
        &state.db,
        member.tenant,
        user,
        member.user.id,
        staff,
        RECENT_GAMES,
    )
    .await?;
    let adjustments = db::adjustments(&state.db, member.tenant, user).await?;
    Ok(Json(MemberRating {
        user_id: user,
        standing,
        initial,
        provisional_games: PROVISIONAL_GAMES,
        adjusted: adjustments.iter().map(|a| a.delta).sum(),
        games,
        adjustments: staff.then_some(adjustments),
    }))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/ratings/{user_id}/adjustments",
    tag = "ratings",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    request_body = NewAdjustment,
    responses(
        (status = CREATED, body = RatingAdjustment, description = "Ajuste gravado, com quem fez e quando; o rating já foi recalculado"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "rating.delta_invalid {max} (zero ou além de ±max), rating.reason_invalid {max}, request.invalid_body"),
    )
)]
pub async fn create_adjustment(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<RatingPath>,
    JsonBody(body): JsonBody<NewAdjustment>,
) -> Result<(StatusCode, Json<RatingAdjustment>), ApiError> {
    if body.delta == 0 || !(-MAX_ADJUSTMENT..=MAX_ADJUSTMENT).contains(&body.delta) {
        return Err(ApiError::invalid("rating.delta_invalid")
            .with_params([("max", i64::from(MAX_ADJUSTMENT))]));
    }
    let reason = clean_text(&body.reason, MAX_REASON, "rating.reason_invalid")
        .map_err(|e| e.with_params([("max", MAX_REASON as i64)]))?;
    let user = path.user_id;
    active_member(&state, member.tenant, user).await?;

    let mut tx = state.db.begin().await?;
    let id = db::insert_adjustment(
        &mut tx,
        member.tenant,
        user,
        body.delta,
        &reason,
        member.user.id,
    )
    .await?;
    refresh(&mut tx, member.tenant).await?;
    tx.commit().await?;

    let created = db::adjustments(&state.db, member.tenant, user)
        .await?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(ApiError::internal)?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/ratings/settings",
    tag = "ratings",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = RatingSettings, description = "O Elo da comunidade (o padrão, se o owner nunca mudou)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn get_settings(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<Json<RatingSettings>, ApiError> {
    let elo = elo(&mut *state.db.acquire().await?, member.tenant).await?;
    Ok(Json(elo.config().into()))
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/ratings/settings",
    tag = "ratings",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = RatingSettings,
    responses(
        (status = OK, body = RatingSettings, description = "Gravado (só o owner); todos os ratings já foram recalculados com o histórico inteiro"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "rating.initial_invalid {min, max}, rating.k_min_invalid {min, max}, rating.k_max_invalid {min, max}, rating.k_decay_invalid {min, max}, rating.k_order_invalid (K mínimo acima do máximo), request.invalid_body"),
    )
)]
pub async fn put_settings(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<RatingSettings>,
) -> Result<Json<RatingSettings>, ApiError> {
    let config = settings::check(&body)?;
    let mut tx = state.db.begin().await?;
    db::put_settings(&mut tx, owner.tenant, &config, owner.user.id).await?;
    refresh(&mut tx, owner.tenant).await?;
    tx.commit().await?;
    Ok(Json(config.into()))
}

/// 404 `member.not_found` se `user` não é membro ativo do tenant.
async fn active_member(
    state: &AppState,
    tenant: tenants::TenantId,
    user: i64,
) -> Result<(), ApiError> {
    match tenants::standing(&state.db, tenant, user).await? {
        Some(s) if s.active => Ok(()),
        _ => Err(ApiError::not_found("member.not_found")),
    }
}
