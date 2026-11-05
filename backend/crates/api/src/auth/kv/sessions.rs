//! Sessões no Redis, só para sobreviverem a um boot: quem responde às requisições
//! é a memória (`session_store.rs`). `session:{hash}` é um JSON com o dono, a
//! criação (em milissegundos) e o `user_agent`, e expira sozinho com a sessão. Só
//! o hash do token chega ao Redis. Ler tudo custa um SCAN e um MGET por página de
//! chaves, seja quantas forem as sessões.

use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc, serde::ts_milliseconds};
use redis::{Cmd, RedisResult};
use serde::{Deserialize, Serialize};

use crate::{auth::session::SESSION_DAYS, shared::Redis};

/// Chaves por página do SCAN. O padrão, 10, é uma ida e volta a cada 10 chaves.
const SCAN_PAGE: usize = 1000;

/// Uma sessão, como fica na memória e, sem o hash (que é a chave), no Redis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredSession {
    /// Hash do token em hexadecimal.
    #[serde(skip)]
    pub hash: String,
    pub user_id: i64,
    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user_agent: Option<String>,
}

impl StoredSession {
    /// Toda sessão dura o mesmo, contado da criação.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.created_at + TimeDelta::days(SESSION_DAYS)
    }
}

fn session_key(redis: &Redis, hash: &str) -> String {
    redis.key(format_args!("session:{hash}"))
}

pub async fn save_session(redis: &Redis, session: &StoredSession) -> RedisResult<()> {
    let json = serde_json::to_string(session).expect("sessão vira JSON");
    let ttl = SESSION_DAYS as u64 * 24 * 60 * 60;
    redis
        .query(&Cmd::set_ex(session_key(redis, &session.hash), json, ttl))
        .await
}

/// Apaga as sessões com estes hashes (em hexadecimal).
pub async fn delete_sessions(redis: &Redis, hashes: &[String]) -> RedisResult<()> {
    if hashes.is_empty() {
        return Ok(());
    }
    let keys: Vec<String> = hashes.iter().map(|h| session_key(redis, h)).collect();
    redis.query(&Cmd::del(keys)).await
}

/// Todas as sessões que ainda não expiraram. O SCAN pode repetir uma chave.
pub async fn load_sessions(redis: &Redis) -> RedisResult<Vec<StoredSession>> {
    let pattern = session_key(redis, "*");
    let mut sessions = Vec::new();
    let mut cursor = 0u64;
    loop {
        let (next, keys): (u64, Vec<String>) = redis
            .query(
                redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&pattern)
                    .arg("COUNT")
                    .arg(SCAN_PAGE),
            )
            .await?;
        if !keys.is_empty() {
            sessions.extend(read_sessions(redis, &keys).await?);
        }
        if next == 0 {
            return Ok(sessions);
        }
        cursor = next;
    }
}

/// A sessão com este hash (em hexadecimal), se existe.
pub async fn load_session(redis: &Redis, hash: &str) -> RedisResult<Option<StoredSession>> {
    let found = read_sessions(redis, &[session_key(redis, hash)]).await?;
    Ok(found.into_iter().next())
}

async fn read_sessions(redis: &Redis, keys: &[String]) -> RedisResult<Vec<StoredSession>> {
    let prefix = session_key(redis, "");
    let hash_of = |key: &str| key.strip_prefix(&prefix).unwrap_or(key).to_owned();
    let values: Vec<Option<String>> = redis.query(redis::cmd("MGET").arg(keys)).await?;

    let mut sessions = Vec::new();
    let mut old_format = Vec::new();
    for (key, value) in keys.iter().zip(values) {
        // O MGET lê como nada o que não é texto.
        let Some(json) = value else {
            old_format.push(key);
            continue;
        };
        match serde_json::from_str::<StoredSession>(&json) {
            Ok(session) => sessions.push(StoredSession {
                hash: hash_of(key),
                ..session
            }),
            Err(err) => tracing::warn!(error = %err, "sessão ilegível no Redis, ignorada"),
        }
    }
    if !old_format.is_empty() {
        let mut pipe = redis::pipe();
        for key in &old_format {
            pipe.hgetall(*key);
        }
        let fields: Vec<HashMap<String, String>> = redis.query_pipe(&pipe).await?;
        sessions.extend(
            old_format
                .into_iter()
                .zip(fields)
                .filter_map(|(key, fields)| from_old_format(hash_of(key), fields)),
        );
    }
    Ok(sessions)
}

/// A sessão gravada como hash do Redis, o formato até novembro de 2026, que
/// expira sozinho em 30 dias.
fn from_old_format(hash: String, mut fields: HashMap<String, String>) -> Option<StoredSession> {
    Some(StoredSession {
        hash,
        user_id: fields.get("user_id")?.parse().ok()?,
        created_at: DateTime::from_timestamp_millis(fields.get("created_at")?.parse().ok()?)?,
        user_agent: fields.remove("user_agent"),
    })
}
