//! Limites de envio, no Redis: `email_cooldown:{endereço}` existe por um tempo
//! depois de cada envio para o endereço, e `email_sent:{AAAA-MM-DD}` conta os
//! envios do dia (UTC) e expira dois dias depois. Juntos, seguram quem pede
//! email sem parar para o mesmo endereço e o teto do plano do provedor. Os
//! avisos (`reserve_today`) só contam no dia.

use chrono::Utc;
use redis::{Cmd, RedisResult};

use crate::shared::Redis;

/// Dois dias: o contador do dia vive além da virada em qualquer fuso.
const DAY_KEY_TTL_SECS: i64 = 2 * 24 * 60 * 60;

pub enum Slot {
    Granted,
    /// O último envio para o endereço foi há pouco; o próximo, em `seconds`.
    TooSoon {
        seconds: i64,
    },
    /// O teto do dia acabou.
    DailyLimit,
}

/// Reserva um envio para `to`: uma espera de `cooldown_secs` para o endereço
/// e um a menos no teto do dia (`daily_limit`).
pub async fn reserve(
    redis: &Redis,
    to: &str,
    cooldown_secs: u64,
    daily_limit: u32,
) -> RedisResult<Slot> {
    let cooldown = redis.key(format_args!("email_cooldown:{to}"));
    let started: Option<String> = redis
        .query(
            redis::cmd("SET")
                .arg(&cooldown)
                .arg(1)
                .arg("NX")
                .arg("EX")
                .arg(cooldown_secs),
        )
        .await?;
    if started.is_none() {
        let ttl: i64 = redis.query(&Cmd::ttl(&cooldown)).await?;
        return Ok(Slot::TooSoon {
            seconds: ttl.max(1),
        });
    }
    reserve_today(redis, daily_limit).await
}

/// Reserva um envio só no teto do dia, sem a espera do endereço.
pub async fn reserve_today(redis: &Redis, daily_limit: u32) -> RedisResult<Slot> {
    let day = redis.key(format_args!("email_sent:{}", Utc::now().format("%Y-%m-%d")));
    let (sent,): (u32,) = redis
        .query_pipe(
            redis::pipe()
                .incr(&day, 1)
                .expire(&day, DAY_KEY_TTL_SECS)
                .ignore(),
        )
        .await?;
    Ok(if sent > daily_limit {
        Slot::DailyLimit
    } else {
        Slot::Granted
    })
}
