//! Os bans no Redis, só para sobreviverem a um boot: quem responde é a memória
//! (`bans.rs`). `scanner_bans` é um sorted set de IPs (ou blocos), com o fim do ban
//! em segundos Unix como score, e expira inteiro quando nenhum ban novo chega. Um
//! set só, e não uma chave por IP: ler tudo ao subir custa dois comandos, seja
//! quantos forem, e os novos vão juntos num ZADD.

use redis::{Cmd, RedisResult, pipe};

use crate::shared::Redis;

fn key(redis: &Redis) -> String {
    redis.key("scanner_bans")
}

/// Grava `(fim do ban, IP)`.
pub async fn save(redis: &Redis, bans: &[(i64, String)], ban_seconds: i64) -> RedisResult<()> {
    let key = key(redis);
    let mut pipeline = pipe();
    pipeline
        .add_command(Cmd::zadd_multiple(&key, bans))
        .ignore()
        .add_command(Cmd::expire(&key, ban_seconds))
        .ignore();
    redis.query_pipe(&pipeline).await
}

/// Apaga os vencidos e devolve os outros: `(IP, fim do ban)`.
pub async fn load(redis: &Redis, now: i64) -> RedisResult<Vec<(String, i64)>> {
    let key = key(redis);
    let _: i64 = redis.query(&Cmd::zrembyscore(&key, "-inf", now)).await?;
    let saved: Vec<(String, f64)> = redis
        .query(&Cmd::zrangebyscore_withscores(&key, "-inf", "+inf"))
        .await?;
    Ok(saved
        .into_iter()
        .map(|(bucket, until)| (bucket, until as i64))
        .collect())
}

pub async fn delete(redis: &Redis, bucket: &str) -> RedisResult<()> {
    let _: i64 = redis.query(&Cmd::zrem(key(redis), bucket)).await?;
    Ok(())
}
