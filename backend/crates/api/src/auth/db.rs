use sqlx::PgPool;

use super::{CurrentUser, session::SESSION_DAYS};
use crate::shared::token;

/// Usuário dono da sessão com este hash, se ela não expirou.
pub async fn user_by_session(
    db: &PgPool,
    hash: &[u8; 32],
) -> Result<Option<CurrentUser>, sqlx::Error> {
    sqlx::query_as!(
        CurrentUser,
        r#"select u.id, u.display_name, u.avatar_url, u.locale
           from sessions s
           join users u on u.id = s.user_id
           where s.id = $1 and s.expires_at > now()"#,
        &hash[..]
    )
    .fetch_optional(db)
    .await
}

/// Acha o usuário pela identidade do Discord (atualizando o avatar) ou cria
/// usuário e identidade. Devolve o id do usuário.
pub async fn upsert_discord_user(
    db: &PgPool,
    discord_id: &str,
    display_name: &str,
    avatar_url: Option<String>,
) -> Result<i64, sqlx::Error> {
    let mut tx = db.begin().await?;
    let existing = sqlx::query_scalar!(
        "select user_id from user_identities where provider = 'discord' and provider_user_id = $1",
        discord_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let user_id = match existing {
        Some(id) => {
            sqlx::query!(
                "update users set avatar_url = $1 where id = $2",
                avatar_url,
                id
            )
            .execute(&mut *tx)
            .await?;
            id
        }
        None => {
            let id = sqlx::query_scalar!(
                "insert into users (display_name, avatar_url) values ($1, $2) returning id",
                display_name,
                avatar_url
            )
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query!(
                "insert into user_identities (user_id, provider, provider_user_id) values ($1, 'discord', $2)",
                id,
                discord_id
            )
            .execute(&mut *tx)
            .await?;
            id
        }
    };
    tx.commit().await?;
    Ok(user_id)
}

/// Cria a sessão e devolve o token (só o hash vai para o banco).
pub async fn create_session(db: &PgPool, user_id: i64) -> Result<String, sqlx::Error> {
    let token = token::random_secret();
    let hash = token::hash_token(&token);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(SESSION_DAYS);
    sqlx::query!(
        "insert into sessions (id, user_id, expires_at) values ($1, $2, $3)",
        &hash[..],
        user_id,
        expires_at
    )
    .execute(db)
    .await?;
    Ok(token)
}

pub async fn delete_session(db: &PgPool, hash: &[u8; 32]) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from sessions where id = $1", &hash[..])
        .execute(db)
        .await?;
    Ok(())
}
