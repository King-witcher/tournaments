//! Exclusão do tenant: agendar, cancelar e, na data, apagar.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::{shared::ApiError, tenants::TenantId};

/// Agenda a exclusão do tenant para daqui a `days` dias, pedida pelo owner
/// `by`, e devolve a data. Torneio em andamento → 409
/// `tenant.tournament_in_progress`: a regra é do banco
/// (`tenants_deletion_in_progress`), que trava a linha do tenant contra o
/// início de um torneio no mesmo instante.
pub async fn schedule_deletion(
    conn: &mut PgConnection,
    tenant: TenantId,
    by: i64,
    days: i32,
) -> Result<DateTime<Utc>, ApiError> {
    let result = sqlx::query_scalar!(
        r#"update tenants
           set deletion_due_at = now() + make_interval(days => $3), deletion_requested_by = $2
           where id = $1
           returning deletion_due_at as "due!""#,
        tenant.get(),
        by,
        days
    )
    .fetch_one(conn)
    .await;
    match result {
        Ok(due) => Ok(due),
        Err(sqlx::Error::Database(err))
            if err.constraint() == Some("tenants_deletion_in_progress") =>
        {
            Err(ApiError::conflict("tenant.tournament_in_progress"))
        }
        Err(err) => Err(err.into()),
    }
}

/// Cancela a exclusão agendada. `false` se não havia.
pub async fn cancel_deletion(
    conn: &mut PgConnection,
    tenant: TenantId,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"update tenants set deletion_due_at = null, deletion_requested_by = null
           where id = $1 and deletion_due_at is not null"#,
        tenant.get()
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Tenants cuja exclusão venceu em `now`, para o agendador (que age pelo
/// sistema, como nos torneios vencidos).
pub async fn due_deletions(db: &PgPool, now: DateTime<Utc>) -> Result<Vec<TenantId>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        "select id from tenants where deletion_due_at <= $1 order by id",
        now
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(TenantId::new)
    .collect())
}

/// Trava o tenant até o fim da transação, se a exclusão dele ainda vence em
/// `now` (o owner não cancelou desde a busca).
pub async fn lock_due_deletion(
    conn: &mut PgConnection,
    tenant: TenantId,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        "select id from tenants where id = $1 and deletion_due_at <= $2 for update",
        tenant.get(),
        now
    )
    .fetch_optional(conn)
    .await?
    .is_some())
}

/// Membros ativos do tenant, que perdem o acesso quando ele é excluído.
pub async fn active_members(
    conn: &mut PgConnection,
    tenant: TenantId,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        "select user_id from tenant_members where tenant_id = $1 and left_at is null order by user_id",
        tenant.get()
    )
    .fetch_all(conn)
    .await
}

/// Apaga o tenant; a cascata leva tudo o que está dentro.
pub async fn delete(conn: &mut PgConnection, tenant: TenantId) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from tenants where id = $1", tenant.get())
        .execute(conn)
        .await?;
    Ok(())
}
