use sqlx::PgPool;

use super::types::Member;
use crate::{
    shared::ApiError,
    tenants::{Role, TenantId},
};

/// Membros do tenant (ou só `only_user`), owner primeiro, depois organizers e
/// players, cada grupo por apelido.
pub async fn list(
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

/// Papel do usuário no tenant; `None` se ele não é membro.
pub async fn role_of(
    db: &PgPool,
    tenant: TenantId,
    user_id: i64,
) -> Result<Option<Role>, ApiError> {
    sqlx::query_scalar!(
        "select role from tenant_members where tenant_id = $1 and user_id = $2",
        tenant.get(),
        user_id
    )
    .fetch_optional(db)
    .await?
    .map(|role| Role::from_db(&role))
    .transpose()
}

pub async fn set_role(
    db: &PgPool,
    tenant: TenantId,
    user_id: i64,
    role: Role,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tenant_members set role = $3 where tenant_id = $1 and user_id = $2",
        tenant.get(),
        user_id,
        role.as_str()
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn set_nickname(
    db: &PgPool,
    tenant: TenantId,
    user_id: i64,
    nickname: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tenant_members set nickname = $3 where tenant_id = $1 and user_id = $2",
        tenant.get(),
        user_id,
        nickname
    )
    .execute(db)
    .await?;
    Ok(())
}
