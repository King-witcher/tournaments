//! SQL das inscrições de Web Push (`push_subscriptions`).

use sqlx::PgPool;

use super::sender::Target;
use crate::shared::Locale;

/// Inscreve o navegador do `endpoint` para `user_id`: se o endpoint já
/// existia (de outra conta, ou com outras chaves), passa a ser deste.
pub async fn upsert(
    db: &PgPool,
    user_id: i64,
    (endpoint, p256dh, auth): (&str, &str, &str),
    locale: Locale,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into push_subscriptions (user_id, endpoint, p256dh, auth, locale)
           values ($1, $2, $3, $4, $5)
           on conflict (endpoint) do update
           set user_id = excluded.user_id, p256dh = excluded.p256dh, auth = excluded.auth,
               locale = excluded.locale"#,
        user_id,
        endpoint,
        p256dh,
        auth,
        locale.as_str()
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Tira a inscrição do `endpoint`, se for de `user_id`.
pub async fn delete(db: &PgPool, user_id: i64, endpoint: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from push_subscriptions where endpoint = $1 and user_id = $2",
        endpoint,
        user_id
    )
    .execute(db)
    .await?;
    Ok(())
}

/// As inscrições de `users`.
pub async fn of_users(db: &PgPool, users: &[i64]) -> Result<Vec<Target>, sqlx::Error> {
    sqlx::query_as!(
        Target,
        r#"select id, endpoint, p256dh, auth from push_subscriptions
           where user_id = any($1)
           order by id"#,
        users
    )
    .fetch_all(db)
    .await
}

/// Apaga as inscrições que o serviço de push disse não existirem mais.
pub async fn remove(db: &PgPool, ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from push_subscriptions where id = any($1)", ids)
        .execute(db)
        .await?;
    Ok(())
}

/// Marca o último aviso entregue.
pub async fn mark_used(db: &PgPool, ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update push_subscriptions set last_used_at = now() where id = any($1)",
        ids
    )
    .execute(db)
    .await?;
    Ok(())
}
