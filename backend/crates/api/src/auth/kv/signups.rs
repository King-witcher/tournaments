//! Cadastros com senha esperando a confirmação do email. A conta só vai para
//! o banco quando o link é aberto ou o código é digitado; até lá, e por no
//! máximo o TTL do link (`email_secrets.rs`):
//! - `signup:{email}`: o registro, com `display_name`, `password_hash` (argon2)
//!   e `locale`;
//! - `signup_token:{hash do token}`: o email do cadastro.
//!
//! Um cadastro por email: pedir de novo troca o anterior.

use std::collections::HashMap;

use redis::RedisResult;

use super::email_secrets::{Family, Secrets};
use crate::shared::{Locale, Redis};

const SIGNUP: Family = Family {
    record: "signup",
    pointer: "signup_token",
};

pub struct PendingSignup {
    pub display_name: String,
    /// Forma canônica.
    pub email: String,
    pub password_hash: String,
    pub locale: Locale,
}

impl PendingSignup {
    fn from_fields(email: String, mut fields: HashMap<String, String>) -> Option<Self> {
        let locale = match fields.get("locale").map(String::as_str) {
            Some("es") => Locale::Es,
            _ => Locale::Pt,
        };
        Some(Self {
            display_name: fields.remove("display_name")?,
            password_hash: fields.remove("password_hash")?,
            email,
            locale,
        })
    }
}

/// Desfaz o cadastro de `email` que espera a confirmação: o link e o código
/// param de valer.
pub async fn drop_signup(redis: &Redis, email: &str) -> RedisResult<()> {
    SIGNUP.cancel(redis, email).await
}

/// Guarda o cadastro no lugar do anterior do mesmo email e devolve o token do
/// link e o código.
pub async fn store_signup(
    redis: &Redis,
    pending: &PendingSignup,
    ttl_secs: u64,
) -> RedisResult<Secrets> {
    let fields = [
        ("display_name", pending.display_name.as_str()),
        ("password_hash", pending.password_hash.as_str()),
        ("locale", pending.locale.as_str()),
    ];
    SIGNUP.store(redis, &pending.email, &fields, ttl_secs).await
}

/// O cadastro deste link, que sai do Redis (o link não vale de novo); `None`
/// se o link expirou, já foi usado ou foi trocado por um mais novo.
pub async fn take_signup(redis: &Redis, link: &str) -> RedisResult<Option<PendingSignup>> {
    Ok(SIGNUP
        .take_by_link(redis, link)
        .await?
        .and_then(|(email, fields)| PendingSignup::from_fields(email, fields)))
}

/// O cadastro de `email`, que sai do Redis, se o código confere; `None` em
/// qualquer recusa, sem dizer qual, para o código não revelar quem tem conta (o
/// email com conta não tem cadastro esperando).
pub async fn take_signup_by_code(
    redis: &Redis,
    email: &str,
    code: &str,
) -> RedisResult<Option<PendingSignup>> {
    Ok(SIGNUP
        .check_code(redis, email, code)
        .await?
        .and_then(|fields| PendingSignup::from_fields(email.to_owned(), fields)))
}
