//! Sessões e o usuário dono delas.

use sqlx::PgPool;

use crate::{
    auth::{CurrentUser, session::SESSION_DAYS},
    shared::token,
};

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

pub async fn user_by_id(db: &PgPool, id: i64) -> Result<CurrentUser, sqlx::Error> {
    sqlx::query_as!(
        CurrentUser,
        "select id, display_name, avatar_url, locale from users where id = $1",
        id
    )
    .fetch_one(db)
    .await
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

/// Encerra as sessões da conta, menos a de hash `keep`.
pub async fn delete_other_sessions(
    db: &PgPool,
    user_id: i64,
    keep: &[u8; 32],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from sessions where user_id = $1 and id <> $2",
        user_id,
        &keep[..]
    )
    .execute(db)
    .await?;
    Ok(())
}
