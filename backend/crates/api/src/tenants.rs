use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::{
    AppState,
    current_user::CurrentUser,
    error::{ApiError, ErrorBody},
    extract::{JsonBody, PathParams},
    tenant_member::{Owner, Role, TenantId, TenantMember},
};

/// Quantos tenants um usuário pode ter como owner (ser membro de outros não conta).
pub const MAX_OWNED_TENANTS: i64 = 3;

const SLUG_LEN: std::ops::RangeInclusive<usize> = 3..=32;
const NAME_MAX: usize = 64;
const NICKNAME_MAX: usize = 32;

/// Slugs que colidem com rotas do front em `/{locale}/...` (hoje `new` e
/// `invite`) ou que ficam guardados para rotas futuras.
const RESERVED_SLUGS: &[&str] = &[
    "about",
    "admin",
    "api",
    "assets",
    "auth",
    "comunidad",
    "comunidade",
    "comunidades",
    "docs",
    "health",
    "help",
    "invite",
    "login",
    "logout",
    "new",
    "privacy",
    "settings",
    "static",
    "tenants",
    "terms",
    "torneios",
    "torneos",
    "tournaments",
    "users",
];

#[derive(Deserialize, ToSchema)]
pub struct CreateTenant {
    /// Nome exibido, 1 a 64 caracteres.
    #[schema(example = "Quake BR")]
    pub name: String,
    /// Identificador na URL: a-z, 0-9 e hífen, 3 a 32 caracteres. Não muda depois.
    #[schema(example = "quake-br")]
    pub slug: String,
}

/// Um tenant do ponto de vista do usuário logado.
#[derive(Serialize, ToSchema)]
pub struct MyTenant {
    pub slug: String,
    pub name: String,
    pub role: Role,
    /// Apelido do usuário neste tenant.
    pub nickname: String,
}

impl<R> From<TenantMember<R>> for MyTenant {
    fn from(m: TenantMember<R>) -> Self {
        Self {
            slug: m.slug,
            name: m.name,
            role: m.role,
            nickname: m.nickname,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Member {
    pub user_id: i64,
    pub nickname: String,
    /// Nome global do usuário (do Discord).
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateMyMembership {
    /// 1 a 32 caracteres; espaços nas pontas são removidos.
    pub nickname: String,
}

/// Papéis que o owner pode dar. A posse do tenant não muda por aqui.
#[derive(Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AssignableRole {
    Organizer,
    Player,
}

impl From<AssignableRole> for Role {
    fn from(role: AssignableRole) -> Self {
        match role {
            AssignableRole::Organizer => Self::Organizer,
            AssignableRole::Player => Self::Player,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct SetRole {
    pub role: AssignableRole,
}

/// `{user_id}` de `/tenants/{slug}/members/{user_id}/...` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct MemberPath {
    user_id: i64,
}

fn validate_slug(slug: &str) -> Result<(), ApiError> {
    // Mesmo formato do check da tabela: ^[a-z0-9]+(-[a-z0-9]+)*$
    let well_formed = SLUG_LEN.contains(&slug.len())
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        });
    if !well_formed {
        return Err(ApiError::invalid("tenant.slug_invalid"));
    }
    if RESERVED_SLUGS.contains(&slug) {
        return Err(ApiError::invalid("tenant.slug_reserved"));
    }
    Ok(())
}

/// Texto de uma linha: sem espaços nas pontas, sem caracteres de controle,
/// de 1 a `max` caracteres.
fn clean_text(raw: &str, max: usize, code: &'static str) -> Result<String, ApiError> {
    let text = raw.trim();
    let len = text.chars().count();
    if len == 0 || len > max || text.chars().any(char::is_control) {
        return Err(ApiError::invalid(code));
    }
    Ok(text.to_owned())
}

/// Apelido inicial de quem entra num tenant: o nome do usuário, cortado no limite.
pub(crate) fn default_nickname(display_name: &str) -> String {
    let cut: String = display_name
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .chars()
        .take(NICKNAME_MAX)
        .collect();
    match cut.trim_end() {
        "" => "player".to_owned(),
        name => name.to_owned(),
    }
}

#[utoipa::path(
    post,
    path = "/tenants",
    tag = "tenants",
    request_body = CreateTenant,
    responses(
        (status = CREATED, body = MyTenant, description = "Tenant criado; quem criou é o owner"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.limit_reached, tenant.slug_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tenant.slug_invalid, tenant.slug_reserved, tenant.name_invalid, request.invalid_body"),
    )
)]
pub async fn create_tenant(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<CreateTenant>,
) -> Result<(StatusCode, Json<MyTenant>), ApiError> {
    validate_slug(&body.slug)?;
    let name = clean_text(&body.name, NAME_MAX, "tenant.name_invalid")?;
    let nickname = default_nickname(&user.display_name);

    let mut tx = state.db.begin().await?;
    // Trava a linha do usuário: duas criações simultâneas não furam o limite.
    sqlx::query_scalar!("select id from users where id = $1 for update", user.id)
        .fetch_one(&mut *tx)
        .await?;
    let owned = sqlx::query_scalar!(
        r#"select count(*) as "count!" from tenant_members where user_id = $1 and role = 'owner'"#,
        user.id
    )
    .fetch_one(&mut *tx)
    .await?;
    if owned >= MAX_OWNED_TENANTS {
        return Err(ApiError::conflict("tenant.limit_reached"));
    }

    let tenant_id = sqlx::query_scalar!(
        "insert into tenants (slug, name) values ($1, $2) on conflict (slug) do nothing returning id",
        body.slug,
        name
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::conflict("tenant.slug_taken"))?;
    sqlx::query!(
        "insert into tenant_members (tenant_id, user_id, role, nickname) values ($1, $2, 'owner', $3)",
        tenant_id,
        user.id,
        nickname
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(MyTenant {
            slug: body.slug,
            name,
            role: Role::Owner,
            nickname,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/tenants",
    tag = "tenants",
    responses(
        (status = OK, body = Vec<MyTenant>, description = "Tenants de que o usuário participa, na ordem em que entrou"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn list_my_tenants(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<MyTenant>>, ApiError> {
    let rows = sqlx::query!(
        r#"select t.slug, t.name, m.role, m.nickname
           from tenant_members m
           join tenants t on t.id = m.tenant_id
           where m.user_id = $1
           order by m.joined_at, t.id"#,
        user.id
    )
    .fetch_all(&state.db)
    .await?;

    rows.into_iter()
        .map(|r| {
            Ok(MyTenant {
                slug: r.slug,
                name: r.name,
                role: Role::from_db(&r.role)?,
                nickname: r.nickname,
            })
        })
        .collect::<Result<_, _>>()
        .map(Json)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = MyTenant, description = "O tenant e o papel do usuário nele"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found (não existe ou o usuário não é membro)"),
    )
)]
pub async fn get_tenant(member: TenantMember) -> Json<MyTenant> {
    Json(member.into())
}

#[utoipa::path(
    patch,
    path = "/tenants/{slug}/me",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = UpdateMyMembership,
    responses(
        (status = OK, body = MyTenant, description = "Apelido atualizado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "member.nickname_invalid, request.invalid_body"),
    )
)]
pub async fn update_my_membership(
    State(state): State<AppState>,
    member: TenantMember,
    JsonBody(body): JsonBody<UpdateMyMembership>,
) -> Result<Json<MyTenant>, ApiError> {
    let nickname = clean_text(&body.nickname, NICKNAME_MAX, "member.nickname_invalid")?;
    sqlx::query!(
        "update tenant_members set nickname = $3 where tenant_id = $1 and user_id = $2",
        member.tenant.get(),
        member.user.id,
        nickname
    )
    .execute(&state.db)
    .await?;
    Ok(Json(MyTenant {
        nickname,
        ..member.into()
    }))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Vec<Member>, description = "Membros: owner, organizers e players, cada grupo por apelido"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn list_members(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<Json<Vec<Member>>, ApiError> {
    fetch_members(&state.db, member.tenant, None)
        .await
        .map(Json)
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/members/{user_id}/role",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    request_body = SetRole,
    responses(
        (status = OK, body = Member, description = "Papel trocado (só o owner pode)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "member.is_owner (o papel do owner não muda)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn set_member_role(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    PathParams(path): PathParams<MemberPath>,
    JsonBody(body): JsonBody<SetRole>,
) -> Result<Json<Member>, ApiError> {
    let user_id = path.user_id;
    let current = sqlx::query_scalar!(
        "select role from tenant_members where tenant_id = $1 and user_id = $2",
        owner.tenant.get(),
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::not_found("member.not_found"))?;
    if Role::from_db(&current)? == Role::Owner {
        return Err(ApiError::conflict("member.is_owner"));
    }

    sqlx::query!(
        "update tenant_members set role = $3 where tenant_id = $1 and user_id = $2",
        owner.tenant.get(),
        user_id,
        Role::from(body.role).as_str()
    )
    .execute(&state.db)
    .await?;

    fetch_members(&state.db, owner.tenant, Some(user_id))
        .await?
        .pop()
        .map(Json)
        .ok_or(ApiError::not_found("member.not_found"))
}

/// Membros do tenant (ou só `only_user`), owner primeiro, depois organizers e players.
async fn fetch_members(
    db: &PgPool,
    tenant: TenantId,
    only_user: Option<i64>,
) -> Result<Vec<Member>, ApiError> {
    let rows = sqlx::query!(
        r#"select u.id as user_id, m.nickname, u.display_name, u.avatar_url, m.role, m.joined_at
           from tenant_members m
           join users u on u.id = m.user_id
           where m.tenant_id = $1 and ($2::bigint is null or m.user_id = $2)
           order by case m.role when 'owner' then 0 when 'organizer' then 1 else 2 end,
                    lower(m.nickname), m.user_id"#,
        tenant.get(),
        only_user
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|r| {
            Ok(Member {
                user_id: r.user_id,
                nickname: r.nickname,
                display_name: r.display_name,
                avatar_url: r.avatar_url,
                role: Role::from_db(&r.role)?,
                joined_at: r.joined_at,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_format() {
        for ok in ["abc", "quake-br", "qc2v2", "a-b-c", &"a".repeat(32)] {
            assert!(validate_slug(ok).is_ok(), "{ok}");
        }
        for bad in [
            "ab",
            &"a".repeat(33),
            "Quake",
            "quake_br",
            "-abc",
            "abc-",
            "a--b",
            "ação",
            "a b c",
        ] {
            assert_eq!(
                validate_slug(bad).unwrap_err().code,
                "tenant.slug_invalid",
                "{bad}"
            );
        }
        assert_eq!(
            validate_slug("invite").unwrap_err().code,
            "tenant.slug_reserved"
        );
    }

    #[test]
    fn clean_text_trims_and_limits() {
        assert_eq!(clean_text("  Quake BR ", 64, "x").unwrap(), "Quake BR");
        assert_eq!(clean_text(&"é".repeat(64), 64, "x").unwrap().len(), 128);
        for bad in ["", "   ", "a\nb", &"é".repeat(65)] {
            assert_eq!(clean_text(bad, 64, "x").unwrap_err().code, "x", "{bad:?}");
        }
    }

    #[test]
    fn default_nickname_fits_the_column() {
        assert_eq!(default_nickname("  Quaker "), "Quaker");
        assert_eq!(default_nickname(&"á".repeat(40)).chars().count(), 32);
        assert_eq!(
            default_nickname(&format!("{} x", "a".repeat(31))),
            "a".repeat(31)
        );
        assert_eq!(default_nickname(" \n "), "player");
    }
}
