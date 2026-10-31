//! Tentativas de senha recentes, para o limite (`attempts.rs`):
//! `password_failures:{email}` (na forma canônica) é um zset com o horário de
//! cada tentativa (em milissegundos, como score) e expira junto com a janela.
//! A tentativa entra antes da conferência e fica como erro; o acerto apaga o
//! zset.

use chrono::{DateTime, TimeDelta, Utc};
use redis::{Cmd, RedisResult};

use crate::shared::{Redis, token};

fn failures_key(redis: &Redis, email: &str) -> String {
    redis.key(format_args!("password_failures:{email}"))
}

/// Conta uma tentativa de senha de `email`, antes de conferi-la, e esquece
/// as que saíram da janela. Com `max` erros na janela, a tentativa não conta,
/// e a resposta é até quando `email` está bloqueado: até o mais antigo dos
/// `max` últimos sair dela. Contar antes de conferir, no mesmo MULTI que lê
/// os erros, é o que segura as tentativas ao mesmo tempo: cada uma vê as
/// outras, então não passam de `max` conferências na janela.
pub async fn begin_password_attempt(
    redis: &Redis,
    email: &str,
    max: usize,
    window: TimeDelta,
) -> RedisResult<Option<DateTime<Utc>>> {
    let key = failures_key(redis, email);
    let now = Utc::now().timestamp_millis();
    // O membro do zset é único: duas tentativas no mesmo milissegundo contam duas.
    let attempt = format!("{now}-{}", token::random_token());
    let (recent,): (Vec<(String, f64)>,) = redis
        .query_pipe(
            redis::pipe()
                .atomic()
                .zrembyscore(&key, "-inf", now - window.num_milliseconds())
                .ignore()
                .zrevrange_withscores(&key, 0, max as isize - 1)
                .zadd(&key, &attempt, now)
                .ignore()
                .pexpire(&key, window.num_milliseconds())
                .ignore(),
        )
        .await?;
    let Some((_, oldest)) = recent.last().filter(|_| recent.len() >= max) else {
        return Ok(None);
    };
    redis.query::<()>(&Cmd::zrem(&key, &attempt)).await?;
    Ok(DateTime::from_timestamp_millis(*oldest as i64).map(|at| at + window))
}

/// Zera a contagem, inclusive a tentativa que acabou de acertar.
pub async fn clear_password_failures(redis: &Redis, email: &str) -> RedisResult<()> {
    redis
        .query(&Cmd::del(failures_key(redis, email)))
        .await
        .map(|_: i64| ())
}
