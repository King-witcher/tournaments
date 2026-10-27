use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::types::Invite;
use crate::tenants::TenantId;

/// Grava o convite (só o hash do token) e devolve id e validade.
pub async fn create(
    db: &PgPool,
    tenant: TenantId,
    token_hash: &[u8; 32],
    created_by: i64,
    valid_days: i32,
) -> Result<(i64, DateTime<Utc>), sqlx::Error> {
    let row = sqlx::query!(
        r#"insert into tenant_invites (tenant_id, token_hash, created_by, expires_at)
           values ($1, $2, $3, now() + make_interval(days => $4))
           returning id, expires_at"#,
        tenant.get(),
        &token_hash[..],
        created_by,
        valid_days
    )
    .fetch_one(db)
    .await?;
    Ok((row.id, row.expires_at))
}

/// Convites nem revogados nem expirados, do mais novo ao mais antigo.
pub async fn list_active(db: &PgPool, tenant: TenantId) -> Result<Vec<Invite>, sqlx::Error> {
    sqlx::query_as!(
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
        tenant.get()
    )
    .fetch_all(db)
    .await
}

/// Revoga (de novo não muda a data). `false` se o convite não é deste tenant.
pub async fn revoke(db: &PgPool, tenant: TenantId, invite_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "update tenant_invites set revoked_at = coalesce(revoked_at, now()) where tenant_id = $1 and id = $2",
        tenant.get(),
        invite_id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub struct StoredInvite {
    pub id: i64,
    /// O token é a autorização para entrar neste tenant.
    pub tenant: TenantId,
    pub tenant_name: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub expired: bool,
}

pub async fn find_by_token_hash(
    db: &PgPool,
    token_hash: &[u8; 32],
) -> Result<Option<StoredInvite>, sqlx::Error> {
    let row = sqlx::query!(
        r#"select i.id, i.tenant_id, t.name as tenant_name, i.expires_at,
                  i.revoked_at is not null as "revoked!",
                  i.expires_at <= now() as "expired!"
           from tenant_invites i
           join tenants t on t.id = i.tenant_id
           where i.token_hash = $1"#,
        &token_hash[..]
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| StoredInvite {
        id: r.id,
        tenant: TenantId::new(r.tenant_id),
        tenant_name: r.tenant_name,
        expires_at: r.expires_at,
        revoked: r.revoked,
        expired: r.expired,
    }))
}

/// Entra no tenant como player; quem já é membro mantém papel e apelido.
pub async fn join_as_player(
    db: &PgPool,
    tenant: TenantId,
    user_id: i64,
    nickname: &str,
    invite_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into tenant_members (tenant_id, user_id, role, nickname, invite_id)
           values ($1, $2, 'player', $3, $4)
           on conflict (tenant_id, user_id) do nothing"#,
        tenant.get(),
        user_id,
        nickname,
        invite_id
    )
    .execute(db)
    .await?;
    Ok(())
}
