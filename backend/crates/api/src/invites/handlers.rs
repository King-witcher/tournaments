use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;

use super::{
    db::{self, StoredInvite},
    types::{CreatedInvite, Invite, InvitePath, InvitePreview},
};
use crate::{
    AppState,
    auth::CurrentUser,
    shared::{ApiError, ErrorBody, PathParams, token},
    tenants::{self, MyTenant, Organizer, TenantMember, default_nickname},
};

/// Validade de um link de convite.
const INVITE_DAYS: i32 = 7;

#[utoipa::path(
    post,
    path = "/tenants/{slug}/invites",
    tag = "invites",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = CREATED, body = CreatedInvite, description = "Link multiuso, válido por 7 dias; quem entra por ele vira player"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn create_invite(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
) -> Result<(StatusCode, Json<CreatedInvite>), ApiError> {
    let token = token::random_token();
    let (id, expires_at) = db::create(
        &state.db,
        member.tenant,
        &token::hash_token(&token),
        member.user.id,
        INVITE_DAYS,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedInvite {
            id,
            token,
            expires_at,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/invites",
    tag = "invites",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Vec<Invite>, description = "Convites ativos, do mais novo ao mais antigo"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn list_invites(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
) -> Result<Json<Vec<Invite>>, ApiError> {
    Ok(Json(db::list_active(&state.db, member.tenant).await?))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/invites/{invite_id}",
    tag = "invites",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("invite_id" = i64, Path, description = "Id do convite"),
    ),
    responses(
        (status = NO_CONTENT, description = "Convite revogado (também se já estava)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, invite.not_found"),
    )
)]
pub async fn revoke_invite(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<InvitePath>,
) -> Result<StatusCode, ApiError> {
    if !db::revoke(&state.db, member.tenant, path.invite_id).await? {
        return Err(ApiError::not_found("invite.not_found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/invites/{token}",
    tag = "invites",
    params(("token" = String, Path, description = "Token do link de convite")),
    responses(
        (status = OK, body = InvitePreview, description = "Convite válido (não exige login)"),
        (status = NOT_FOUND, body = ErrorBody, description = "invite.not_found"),
        (status = GONE, body = ErrorBody, description = "invite.expired, invite.revoked"),
    )
)]
pub async fn preview_invite(
    State(state): State<AppState>,
    PathParams(token): PathParams<String>,
) -> Result<Json<InvitePreview>, ApiError> {
    let invite = find_valid_invite(&state.db, &token).await?;
    Ok(Json(InvitePreview {
        tenant_name: invite.tenant_name,
        expires_at: invite.expires_at,
    }))
}

#[utoipa::path(
    post,
    path = "/invites/{token}/accept",
    tag = "invites",
    params(("token" = String, Path, description = "Token do link de convite")),
    responses(
        (status = OK, body = MyTenant, description = "Entrou como player; quem já era membro mantém papel e apelido"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "invite.not_found"),
        (status = GONE, body = ErrorBody, description = "invite.expired, invite.revoked"),
    )
)]
pub async fn accept_invite(
    State(state): State<AppState>,
    user: CurrentUser,
    PathParams(token): PathParams<String>,
) -> Result<Json<MyTenant>, ApiError> {
    let invite = find_valid_invite(&state.db, &token).await?;
    db::join_as_player(
        &state.db,
        invite.tenant,
        user.id,
        &default_nickname(&user.display_name),
        invite.id,
    )
    .await?;
    tenants::my_tenant(&state.db, invite.tenant, user.id)
        .await
        .map(Json)
}

/// Convite pelo token: inexistente → 404, revogado ou expirado → 410.
async fn find_valid_invite(db: &PgPool, token: &str) -> Result<StoredInvite, ApiError> {
    let invite = db::find_by_token_hash(db, &token::hash_token(token))
        .await?
        .ok_or(ApiError::not_found("invite.not_found"))?;
    if invite.revoked {
        return Err(ApiError::gone("invite.revoked"));
    }
    if invite.expired {
        return Err(ApiError::gone("invite.expired"));
    }
    Ok(invite)
}
