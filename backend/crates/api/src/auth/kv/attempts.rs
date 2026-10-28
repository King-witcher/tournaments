//! Erros de senha recentes, para o limite de tentativas (`attempts.rs`):
//! `password_failures:{username}` é um zset com o horário de cada erro (em
//! milissegundos, como score) e expira junto com a janela.

use chrono::{DateTime, TimeDelta, Utc};
use redis::{Cmd, RedisResult};

use crate::shared::{Redis, token};

fn failures_key(redis: &Redis, username: &str) -> String {
    redis.key(format_args!("password_failures:{username}"))
}

/// Até quando `username` está bloqueado: com `max` erros na janela, até o
/// mais antigo deles sair dela.
pub async fn password_blocked_until(
    redis: &Redis,
    username: &str,
    max: usize,
    window: TimeDelta,
) -> RedisResult<Option<DateTime<Utc>>> {
    let since = Utc::now().timestamp_millis() - window.num_milliseconds();
    let recent: Vec<(String, f64)> = redis
        .query(&Cmd::zrevrangebyscore_limit_withscores(
            failures_key(redis, username),
            "+inf",
            format!("({since}"),
            0,
            max as isize,
        ))
        .await?;
    Ok(match recent.last() {
        Some((_, oldest)) if recent.len() >= max => {
            DateTime::from_timestamp_millis(*oldest as i64).map(|at| at + window)
        }
        _ => None,
    })
}

/// Registra um erro de senha e esquece os que saíram da janela.
pub async fn record_password_failure(
    redis: &Redis,
    username: &str,
    window: TimeDelta,
) -> RedisResult<()> {
    let key = failures_key(redis, username);
    let now = Utc::now().timestamp_millis();
    // O membro do zset é único: dois erros no mesmo milissegundo contam dois.
    let failure = format!("{now}-{}", token::random_token());
    redis
        .query_pipe(
            redis::pipe()
                .atomic()
                .zadd(&key, failure, now)
                .ignore()
                .zrembyscore(&key, "-inf", now - window.num_milliseconds())
                .ignore()
                .pexpire(&key, window.num_milliseconds())
                .ignore(),
        )
        .await
}

pub async fn clear_password_failures(redis: &Redis, username: &str) -> RedisResult<()> {
    redis
        .query(&Cmd::del(failures_key(redis, username)))
        .await
        .map(|_: i64| ())
}
