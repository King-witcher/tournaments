use sqlx::{PgConnection, PgExecutor, PgPool};

use crate::{tenants::TenantId, tournaments::TournamentId};

/// O token do link de convite da comunidade.
pub async fn tenant_token(db: &PgPool, tenant: TenantId) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        "select invite_token from tenants where id = $1",
        tenant.get()
    )
    .fetch_one(db)
    .await
}

/// Troca o token do link de convite da comunidade: o antigo deixa de existir.
pub async fn set_tenant_token(
    conn: &mut PgConnection,
    tenant: TenantId,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tenants set invite_token = $2 where id = $1",
        tenant.get(),
        token
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O token do link de inscrição do torneio.
pub async fn tournament_token(
    db: &PgPool,
    tenant: TenantId,
    tournament: TournamentId,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        "select invite_token from tournaments where tenant_id = $1 and id = $2",
        tenant.get(),
        tournament.get()
    )
    .fetch_one(db)
    .await
}

/// Troca o token do link de inscrição do torneio: o antigo deixa de existir.
pub async fn set_tournament_token(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournaments set invite_token = $3 where tenant_id = $1 and id = $2",
        tenant.get(),
        tournament.get(),
        token
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub struct StoredInvite {
    /// O token é a autorização para entrar neste tenant.
    pub tenant: TenantId,
    pub tenant_slug: String,
    pub tenant_name: String,
    pub tenant_description: Option<String>,
    /// O objeto do logo da comunidade no bucket público.
    pub tenant_logo_key: Option<String>,
    /// O objeto da capa da comunidade no bucket público.
    pub tenant_cover_key: Option<String>,
    /// O apelido do dono atual na comunidade (a comunidade não grava quem a criou).
    pub owner_name: String,
    /// Membros ativos.
    pub member_count: i64,
    /// Slug do torneio, no link de inscrição.
    pub tournament: Option<String>,
    /// A comunidade tem a exclusão agendada: ninguém entra até lá.
    pub tenant_deleting: bool,
}

/// O convite de `token`, numa consulta só, pelo índice único de cada escopo.
/// O link de torneio em rascunho, que não recebe ninguém, não conta: responde
/// como link inexistente, sem mostrar o torneio.
pub async fn find_by_token(
    db: impl PgExecutor<'_>,
    token: &str,
) -> Result<Option<StoredInvite>, sqlx::Error> {
    let row = sqlx::query!(
        r#"with link as (
               select id as tenant_id, null::bigint as tournament_id
               from tenants where invite_token = $1
               union all
               select tenant_id, id from tournaments where invite_token = $1 and status <> 'draft'
           )
           select t.id as tenant_id, t.slug as tenant_slug, t.name as tenant_name,
                  tr.slug as "tournament_slug?",
                  t.description as tenant_description, t.logo_key, t.cover_key,
                  t.deletion_due_at is not null as "tenant_deleting!",
                  coalesce((select m.nickname from tenant_members m
                            where m.tenant_id = t.id and m.role = 'owner'), '') as "owner_name!",
                  (select count(*) from tenant_members m
                   where m.tenant_id = t.id and m.left_at is null) as "member_count!"
           from link
           join tenants t on t.id = link.tenant_id
           left join tournaments tr on tr.tenant_id = link.tenant_id and tr.id = link.tournament_id"#,
        token
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| StoredInvite {
        tenant: TenantId::new(r.tenant_id),
        tenant_slug: r.tenant_slug,
        tenant_name: r.tenant_name,
        tenant_description: r.tenant_description,
        tenant_logo_key: r.logo_key,
        tenant_cover_key: r.cover_key,
        owner_name: r.owner_name,
        member_count: r.member_count,
        tournament: r.tournament_slug,
        tenant_deleting: r.tenant_deleting,
    }))
}

/// Entra no tenant como player; quem já é membro mantém papel e apelido, e
/// quem tinha saído volta (a mesma linha, com o apelido de antes) como player.
pub async fn join_as_player(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
    nickname: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into tenant_members (tenant_id, user_id, role, nickname)
           values ($1, $2, 'player', $3)
           on conflict (tenant_id, user_id) do update
           set role = 'player', left_at = null, removed_by = null
           where tenant_members.left_at is not null"#,
        tenant.get(),
        user_id,
        nickname
    )
    .execute(conn)
    .await?;
    Ok(())
}
