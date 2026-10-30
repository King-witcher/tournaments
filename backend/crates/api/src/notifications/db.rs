//! SQL das notificações: gravar na transação do despachante, listar, contar
//! as não lidas, marcar como vistas e apagar.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgConnection, PgExecutor, PgPool, types::Json};

use super::types::Notification;
use crate::outbox::Notice;

/// Guarda `notice` para cada um de `users` que ainda existe (o despachante
/// roda depois de quem emitiu). Roda na transação do despachante.
pub async fn store(
    conn: &mut PgConnection,
    users: &[i64],
    notice: &Notice,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "insert into notifications (user_id, notice)
         select id, $2 from users where id = any($1)",
        users,
        Json(notice) as _
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Quantas notificações de `user_id` faltam ver.
pub async fn unread_count(db: impl PgExecutor<'_>, user_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "count!" from notifications
           where user_id = $1 and read_at is null"#,
        user_id
    )
    .fetch_one(db)
    .await
}

/// As `limit` notificações mais recentes de `user_id`. A que a API não
/// entende mais (um código que saiu) fica de fora, com log.
pub async fn recent(
    db: &PgPool,
    user_id: i64,
    limit: i64,
) -> Result<Vec<Notification>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select id, notice as "notice: Json<Value>", created_at,
                  read_at is not null as "read!"
           from notifications
           where user_id = $1
           order by id desc
           limit $2"#,
        user_id,
        limit
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            let notice = serde_json::from_value(r.notice.0)
                .inspect_err(|e| tracing::error!(id = r.id, %e, "notificação ilegível"))
                .ok()?;
            Some(Notification {
                id: r.id,
                notice,
                created_at: r.created_at,
                read: r.read,
            })
        })
        .collect())
}

/// Marca como vistas as notificações de `user_id` até `up_to` e devolve
/// quantas ainda faltam.
pub async fn mark_read(
    conn: &mut PgConnection,
    user_id: i64,
    up_to: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query!(
        "update notifications set read_at = now()
         where user_id = $1 and id <= $2 and read_at is null",
        user_id,
        up_to
    )
    .execute(&mut *conn)
    .await?;
    unread_count(conn, user_id).await
}

/// Apaga as notificações criadas antes de `before` e devolve quantas.
pub async fn delete_before(db: &PgPool, before: DateTime<Utc>) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!("delete from notifications where created_at < $1", before)
        .execute(db)
        .await?;
    Ok(done.rows_affected())
}

/// Apaga todas as notificações de `user_id` (a conta excluída).
pub async fn forget_user(conn: &mut PgConnection, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from notifications where user_id = $1", user_id)
        .execute(conn)
        .await?;
    Ok(())
}
