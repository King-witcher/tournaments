//! Limites de envio, no Redis: `email_cooldown:{endereço}` existe por um tempo
//! depois de cada envio para o endereço, e `email_sent:{AAAA-MM-DD}` conta os
//! envios do dia (UTC) e expira dois dias depois. Juntos, seguram quem pede
//! email sem parar para o mesmo endereço e o teto do plano do provedor. Os
//! avisos (`reserve_today`) só contam no dia. O envio que falha no provedor
//! devolve a reserva (`release`): a espera sai e o dia volta um.
//!
//! Os pedidos sem login que mandam email contam também por IP (`count_ip`):
//! `email_ip:{ip}:{AAAA-MM-DDTHH}` na hora e `email_ip:{ip}:{AAAA-MM-DD}` no
//! dia (UTC), cada um com o TTL do seu período.

use chrono::{DateTime, Timelike, Utc};
use redis::{Cmd, RedisResult};

use crate::shared::Redis;

/// Dois dias: o contador do dia vive além da virada em qualquer fuso.
const DAY_KEY_TTL_SECS: i64 = 2 * 24 * 60 * 60;

pub enum Slot {
    /// Reservado no dia `day` (`AAAA-MM-DD`, UTC), o que `release` devolve.
    Granted { day: String },
    /// O último envio para o endereço foi há pouco; o próximo, em `seconds`.
    TooSoon { seconds: i64 },
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
    let cooldown = cooldown_key(redis, to);
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
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let day = day_key(redis, &date);
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
        Slot::Granted { day: date }
    })
}

/// Devolve a reserva de um envio que não saiu: a espera do endereço `to` (se
/// o envio a marcou) e um no contador do dia `day`.
pub async fn release(redis: &Redis, to: Option<&str>, day: &str) -> RedisResult<()> {
    let mut pipe = redis::pipe();
    if let Some(to) = to {
        pipe.del(cooldown_key(redis, to)).ignore();
    }
    pipe.decr(day_key(redis, day), 1).ignore();
    redis.query_pipe(&pipe).await
}

/// Pedidos de um IP: dentro dos limites ou, fora, quantos segundos até o
/// período que estourou acabar.
pub enum IpSlot {
    Granted,
    Limited { seconds: i64 },
}

/// Conta mais um pedido do IP `bucket` na hora e no dia, e diz se ele passou
/// de `hourly` na hora ou de `daily` no dia. Dois comandos por pedido; o
/// primeiro pedido de cada período faz mais um, que põe o TTL.
pub async fn count_ip(
    redis: &Redis,
    bucket: &str,
    hourly: u32,
    daily: u32,
    now: DateTime<Utc>,
) -> RedisResult<IpSlot> {
    let hour = redis.key(format_args!(
        "email_ip:{bucket}:{}",
        now.format("%Y-%m-%dT%H")
    ));
    let day = redis.key(format_args!("email_ip:{bucket}:{}", now.format("%Y-%m-%d")));
    let (in_hour, in_day): (u32, u32) = redis
        .query_pipe(redis::pipe().incr(&hour, 1).incr(&day, 1))
        .await?;
    if in_hour == 1 || in_day == 1 {
        let mut pipe = redis::pipe();
        if in_hour == 1 {
            pipe.expire(&hour, 60 * 60).ignore();
        }
        if in_day == 1 {
            pipe.expire(&day, 24 * 60 * 60).ignore();
        }
        redis.query_pipe::<()>(&pipe).await?;
    }
    let into_hour = i64::from(now.minute() * 60 + now.second());
    let into_day = i64::from(now.num_seconds_from_midnight());
    Ok(if in_day > daily {
        IpSlot::Limited {
            seconds: 24 * 60 * 60 - into_day,
        }
    } else if in_hour > hourly {
        IpSlot::Limited {
            seconds: 60 * 60 - into_hour,
        }
    } else {
        IpSlot::Granted
    })
}

fn cooldown_key(redis: &Redis, to: &str) -> String {
    redis.key(format_args!("email_cooldown:{to}"))
}

fn day_key(redis: &Redis, day: &str) -> String {
    redis.key(format_args!("email_sent:{day}"))
}
