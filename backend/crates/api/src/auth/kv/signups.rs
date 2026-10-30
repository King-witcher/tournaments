//! Cadastros com senha esperando a confirmação do email. A conta só vai para
//! o banco quando o link é aberto ou o código é digitado; até lá, e por no
//! máximo o TTL do link:
//! - `signup:{email}`: hash com `username`, `password_hash` (argon2),
//!   `locale`, `token` (hash do token do link, em hexadecimal), `code` (hash
//!   do código, com o do token como sal; sai quando as tentativas acabam) e
//!   `code_attempts` (códigos conferidos);
//! - `signup_token:{hash do token}`: o email do cadastro. Vale uma vez só
//!   (`GETDEL`), pelo link ou pelo código;
//! - `signup_username:{username em minúsculas}`: o email do cadastro que
//!   reservou o username, para outro cadastro não pegar o mesmo nome antes da
//!   confirmação.
//!
//! Um cadastro por email: pedir de novo troca o anterior (o link e o código
//! velhos param de valer e o username dele fica livre).

use std::collections::HashMap;

use redis::{Cmd, RedisResult};

use crate::{
    auth::username::username_key,
    shared::{Locale, Redis, token},
};

pub struct PendingSignup {
    pub username: String,
    /// Forma canônica.
    pub email: String,
    pub password_hash: String,
    pub locale: Locale,
}

/// O que vai no email do cadastro: o token do link e o código.
pub struct SignupSecrets {
    pub link: String,
    pub code: String,
}

/// O código digitado para o cadastro de um email.
pub enum CodeCheck {
    /// Certo: o cadastro, que sai do Redis (nem o link vale mais).
    Confirmed(PendingSignup),
    /// Errado; com 0, o código morreu (o link continua valendo).
    Wrong { attempts_left: u32 },
    /// Sem cadastro esperando, ou o código já morreu.
    Expired,
}

fn signup_key(redis: &Redis, email: &str) -> String {
    redis.key(format_args!("signup:{email}"))
}

fn token_key(redis: &Redis, hash: &str) -> String {
    redis.key(format_args!("signup_token:{hash}"))
}

fn username_reservation(redis: &Redis, username: &str) -> String {
    redis.key(format_args!("signup_username:{}", username_key(username)))
}

/// Hash do código, com o hash do token do mesmo cadastro como sal. Com um
/// milhão de códigos possíveis, o hash não esconde o código de quem lê o
/// Redis; só o tira do texto puro. O que segura são as tentativas.
fn code_hash(link_hash: &str, code: &str) -> String {
    token::hex(&token::hash_token(&format!("{link_hash}:{code}")))
}

/// Reserva `username` para o cadastro de `email`. Devolve o email do cadastro
/// que já o reservou, se for outro; pedir de novo com o mesmo email vale (a
/// reserva é dele).
pub async fn reserve_username(
    redis: &Redis,
    username: &str,
    email: &str,
    ttl_secs: u64,
) -> RedisResult<Option<String>> {
    let reservation = username_reservation(redis, username);
    let reserved: Option<String> = redis
        .query(
            redis::cmd("SET")
                .arg(&reservation)
                .arg(email)
                .arg("NX")
                .arg("EX")
                .arg(ttl_secs),
        )
        .await?;
    if reserved.is_some() {
        return Ok(None);
    }
    let holder: Option<String> = redis.query(&Cmd::get(&reservation)).await?;
    Ok(holder.filter(|holder| holder != email))
}

/// Hash da senha do cadastro de `email` que espera a confirmação.
pub async fn pending_password_hash(redis: &Redis, email: &str) -> RedisResult<Option<String>> {
    redis
        .query(&Cmd::hget(signup_key(redis, email), "password_hash"))
        .await
}

/// Desfaz o cadastro de `email` que espera a confirmação: o link e o código
/// param de valer e o username dele fica livre.
pub async fn drop_signup(redis: &Redis, email: &str) -> RedisResult<()> {
    let key = signup_key(redis, email);
    let (username, token): (Option<String>, Option<String>) = redis
        .query(redis::cmd("HMGET").arg(&key).arg("username").arg("token"))
        .await?;
    let mut pipe = redis::pipe();
    pipe.atomic().del(&key).ignore();
    if let Some(token) = token {
        pipe.del(token_key(redis, &token)).ignore();
    }
    if let Some(username) = username {
        pipe.del(username_reservation(redis, &username)).ignore();
    }
    redis.query_pipe(&pipe).await
}

/// Guarda o cadastro, com o username já reservado (`reserve_username`), no
/// lugar do anterior do mesmo email, e devolve o token do link e o código.
pub async fn store_signup(
    redis: &Redis,
    pending: &PendingSignup,
    ttl_secs: u64,
) -> RedisResult<SignupSecrets> {
    let reservation = username_reservation(redis, &pending.username);
    let key = signup_key(redis, &pending.email);
    let (old_username, old_token): (Option<String>, Option<String>) = redis
        .query(redis::cmd("HMGET").arg(&key).arg("username").arg("token"))
        .await?;

    let link = token::random_token();
    let code = token::random_code();
    let hash = token::hex(&token::hash_token(&link));
    let mut pipe = redis::pipe();
    pipe.atomic();
    if let Some(old) = old_token {
        pipe.del(token_key(redis, &old)).ignore();
    }
    if let Some(old) = old_username.filter(|u| username_key(u) != username_key(&pending.username)) {
        pipe.del(username_reservation(redis, &old)).ignore();
    }
    pipe.del(&key)
        .ignore()
        .hset_multiple(
            &key,
            &[
                ("username", pending.username.as_str()),
                ("password_hash", pending.password_hash.as_str()),
                ("locale", pending.locale.as_str()),
                ("token", hash.as_str()),
                ("code", code_hash(&hash, &code).as_str()),
            ],
        )
        .ignore()
        .expire(&key, ttl_secs as i64)
        .ignore()
        .set_ex(token_key(redis, &hash), &pending.email, ttl_secs)
        .ignore()
        .set_ex(&reservation, &pending.email, ttl_secs)
        .ignore();
    redis.query_pipe::<()>(&pipe).await?;
    Ok(SignupSecrets { link, code })
}

/// O cadastro deste link, que sai do Redis (o link não vale de novo); `None`
/// se o link expirou, já foi usado ou foi trocado por um mais novo.
pub async fn take_signup(redis: &Redis, link: &str) -> RedisResult<Option<PendingSignup>> {
    take_by_hash(redis, &token::hex(&token::hash_token(link))).await
}

/// Confere o código digitado para o cadastro de `email`. Cada conferência
/// conta antes de comparar (num `HINCRBY`, que pedidos ao mesmo tempo não
/// leem iguais), e a de número `max_attempts` errada mata o código: o mesmo
/// cadastro nunca compara mais que `max_attempts` códigos.
pub async fn take_signup_by_code(
    redis: &Redis,
    email: &str,
    code: &str,
    max_attempts: u32,
) -> RedisResult<CodeCheck> {
    let key = signup_key(redis, email);
    let (stored, link_hash): (Option<String>, Option<String>) = redis
        .query(redis::cmd("HMGET").arg(&key).arg("code").arg("token"))
        .await?;
    let (Some(stored), Some(link_hash)) = (stored, link_hash) else {
        return Ok(CodeCheck::Expired);
    };
    let (attempt, ttl): (u32, i64) = redis
        .query_pipe(
            redis::pipe()
                .atomic()
                .hincr(&key, "code_attempts", 1)
                .ttl(&key),
        )
        .await?;
    if ttl == -1 {
        // O cadastro expirou entre os dois comandos, e o HINCRBY criou uma
        // chave só com a contagem, sem TTL.
        redis.query::<()>(&Cmd::del(&key)).await?;
        return Ok(CodeCheck::Expired);
    }
    if attempt > max_attempts {
        return Ok(CodeCheck::Expired);
    }
    let typed = code_hash(&link_hash, code);
    if token::constant_time_eq(typed.as_bytes(), stored.as_bytes()) {
        return Ok(match take_by_hash(redis, &link_hash).await? {
            Some(pending) => CodeCheck::Confirmed(pending),
            // O link foi aberto no meio tempo.
            None => CodeCheck::Expired,
        });
    }
    if attempt == max_attempts {
        redis.query::<()>(&Cmd::hdel(&key, "code")).await?;
    }
    Ok(CodeCheck::Wrong {
        attempts_left: max_attempts - attempt,
    })
}

/// O cadastro do link de hash `hash`, que sai do Redis.
async fn take_by_hash(redis: &Redis, hash: &str) -> RedisResult<Option<PendingSignup>> {
    let email: Option<String> = redis.query(&Cmd::get_del(token_key(redis, hash))).await?;
    let Some(email) = email else {
        return Ok(None);
    };
    let key = signup_key(redis, &email);
    let (mut fields,): (HashMap<String, String>,) = redis
        .query_pipe(redis::pipe().atomic().hgetall(&key).del(&key).ignore())
        .await?;
    if fields.get("token").map(String::as_str) != Some(hash) {
        return Ok(None);
    }
    let (Some(username), Some(password_hash)) =
        (fields.remove("username"), fields.remove("password_hash"))
    else {
        return Ok(None);
    };
    let locale = match fields.get("locale").map(String::as_str) {
        Some("es") => Locale::Es,
        _ => Locale::Pt,
    };
    redis
        .query::<()>(&Cmd::del(username_reservation(redis, &username)))
        .await?;
    Ok(Some(PendingSignup {
        username,
        email,
        password_hash,
        locale,
    }))
}

/// Um cadastro ainda sem confirmação reservou este username.
pub async fn username_reserved(redis: &Redis, username: &str) -> RedisResult<bool> {
    redis
        .query(&Cmd::exists(username_reservation(redis, username)))
        .await
}
