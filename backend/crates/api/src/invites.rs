use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::{
    AppState,
    auth::hash_token,
    current_user::CurrentUser,
    error::{ApiError, ErrorBody},
    extract::PathParams,
    ids,
    tenant_member::{Organizer, Role, TenantId, TenantMember},
    tenants::{MyTenant, default_nickname},
};

/// Validade de um link de convite.
const INVITE_DAYS: i32 = 7;

#[derive(Serialize, ToSchema)]
pub struct CreatedInvite {
    pub id: i64,
    /// Token do link `/invite/{token}`. Só aparece nesta resposta: o banco guarda o hash.
    #[schema(example = "7Fq2xKpR9mWbTnVc3hJdYe")]
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Convite ativo (nem revogado nem expirado).
#[derive(Serialize, ToSchema)]
pub struct Invite {
    pub id: i64,
    /// Apelido, neste tenant, de quem criou.
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Quantas pessoas entraram por este link.
    pub uses: i64,
}

/// `{invite_id}` de `/tenants/{slug}/invites/{invite_id}` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct InvitePath {
    invite_id: i64,
}

/// O que quem recebeu o link vê antes de aceitar.
#[derive(Serialize, ToSchema)]
pub struct InvitePreview {
    pub tenant_name: String,
    pub expires_at: DateTime<Utc>,
}

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
    let token = ids::random_token();
    let hash = hash_token(&token);
    let row = sqlx::query!(
        r#"insert into tenant_invites (tenant_id, token_hash, created_by, expires_at)
           values ($1, $2, $3, now() + make_interval(days => $4))
           returning id, expires_at"#,
        member.tenant.get(),
        &hash[..],
        member.user.id,
        INVITE_DAYS
    )
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedInvite {
            id: row.id,
            token,
            expires_at: row.expires_at,
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
    let invites = sqlx::query_as!(
        Invite,
        r#"select i.id,
                  coalesce(m.nickname, u.display_name) as "created_by!",
                  i.created_at,
                  i.expires_at,
                  (select count(*) from tenant_members j
                   where j.tenant_id = i.tenant_id and j.invite_id = i.id) as "uses!"
           from tenant_invites i
           join users u on u.id = i.created_by
           left join tenant_members m on m.tenant_id = i.tenant_id and m.user_id = i.created_by
           where i.tenant_id = $1 and i.revoked_at is null and i.expires_at > now()
           order by i.created_at desc, i.id desc"#,
        member.tenant.get()
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(invites))
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
    let result = sqlx::query!(
        "update tenant_invites set revoked_at = coalesce(revoked_at, now()) where tenant_id = $1 and id = $2",
        member.tenant.get(),
        path.invite_id
    )
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
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
    sqlx::query!(
        r#"insert into tenant_members (tenant_id, user_id, role, nickname, invite_id)
           values ($1, $2, 'player', $3, $4)
           on conflict (tenant_id, user_id) do nothing"#,
        invite.tenant.get(),
        user.id,
        default_nickname(&user.display_name),
        invite.id
    )
    .execute(&state.db)
    .await?;

    let row = sqlx::query!(
        r#"select t.slug, t.name, m.role, m.nickname
           from tenant_members m
           join tenants t on t.id = m.tenant_id
           where m.tenant_id = $1 and m.user_id = $2"#,
        invite.tenant.get(),
        user.id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(MyTenant {
        slug: row.slug,
        name: row.name,
        role: Role::from_db(&row.role)?,
        nickname: row.nickname,
    }))
}

struct ValidInvite {
    id: i64,
    /// O token é a autorização para entrar neste tenant.
    tenant: TenantId,
    tenant_name: String,
    expires_at: DateTime<Utc>,
}

/// Convite pelo token: inexistente → 404, revogado ou expirado → 410.
async fn find_valid_invite(db: &PgPool, token: &str) -> Result<ValidInvite, ApiError> {
    let hash = hash_token(token);
    let row = sqlx::query!(
        r#"select i.id, i.tenant_id, t.name as tenant_name, i.expires_at,
                  i.revoked_at is not null as "revoked!",
                  i.expires_at <= now() as "expired!"
           from tenant_invites i
           join tenants t on t.id = i.tenant_id
           where i.token_hash = $1"#,
        &hash[..]
    )
    .fetch_optional(db)
    .await?
    .ok_or(ApiError::not_found("invite.not_found"))?;

    if row.revoked {
        return Err(ApiError::gone("invite.revoked"));
    }
    if row.expired {
        return Err(ApiError::gone("invite.expired"));
    }
    Ok(ValidInvite {
        id: row.id,
        tenant: TenantId::new(row.tenant_id),
        tenant_name: row.tenant_name,
        expires_at: row.expires_at,
    })
}
