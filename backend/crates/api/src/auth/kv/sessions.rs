//! Sessões. `session:{hash}` é um hash com o dono (`user_id`), a criação (em
//! milissegundos) e o `user_agent`, e expira sozinho com a sessão.
//! `user_sessions:{user_id}` é o índice das sessões de cada conta (zset com a
//! expiração, em segundos, como score), para listar e encerrar as outras. Só o
//! hash do token chega ao Redis.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use redis::{Cmd, RedisResult};

use crate::{
    auth::session::SESSION_DAYS,
    shared::{Redis, token},
};

/// Uma sessão viva, como a lista mostra.
pub struct StoredSession {
    /// Hash do token em hexadecimal.
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
}

fn session_key(redis: &Redis, hash: &str) -> String {
    redis.key(format_args!("session:{hash}"))
}

fn index_key(redis: &Redis, user_id: i64) -> String {
    redis.key(format_args!("user_sessions:{user_id}"))
}

/// Cria a sessão e devolve o token. Sessão e índice entram juntos (MULTI):
/// sessão fora do índice escaparia de "encerrar as outras".
pub async fn create_session(
    redis: &Redis,
    user_id: i64,
    user_agent: Option<&str>,
) -> RedisResult<String> {
    let token = token::random_secret();
    let hash = token::hex(&token::hash_token(&token));
    let now = Utc::now();
    let expires_at = (now + chrono::Duration::days(SESSION_DAYS)).timestamp();

    let mut fields = vec![
        ("user_id", user_id.to_string()),
        ("created_at", now.timestamp_millis().to_string()),
    ];
    if let Some(agent) = user_agent {
        fields.push(("user_agent", agent.to_owned()));
    }
    let key = session_key(redis, &hash);
    let index = index_key(redis, user_id);
    redis
        .query_pipe::<()>(
            redis::pipe()
                .atomic()
                .hset_multiple(&key, &fields)
                .ignore()
                .expire_at(&key, expires_at)
                .ignore()
                .zadd(&index, &hash, expires_at)
                .ignore()
                .zrembyscore(&index, "-inf", now.timestamp())
                .ignore()
                // Toda sessão dura o mesmo: a nova é a última a expirar.
                .expire_at(&index, expires_at)
                .ignore(),
        )
        .await?;
    Ok(token)
}

/// Dono da sessão com este hash, se ela não expirou. É o único comando no
/// caminho de cada requisição logada.
pub async fn session_user(redis: &Redis, hash: &[u8; 32]) -> RedisResult<Option<i64>> {
    redis
        .query(&Cmd::hget(session_key(redis, &token::hex(hash)), "user_id"))
        .await
}

/// Sessões vivas da conta, da mais nova para a mais antiga. As vencidas saem
/// do índice.
pub async fn user_sessions(redis: &Redis, user_id: i64) -> RedisResult<Vec<StoredSession>> {
    let index = index_key(redis, user_id);
    let (entries,): (Vec<(String, f64)>,) = redis
        .query_pipe(
            redis::pipe()
                .zrembyscore(&index, "-inf", Utc::now().timestamp())
                .ignore()
                .zrange_withscores(&index, 0, -1),
        )
        .await?;
    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let mut details = redis::pipe();
    for (hash, _) in &entries {
        details.hgetall(session_key(redis, hash));
    }
    let details: Vec<HashMap<String, String>> = redis.query_pipe(&details).await?;

    let mut sessions: Vec<StoredSession> = entries
        .into_iter()
        .zip(details)
        .filter_map(|((hash, expires_at), mut fields)| {
            Some(StoredSession {
                hash,
                created_at: DateTime::from_timestamp_millis(
                    fields.get("created_at")?.parse().ok()?,
                )?,
                expires_at: DateTime::from_timestamp(expires_at as i64, 0)?,
                user_agent: fields.remove("user_agent"),
            })
        })
        .collect();
    sessions.sort_by_key(|s| std::cmp::Reverse(s.created_at));
    Ok(sessions)
}

/// Encerra a sessão com este hash e devolve o dono; sem sessão, nada
/// acontece.
pub async fn delete_session(redis: &Redis, hash: &[u8; 32]) -> RedisResult<Option<i64>> {
    let user_id = session_user(redis, hash).await?;
    if let Some(user_id) = user_id {
        delete_sessions(redis, user_id, &[token::hex(hash)]).await?;
    }
    Ok(user_id)
}

/// Hashes (em hexadecimal) das sessões no índice da conta.
pub async fn session_hashes(redis: &Redis, user_id: i64) -> RedisResult<Vec<String>> {
    redis
        .query(&Cmd::zrange(index_key(redis, user_id), 0, -1))
        .await
}

/// Encerra todas as sessões da conta e devolve os hashes (em hexadecimal).
pub async fn delete_all_sessions(redis: &Redis, user_id: i64) -> RedisResult<Vec<String>> {
    let all = session_hashes(redis, user_id).await?;
    delete_sessions(redis, user_id, &all).await?;
    Ok(all)
}

/// Encerra as sessões da conta, menos a de hash `keep`, e devolve os hashes
/// (em hexadecimal) das que encerrou.
pub async fn delete_other_sessions(
    redis: &Redis,
    user_id: i64,
    keep: &[u8; 32],
) -> RedisResult<Vec<String>> {
    let keep = token::hex(keep);
    let others: Vec<String> = session_hashes(redis, user_id)
        .await?
        .into_iter()
        .filter(|h| *h != keep)
        .collect();
    delete_sessions(redis, user_id, &others).await?;
    Ok(others)
}

/// Encerra as sessões da conta com estes hashes (em hexadecimal).
pub async fn delete_sessions(redis: &Redis, user_id: i64, hashes: &[String]) -> RedisResult<()> {
    if hashes.is_empty() {
        return Ok(());
    }
    let mut pipe = redis::pipe();
    pipe.atomic();
    for hash in hashes {
        pipe.del(session_key(redis, hash)).ignore();
    }
    pipe.zrem(index_key(redis, user_id), hashes).ignore();
    redis.query_pipe(&pipe).await
}
