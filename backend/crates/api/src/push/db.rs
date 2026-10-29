//! SQL das inscrições de Web Push (`push_subscriptions`).

use sqlx::{PgConnection, PgPool};

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

/// As inscrições de `users`, cada uma com o idioma dos avisos.
pub async fn of_users(conn: &mut PgConnection, users: &[i64]) -> Result<Vec<Target>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"select id, endpoint, p256dh, auth, locale from push_subscriptions
           where user_id = any($1)
           order by id"#,
        users
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|r| {
            // O check da tabela só deixa pt e es.
            let locale = Locale::from_db(&r.locale)
                .map_err(|_| sqlx::Error::Decode(format!("locale {}", r.locale).into()))?;
            Ok(Target {
                id: r.id,
                endpoint: r.endpoint,
                p256dh: r.p256dh,
                auth: r.auth,
                locale,
            })
        })
        .collect()
}

/// Apaga as inscrições que o serviço de push disse não existirem mais.
pub async fn remove(conn: &mut PgConnection, ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from push_subscriptions where id = any($1)", ids)
        .execute(conn)
        .await?;
    Ok(())
}

/// Marca o último aviso entregue.
pub async fn mark_used(conn: &mut PgConnection, ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update push_subscriptions set last_used_at = now() where id = any($1)",
        ids
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Tira todas as inscrições de `user_id` (a conta excluída não recebe mais
/// aviso).
pub async fn forget_user(conn: &mut PgConnection, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from push_subscriptions where user_id = $1", user_id)
        .execute(conn)
        .await?;
    Ok(())
}
