use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgExecutor, PgPool};

use super::types::{Invite, TournamentInvite};
use crate::{shared::token, tenants::TenantId, tournaments::TournamentId};

/// Grava o convite do tenant (o hash do token e os primeiros caracteres, para
/// reconhecê-lo) e devolve id e validade.
pub async fn create(
    conn: &mut PgConnection,
    tenant: TenantId,
    token: &str,
    created_by: i64,
    valid_days: i32,
) -> Result<(i64, DateTime<Utc>), sqlx::Error> {
    let row = sqlx::query!(
        r#"insert into tenant_invites (tenant_id, token_hash, token_hint, created_by, expires_at)
           values ($1, $2, $3, $4, now() + make_interval(days => $5))
           returning id, expires_at as "expires_at!""#,
        tenant.get(),
        &token::hash_token(token)[..],
        token::hint(token),
        created_by,
        valid_days
    )
    .fetch_one(conn)
    .await?;
    Ok((row.id, row.expires_at))
}

/// Grava o link de inscrição do torneio, sem validade (vale enquanto a
/// inscrição estiver aberta), e devolve o id.
pub async fn create_for_tournament(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    token: &str,
    created_by: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"insert into tenant_invites (tenant_id, tournament_id, token_hash, token_hint, created_by)
           values ($1, $2, $3, $4, $5)
           returning id"#,
        tenant.get(),
        tournament.get(),
        &token::hash_token(token)[..],
        token::hint(token),
        created_by
    )
    .fetch_one(conn)
    .await
}

/// Convites do tenant (não os de torneio) nem revogados nem expirados, do
/// mais novo ao mais antigo.
pub async fn list_active(db: &PgPool, tenant: TenantId) -> Result<Vec<Invite>, sqlx::Error> {
    sqlx::query_as!(
        Invite,
        r#"select i.id,
                  coalesce(m.nickname, u.display_name) as "created_by!",
                  i.created_at,
                  i.token_hint as hint,
                  i.expires_at as "expires_at!",
                  (select count(*) from tenant_members j
                   where j.tenant_id = i.tenant_id and j.invite_id = i.id) as "uses!"
           from tenant_invites i
           join users u on u.id = i.created_by
           left join tenant_members m on m.tenant_id = i.tenant_id and m.user_id = i.created_by
           where i.tenant_id = $1 and i.tournament_id is null
             and i.revoked_at is null and i.expires_at > now()
           order by i.created_at desc, i.id desc"#,
        tenant.get()
    )
    .fetch_all(db)
    .await
}

/// Links de inscrição não revogados do torneio, do mais novo ao mais antigo.
pub async fn list_for_tournament(
    db: &PgPool,
    tenant: TenantId,
    tournament: TournamentId,
) -> Result<Vec<TournamentInvite>, sqlx::Error> {
    sqlx::query_as!(
        TournamentInvite,
        r#"select i.id,
                  coalesce(m.nickname, u.display_name) as "created_by!",
                  i.created_at,
                  i.token_hint as hint,
                  (select count(*) from participants p
                   where p.tenant_id = i.tenant_id and p.invite_id = i.id) as "uses!"
           from tenant_invites i
           join users u on u.id = i.created_by
           left join tenant_members m on m.tenant_id = i.tenant_id and m.user_id = i.created_by
           where i.tenant_id = $1 and i.tournament_id = $2 and i.revoked_at is null
           order by i.created_at desc, i.id desc"#,
        tenant.get(),
        tournament.get()
    )
    .fetch_all(db)
    .await
}

/// Revoga (de novo não muda a data). `false` se o convite não é deste tenant.
pub async fn revoke(
    conn: &mut PgConnection,
    tenant: TenantId,
    invite_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "update tenant_invites set revoked_at = coalesce(revoked_at, now()) where tenant_id = $1 and id = $2",
        tenant.get(),
        invite_id
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Revoga os convites abertos que `user_id` criou no tenant (do tenant e
/// links de inscrição), quando ele deixa o tenant.
pub async fn revoke_created_by(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update tenant_invites set revoked_at = now()
           where tenant_id = $1 and created_by = $2 and revoked_at is null"#,
        tenant.get(),
        user_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub struct StoredInvite {
    pub id: i64,
    /// O token é a autorização para entrar neste tenant.
    pub tenant: TenantId,
    pub tenant_slug: String,
    pub tenant_name: String,
    /// Slug do torneio, no link de inscrição.
    pub tournament: Option<String>,
    /// Nulo no link de inscrição.
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub expired: bool,
    /// A comunidade tem a exclusão agendada: ninguém entra até lá.
    pub tenant_deleting: bool,
}

pub async fn find_by_token_hash(
    db: impl PgExecutor<'_>,
    token_hash: &[u8; 32],
) -> Result<Option<StoredInvite>, sqlx::Error> {
    let row = sqlx::query!(
        r#"select i.id, i.tenant_id, t.slug as tenant_slug, t.name as tenant_name, tr.slug as "tournament_slug?",
                  i.expires_at,
                  i.revoked_at is not null as "revoked!",
                  coalesce(i.expires_at <= now(), false) as "expired!",
                  t.deletion_due_at is not null as "tenant_deleting!"
           from tenant_invites i
           join tenants t on t.id = i.tenant_id
           left join tournaments tr on tr.tenant_id = i.tenant_id and tr.id = i.tournament_id
           where i.token_hash = $1"#,
        &token_hash[..]
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| StoredInvite {
        id: r.id,
        tenant: TenantId::new(r.tenant_id),
        tenant_slug: r.tenant_slug,
        tenant_name: r.tenant_name,
        tournament: r.tournament_slug,
        expires_at: r.expires_at,
        revoked: r.revoked,
        expired: r.expired,
        tenant_deleting: r.tenant_deleting,
    }))
}

/// Entra no tenant como player; quem já é membro mantém papel e apelido, e
/// quem tinha saído volta (a mesma linha, com o apelido de antes) como
/// player, por este convite.
pub async fn join_as_player(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
    nickname: &str,
    invite_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into tenant_members (tenant_id, user_id, role, nickname, invite_id)
           values ($1, $2, 'player', $3, $4)
           on conflict (tenant_id, user_id) do update
           set role = 'player', left_at = null, removed_by = null, invite_id = excluded.invite_id
           where tenant_members.left_at is not null"#,
        tenant.get(),
        user_id,
        nickname,
        invite_id
    )
    .execute(conn)
    .await?;
    Ok(())
}
