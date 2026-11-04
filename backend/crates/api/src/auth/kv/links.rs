//! Link e código que vão por email para uma conta que já existe, na forma de
//! `email_secrets.rs`, um vivo por conta e tipo:
//! - entrar por email: `email_login_pending:{user_id}`, com o `email` para onde
//!   foi, e `email_login_link:{hash do token}`;
//! - trocar o email: `email_change_pending:{user_id}`, com o `email` novo e a
//!   `session` que pediu (hash, em hexadecimal), e
//!   `email_change_link:{hash do token}`;
//! - provar o email de agora (conta só com email): `email_proof_pending:{user_id}`
//!   e `email_proof_link:{hash do token}` (o link não vai no email; só o código);
//!   provado, `email_proven:{hash da sessão}` diz que aquela sessão digitou o
//!   código.
//!
//! O que muda o acesso derruba o que mudaria de novo: remover o email, trocar a
//! senha e a senha nova ao entrar por email apagam a troca de email (quem a
//! pediu pode ser quem perdeu o acesso), e a troca morre também com a sessão que
//! a pediu; trocar ou remover o email e trocar a senha apagam o de entrar por
//! email (foi para o email de antes, ou é de quem sabia a senha de antes).

use redis::{Cmd, RedisResult};

use super::email_secrets::{Family, Secrets};
use crate::shared::Redis;

const EMAIL_LOGIN: Family = Family {
    record: "email_login_pending",
    pointer: "email_login_link",
};

const EMAIL_CHANGE: Family = Family {
    record: "email_change_pending",
    pointer: "email_change_link",
};

const EMAIL_PROOF: Family = Family {
    record: "email_proof_pending",
    pointer: "email_proof_link",
};

/// Link e código para entrar na conta `user_id`, mandados a `email`.
pub async fn create_email_login(
    redis: &Redis,
    user_id: i64,
    email: &str,
    ttl_secs: u64,
) -> RedisResult<Secrets> {
    EMAIL_LOGIN
        .store(redis, &user_id.to_string(), &[("email", email)], ttl_secs)
        .await
}

/// Conta e email para onde foi o link de entrar, que deixa de valer junto com o
/// código.
pub async fn take_email_login(redis: &Redis, link: &str) -> RedisResult<Option<(i64, String)>> {
    Ok(EMAIL_LOGIN
        .take_by_link(redis, link)
        .await?
        .and_then(|(user_id, mut fields)| Some((user_id.parse().ok()?, fields.remove("email")?))))
}

/// O email para onde foi o código de entrar na conta `user_id`, se ele confere;
/// ele e o link deixam de valer. Quem chama não diz por que recusou: errado e
/// sem pedido dão no mesmo, para a resposta não revelar quem tem conta.
pub async fn take_email_login_by_code(
    redis: &Redis,
    user_id: i64,
    code: &str,
) -> RedisResult<Option<String>> {
    Ok(EMAIL_LOGIN
        .check_code(redis, &user_id.to_string(), code)
        .await?
        .and_then(|mut fields| fields.remove("email")))
}

/// Derruba o link e o código de entrar por email da conta, se houver.
pub async fn cancel_email_login(redis: &Redis, user_id: i64) -> RedisResult<()> {
    EMAIL_LOGIN.cancel(redis, &user_id.to_string()).await
}

/// Uma troca de email que o link ou o código acabaram de confirmar.
pub struct ConfirmedChange {
    /// O email novo.
    pub email: String,
    /// Hash da sessão que pediu, em hexadecimal.
    pub session: String,
}

/// Link e código que confirmam `email` como o novo email da conta, pedidos pela
/// sessão de hash `session` (em hexadecimal).
pub async fn create_email_change(
    redis: &Redis,
    user_id: i64,
    email: &str,
    session: &str,
    ttl_secs: u64,
) -> RedisResult<Secrets> {
    let fields = [("email", email), ("session", session)];
    EMAIL_CHANGE
        .store(redis, &user_id.to_string(), &fields, ttl_secs)
        .await
}

/// A conta e a troca do link, que deixa de valer junto com o código.
pub async fn take_email_change(
    redis: &Redis,
    link: &str,
) -> RedisResult<Option<(i64, ConfirmedChange)>> {
    Ok(EMAIL_CHANGE
        .take_by_link(redis, link)
        .await?
        .and_then(|(user_id, mut fields)| {
            Some((
                user_id.parse().ok()?,
                ConfirmedChange {
                    email: fields.remove("email")?,
                    session: fields.remove("session")?,
                },
            ))
        }))
}

/// A troca, se o código da conta `user_id` confere; ele e o link deixam de valer.
/// Errado e sem pedido dão no mesmo: quem põe o email de outra conta não tem
/// pedido, e a resposta não pode revelar isso.
pub async fn take_email_change_by_code(
    redis: &Redis,
    user_id: i64,
    code: &str,
) -> RedisResult<Option<ConfirmedChange>> {
    Ok(EMAIL_CHANGE
        .check_code(redis, &user_id.to_string(), code)
        .await?
        .and_then(|mut fields| {
            Some(ConfirmedChange {
                email: fields.remove("email")?,
                session: fields.remove("session")?,
            })
        }))
}

/// Derruba o link e o código de troca de email da conta, se houver.
pub async fn cancel_email_change(redis: &Redis, user_id: i64) -> RedisResult<()> {
    EMAIL_CHANGE.cancel(redis, &user_id.to_string()).await
}

/// O código que prova o email de agora da conta `user_id`.
pub async fn create_email_proof(
    redis: &Redis,
    user_id: i64,
    ttl_secs: u64,
) -> RedisResult<Secrets> {
    EMAIL_PROOF
        .store(redis, &user_id.to_string(), &[], ttl_secs)
        .await
}

/// Derruba o código de prova da conta, se houver.
pub async fn cancel_email_proof(redis: &Redis, user_id: i64) -> RedisResult<()> {
    EMAIL_PROOF.cancel(redis, &user_id.to_string()).await
}

/// O código de prova da conta `user_id` confere; se sim, ele deixa de valer.
pub async fn take_email_proof_by_code(
    redis: &Redis,
    user_id: i64,
    code: &str,
) -> RedisResult<bool> {
    Ok(EMAIL_PROOF
        .check_code(redis, &user_id.to_string(), code)
        .await?
        .is_some())
}

fn proven_key(redis: &Redis, session: &str) -> String {
    redis.key(format_args!("email_proven:{session}"))
}

/// A sessão de hash `session` (em hexadecimal) digitou o código.
pub async fn mark_email_proven(redis: &Redis, session: &str, ttl_secs: u64) -> RedisResult<()> {
    redis
        .query(&Cmd::set_ex(proven_key(redis, session), 1, ttl_secs))
        .await
}

/// A sessão de hash `session` (em hexadecimal) digitou o código há pouco.
pub async fn email_is_proven(redis: &Redis, session: &str) -> RedisResult<bool> {
    redis.query(&Cmd::exists(proven_key(redis, session))).await
}
