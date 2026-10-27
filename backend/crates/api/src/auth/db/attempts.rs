//! Erros de senha recentes, para o limite de tentativas (`attempts.rs`).

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Até quando `key` está bloqueado: com `max` erros na janela, até o mais
/// antigo deles sair dela.
pub async fn password_blocked_until(
    db: &PgPool,
    key: &str,
    max: i64,
    window_minutes: i32,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select min(failed_at) + make_interval(mins => $2) as "until!"
           from (select failed_at from password_failures
                 where username = $1 and failed_at > now() - make_interval(mins => $2)
                 order by failed_at desc
                 limit $3) recent
           having count(*) >= $3"#,
        key,
        window_minutes,
        max
    )
    .fetch_optional(db)
    .await
}

/// Registra um erro de senha e apaga os que já saíram da janela (de todos).
pub async fn record_password_failure(
    db: &PgPool,
    key: &str,
    window_minutes: i32,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query!(
        "delete from password_failures where failed_at <= now() - make_interval(mins => $1)",
        window_minutes
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("insert into password_failures (username) values ($1)", key)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub async fn clear_password_failures(db: &PgPool, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from password_failures where username = $1", key)
        .execute(db)
        .await?;
    Ok(())
}
