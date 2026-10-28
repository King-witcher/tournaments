//! Links de uso único que vão por email para uma conta que já existe:
//! - `password_reset:{hash do token}`: o `user_id` da conta que pediu;
//! - `email_change:{hash do token}`: `user_id` e o `email` novo;
//! - `password_reset_user:{user_id}` e `email_change_user:{user_id}`: o hash
//!   do último link da conta, para um pedido novo derrubar o anterior.
//!
//! Tudo expira com o link; abrir o link apaga a chave dele.

use std::collections::HashMap;

use redis::{Cmd, RedisResult};

use crate::shared::{Redis, token};

#[derive(Clone, Copy)]
enum Kind {
    PasswordReset,
    EmailChange,
}

impl Kind {
    const fn prefix(self) -> &'static str {
        match self {
            Self::PasswordReset => "password_reset",
            Self::EmailChange => "email_change",
        }
    }
}

fn link_key(redis: &Redis, kind: Kind, hash: &str) -> String {
    redis.key(format_args!("{}:{hash}", kind.prefix()))
}

fn user_key(redis: &Redis, kind: Kind, user_id: i64) -> String {
    redis.key(format_args!("{}_user:{user_id}", kind.prefix()))
}

/// Link de recuperação de senha da conta; devolve o token.
pub async fn create_password_reset(
    redis: &Redis,
    user_id: i64,
    ttl_secs: u64,
) -> RedisResult<String> {
    create(redis, Kind::PasswordReset, user_id, &[], ttl_secs).await
}

/// Conta do link de recuperação, que deixa de valer.
pub async fn take_password_reset(redis: &Redis, link: &str) -> RedisResult<Option<i64>> {
    Ok(take(redis, Kind::PasswordReset, link)
        .await?
        .map(|(user_id, _)| user_id))
}

/// Link que confirma `email` como o novo email da conta; devolve o token.
pub async fn create_email_change(
    redis: &Redis,
    user_id: i64,
    email: &str,
    ttl_secs: u64,
) -> RedisResult<String> {
    create(
        redis,
        Kind::EmailChange,
        user_id,
        &[("email", email)],
        ttl_secs,
    )
    .await
}

/// Conta e email novo do link de troca, que deixa de valer.
pub async fn take_email_change(redis: &Redis, link: &str) -> RedisResult<Option<(i64, String)>> {
    Ok(take(redis, Kind::EmailChange, link)
        .await?
        .and_then(|(user_id, mut fields)| Some((user_id, fields.remove("email")?))))
}

async fn create(
    redis: &Redis,
    kind: Kind,
    user_id: i64,
    extra: &[(&str, &str)],
    ttl_secs: u64,
) -> RedisResult<String> {
    let link = token::random_token();
    let hash = token::hex(&token::hash_token(&link));
    let key = link_key(redis, kind, &hash);
    let last = user_key(redis, kind, user_id);
    let previous: Option<String> = redis.query(&Cmd::get(&last)).await?;

    let user = user_id.to_string();
    let mut fields = vec![("user_id", user.as_str())];
    fields.extend_from_slice(extra);
    let mut pipe = redis::pipe();
    pipe.atomic();
    if let Some(old) = previous {
        pipe.del(link_key(redis, kind, &old)).ignore();
    }
    pipe.hset_multiple(&key, &fields)
        .ignore()
        .expire(&key, ttl_secs as i64)
        .ignore()
        .set_ex(&last, &hash, ttl_secs)
        .ignore();
    redis.query_pipe::<()>(&pipe).await?;
    Ok(link)
}

async fn take(
    redis: &Redis,
    kind: Kind,
    link: &str,
) -> RedisResult<Option<(i64, HashMap<String, String>)>> {
    let key = link_key(redis, kind, &token::hex(&token::hash_token(link)));
    let (fields,): (HashMap<String, String>,) = redis
        .query_pipe(redis::pipe().atomic().hgetall(&key).del(&key).ignore())
        .await?;
    Ok(fields
        .get("user_id")
        .and_then(|id| id.parse().ok())
        .map(|user_id| (user_id, fields)))
}
