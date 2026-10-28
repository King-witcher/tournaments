use sqlx::PgPool;

use super::{MyTenant, Role, TenantId};
use crate::shared::ApiError;

/// Participação ativa de um usuário num tenant, lida pelo extractor
/// `TenantMember`: quem saiu é tratado como quem é de fora.
pub struct Membership {
    pub tenant: TenantId,
    pub slug: String,
    pub name: String,
    pub role: Role,
    pub nickname: String,
}

pub async fn membership(
    db: &PgPool,
    slug: &str,
    user_id: i64,
) -> Result<Option<Membership>, ApiError> {
    let row = sqlx::query!(
        r#"select t.id, t.slug, t.name, m.role, m.nickname
           from tenants t
           join tenant_members m on m.tenant_id = t.id
           where t.slug = $1 and m.user_id = $2 and m.left_at is null"#,
        slug,
        user_id
    )
    .fetch_optional(db)
    .await?;

    row.map(|r| {
        Ok(Membership {
            tenant: TenantId::new(r.id),
            slug: r.slug,
            name: r.name,
            role: Role::from_db(&r.role)?,
            nickname: r.nickname,
        })
    })
    .transpose()
}

/// Cria o tenant com `owner_id` como owner, desde que ele ainda não seja
/// owner de `max_owned` tenants.
pub async fn create_with_owner(
    db: &PgPool,
    owner_id: i64,
    slug: &str,
    name: &str,
    nickname: &str,
    max_owned: i64,
) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    // Trava a linha do usuário: duas criações simultâneas não furam o limite.
    sqlx::query_scalar!("select id from users where id = $1 for update", owner_id)
        .fetch_one(&mut *tx)
        .await?;
    let owned = sqlx::query_scalar!(
        r#"select count(*) as "count!" from tenant_members where user_id = $1 and role = 'owner'"#,
        owner_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if owned >= max_owned {
        return Err(ApiError::conflict("tenant.limit_reached"));
    }

    let tenant_id = sqlx::query_scalar!(
        "insert into tenants (slug, name) values ($1, $2) on conflict (slug) do nothing returning id",
        slug,
        name
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::conflict("tenant.slug_taken"))?;
    sqlx::query!(
        "insert into tenant_members (tenant_id, user_id, role, nickname) values ($1, $2, 'owner', $3)",
        tenant_id,
        owner_id,
        nickname
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Tenants de que o usuário participa (sem os que deixou), na ordem em que
/// entrou.
pub async fn list_for_user(db: &PgPool, user_id: i64) -> Result<Vec<MyTenant>, ApiError> {
    let rows = sqlx::query!(
        r#"select t.slug, t.name, m.role, m.nickname
           from tenant_members m
           join tenants t on t.id = m.tenant_id
           where m.user_id = $1 and m.left_at is null
           order by m.joined_at, t.id"#,
        user_id
    )
    .fetch_all(db)
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
        .collect()
}

/// O tenant visto por um usuário que já é membro dele.
pub async fn my_tenant(db: &PgPool, tenant: TenantId, user_id: i64) -> Result<MyTenant, ApiError> {
    let row = sqlx::query!(
        r#"select t.slug, t.name, m.role, m.nickname
           from tenant_members m
           join tenants t on t.id = m.tenant_id
           where m.tenant_id = $1 and m.user_id = $2"#,
        tenant.get(),
        user_id
    )
    .fetch_one(db)
    .await?;
    Ok(MyTenant {
        slug: row.slug,
        name: row.name,
        role: Role::from_db(&row.role)?,
        nickname: row.nickname,
    })
}
