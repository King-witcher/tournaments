//! SQL das notificações: gravar na transação do despachante, listar, contar
//! as não lidas, marcar como lidas e apagar.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgConnection, PgExecutor, PgPool, types::Json};

use super::types::{Notification, NotificationPage};
use crate::{
    outbox::{self, Event, Notice},
    tenants::TenantId,
};

/// Guarda `notice` para cada um de `users` que ainda existe (o despachante
/// roda depois de quem emitiu) e devolve o id da notificação de cada um. Roda
/// na transação do despachante.
pub async fn store(
    conn: &mut PgConnection,
    users: &[i64],
    notice: &Notice,
) -> Result<HashMap<i64, i64>, sqlx::Error> {
    let rows = sqlx::query!(
        "insert into notifications (user_id, notice)
         select id, $2 from users where id = any($1)
         returning user_id, id",
        users,
        Json(notice) as _
    )
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|r| (r.user_id, r.id)).collect())
}

/// Quantas notificações de `user_id` faltam ler.
pub async fn unread_count(db: impl PgExecutor<'_>, user_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "count!" from notifications
           where user_id = $1 and read_at is null"#,
        user_id
    )
    .fetch_one(db)
    .await
}

/// As `limit` notificações mais recentes de `user_id` com id menor que
/// `before`; com `read`, só as lidas ou só as não lidas. A que a API não
/// entende mais fica de fora, com log, mas continua na contagem das não lidas
/// (quem tira um código do `NoticeKind` apaga as linhas dele na migration) e
/// no `next`, para a página seguinte não a buscar de novo.
pub async fn page(
    db: &PgPool,
    user_id: i64,
    read: Option<bool>,
    before: Option<i64>,
    limit: i64,
) -> Result<NotificationPage, sqlx::Error> {
    // Uma a mais diz se há outra página.
    let mut rows = sqlx::query!(
        r#"select id, notice as "notice: Json<Value>", created_at,
                  read_at is not null as "read!"
           from notifications
           where user_id = $1
             and ($2::bool is null or (read_at is not null) = $2)
             and ($3::bigint is null or id < $3)
           order by id desc
           limit $4"#,
        user_id,
        read,
        before,
        limit + 1
    )
    .fetch_all(db)
    .await?;
    let next = if rows.len() as i64 > limit {
        rows.truncate(limit as usize);
        rows.last().map(|r| r.id)
    } else {
        None
    };
    let notifications = rows
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
        .collect();
    Ok(NotificationPage {
        notifications,
        next,
    })
}

/// Marca como lidas as notificações de `user_id` até `up_to` e devolve quantas
/// mudaram.
pub async fn mark_read(
    conn: &mut PgConnection,
    user_id: i64,
    up_to: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        "update notifications set read_at = now()
         where user_id = $1 and id <= $2 and read_at is null",
        user_id,
        up_to
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected())
}

/// Marca como lidas as notificações de `user_id` que levam a `path`, sem
/// olhar as de outra página, e devolve quantas mudaram.
pub async fn mark_read_at_path(
    conn: &mut PgConnection,
    user_id: i64,
    path: &str,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        "update notifications set read_at = now()
         where user_id = $1 and read_at is null and notice ->> 'path' = $2",
        user_id,
        path
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected())
}

/// Quem da comunidade `tenant` tem notificação não vista que leva a `root` (o
/// caminho de um torneio, que vai ser excluído): a contagem dessas pessoas muda.
pub async fn unread_under(
    conn: &mut PgConnection,
    tenant: TenantId,
    root: &str,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select distinct user_id from notifications
           where user_id in (select user_id from tenant_members where tenant_id = $1)
             and read_at is null
             and path_under(notice ->> 'path', $2)"#,
        tenant.get(),
        root
    )
    .fetch_all(conn)
    .await
}

/// Diz a `users` quantas notificações faltam ver (depois de sumir alguma):
/// `Event::Unread`, como ao ver numa outra aba.
pub async fn recount(conn: &mut PgConnection, users: &[i64]) -> Result<(), sqlx::Error> {
    if users.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query!(
        r#"select u.id as "user!",
                  (select count(*) from notifications n
                   where n.user_id = u.id and n.read_at is null) as "count!"
           from unnest($1::bigint[]) as u(id)"#,
        users
    )
    .fetch_all(&mut *conn)
    .await?;
    let events = rows.into_iter().map(|r| Event::Unread {
        user: r.user,
        count: r.count,
    });
    outbox::emit_all(conn, events).await
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
