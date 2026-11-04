//! Limites de envio, no Redis. Duas esperas por endereço, uma por fila
//! (`Lane`): `email_cooldown:{endereço}` e `email_quiet_cooldown:{endereço}`
//! existem por um tempo depois de cada envio. `email_sent:{AAAA-MM-DD}` conta os
//! envios do dia (UTC) e expira dois dias depois. Juntos, seguram quem pede
//! email sem parar para o mesmo endereço e o teto do plano do provedor. Os
//! avisos (`reserve_today`) só contam no dia. O envio que falha no provedor
//! devolve a reserva (`release`): a espera sai e o dia volta um.
//!
//! Os pedidos de quem chama contam também por período (`count_requests`):
//! `email_ip:{ip}:{período}` e `email_account:{conta}:{período}`, na hora
//! (`AAAA-MM-DDTHH`) e no dia (`AAAA-MM-DD`, UTC), cada um com o TTL dele.

use chrono::{DateTime, Timelike, Utc};
use redis::{Cmd, RedisResult};

use crate::shared::Redis;

/// Dois dias: o contador do dia vive além da virada em qualquer fuso.
const DAY_KEY_TTL_SECS: i64 = 2 * 24 * 60 * 60;

/// Qual espera entre envios um email usa.
#[derive(Clone, Copy)]
pub enum Lane {
    /// O que responde `email.too_soon`: o cadastro e a troca de email, com o
    /// link e com o aviso de que o endereço já tem dono. Os dois ramos marcam a
    /// mesma espera, para a resposta não dizer qual foi.
    Requests,
    /// O que nenhuma resposta mostra: entrar por email e o que só vai à própria
    /// conta. Fica à parte para nada aqui aparecer num `email.too_soon` (só conta
    /// existente envia) e para os avisos da outra fila não segurarem o email que
    /// entra.
    Quiet,
}

pub enum Slot {
    /// Reservado no dia `day` (`AAAA-MM-DD`, UTC), o que `release` devolve.
    Granted { day: String },
    /// O último envio para o endereço foi há pouco; o próximo, em `seconds`.
    TooSoon { seconds: i64 },
    /// O teto do dia acabou.
    DailyLimit,
}

/// Reserva um envio para `to`: uma espera de `cooldown_secs` para o endereço
/// nesta fila e um a menos no teto do dia (`daily_limit`).
pub async fn reserve(
    redis: &Redis,
    to: &str,
    lane: Lane,
    cooldown_secs: u64,
    daily_limit: u32,
) -> RedisResult<Slot> {
    let cooldown = cooldown_key(redis, lane, to);
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
/// o envio a marcou, nesta fila) e um no contador do dia `day`.
pub async fn release(redis: &Redis, to: Option<(Lane, &str)>, day: &str) -> RedisResult<()> {
    let mut pipe = redis::pipe();
    if let Some((lane, to)) = to {
        pipe.del(cooldown_key(redis, lane, to)).ignore();
    }
    pipe.decr(day_key(redis, day), 1).ignore();
    redis.query_pipe(&pipe).await
}

/// Pedidos de um IP ou de uma conta: dentro dos limites ou, fora, quantos
/// segundos até o período que estourou acabar.
pub enum RequestSlot {
    Granted,
    Limited { seconds: i64 },
}

/// Conta mais um pedido de `scope` (`email_ip:{ip}`, `email_account:{conta}`) na
/// hora e no dia, e diz se ele passou de `hourly` na hora ou de `daily` no dia.
/// Dois comandos por pedido; o primeiro pedido de cada período faz mais um, que
/// põe o TTL.
pub async fn count_requests(
    redis: &Redis,
    scope: &str,
    hourly: u32,
    daily: u32,
    now: DateTime<Utc>,
) -> RedisResult<RequestSlot> {
    let hour = redis.key(format_args!("{scope}:{}", now.format("%Y-%m-%dT%H")));
    let day = redis.key(format_args!("{scope}:{}", now.format("%Y-%m-%d")));
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
        RequestSlot::Limited {
            seconds: 24 * 60 * 60 - into_day,
        }
    } else if in_hour > hourly {
        RequestSlot::Limited {
            seconds: 60 * 60 - into_hour,
        }
    } else {
        RequestSlot::Granted
    })
}

fn cooldown_key(redis: &Redis, lane: Lane, to: &str) -> String {
    match lane {
        Lane::Requests => redis.key(format_args!("email_cooldown:{to}")),
        Lane::Quiet => redis.key(format_args!("email_quiet_cooldown:{to}")),
    }
}

fn day_key(redis: &Redis, day: &str) -> String {
    redis.key(format_args!("email_sent:{day}"))
}
