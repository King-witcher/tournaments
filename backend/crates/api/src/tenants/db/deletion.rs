//! Exclusão do tenant: agendar, cancelar, avisar o owner na véspera e, na
//! data, apagar.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::{shared::ApiError, tenants::TenantId};

/// O que os torneios do tenant dizem sobre excluí-lo.
pub struct TournamentHistory {
    /// Algum está em andamento ou encerrado (um início desfeito não conta).
    pub started: bool,
    pub in_progress: bool,
}

pub async fn tournament_history(
    db: &PgPool,
    tenant: TenantId,
) -> Result<TournamentHistory, sqlx::Error> {
    sqlx::query_as!(
        TournamentHistory,
        r#"select coalesce(bool_or(started_at is not null), false) as "started!",
                  coalesce(bool_or(status = 'in_progress'), false) as "in_progress!"
           from tournaments where tenant_id = $1"#,
        tenant.get()
    )
    .fetch_one(db)
    .await
}

/// Trava os torneios e o tenant até o fim da transação e diz se algum torneio
/// começou. `None`: o tenant já não existe. Os torneios vêm antes, na mesma
/// ordem do início (o torneio e, no trigger, o tenant), para os dois ao mesmo
/// tempo não se travarem um esperando o outro.
pub async fn lock_for_deletion(
    conn: &mut PgConnection,
    tenant: TenantId,
) -> Result<Option<bool>, sqlx::Error> {
    let started = sqlx::query_scalar!(
        r#"select started_at is not null as "started!"
           from tournaments where tenant_id = $1 for update"#,
        tenant.get()
    )
    .fetch_all(&mut *conn)
    .await?;
    let exists = sqlx::query_scalar!(
        "select id from tenants where id = $1 for update",
        tenant.get()
    )
    .fetch_optional(conn)
    .await?
    .is_some();
    Ok(exists.then(|| started.contains(&true)))
}

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
        r#"update tenants
           set deletion_due_at = null, deletion_requested_by = null, deletion_reminded_at = null
           where id = $1 and deletion_due_at is not null"#,
        tenant.get()
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Tenants cuja exclusão vence entre `now` e `until`, sem o aviso ao owner,
/// para o agendador.
pub async fn deletions_to_remind(
    db: &PgPool,
    now: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<Vec<TenantId>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        r#"select id from tenants
           where deletion_due_at > $1 and deletion_due_at <= $2 and deletion_reminded_at is null
           order by id"#,
        now,
        until
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(TenantId::new)
    .collect())
}

/// O aviso da exclusão para o owner que a pediu.
pub struct DeletionReminder {
    pub owner: i64,
    pub name: String,
    pub slug: String,
    pub due: DateTime<Utc>,
}

/// Marca o aviso da exclusão, se ainda vale (o owner não cancelou e ninguém
/// marcou desde a busca), e devolve o que vai nele.
pub async fn mark_deletion_reminded(
    conn: &mut PgConnection,
    tenant: TenantId,
    now: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<Option<DeletionReminder>, sqlx::Error> {
    sqlx::query_as!(
        DeletionReminder,
        r#"update tenants set deletion_reminded_at = $2
           where id = $1 and deletion_due_at > $2 and deletion_due_at <= $3
             and deletion_reminded_at is null
           returning deletion_requested_by as "owner!", name, slug, deletion_due_at as "due!""#,
        tenant.get(),
        now,
        until
    )
    .fetch_optional(conn)
    .await
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
