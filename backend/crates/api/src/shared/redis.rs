//! Conexão com o Redis e o prefixo que separa as chaves de cada ambiente (e de
//! cada teste) no mesmo servidor. O que cada domínio guarda nele fica no `kv/`
//! do domínio. No Upstash grátis cada comando conta na cota do mês: nada de
//! comando no caminho de toda requisição sem necessidade.

use std::{fmt::Display, sync::Arc, time::Duration};

use redis::{
    Client, Cmd, FromRedisValue, Pipeline, RedisError, RedisResult,
    aio::{ConnectionManager, ConnectionManagerConfig},
};

/// Clonar é barato: os clones dividem a mesma conexão multiplexada.
#[derive(Clone)]
pub struct Redis {
    conn: ConnectionManager,
    prefix: Arc<str>,
}

impl Redis {
    /// Prepara a conexão sem abrir: ela abre no primeiro comando e reabre
    /// sozinha se cair. Precisa rodar dentro do runtime do tokio.
    pub fn open(url: &str, prefix: &str) -> RedisResult<Self> {
        let config = ConnectionManagerConfig::new()
            .set_connection_timeout(Some(Duration::from_secs(3)))
            .set_response_timeout(Some(Duration::from_secs(3)))
            .set_number_of_retries(2);
        Ok(Self {
            conn: ConnectionManager::new_lazy_with_config(Client::open(url)?, config)?,
            prefix: prefix.into(),
        })
    }

    /// Confere a conexão (a API faz uma vez, ao subir).
    pub async fn ping(&self) -> RedisResult<()> {
        self.query::<String>(&redis::cmd("PING")).await.map(drop)
    }

    /// `key` com o prefixo do ambiente.
    pub fn key(&self, key: impl Display) -> String {
        format!("{}{key}", self.prefix)
    }

    pub async fn query<T: FromRedisValue>(&self, cmd: &Cmd) -> RedisResult<T> {
        let mut conn = self.conn.clone();
        match cmd.query_async(&mut conn).await {
            Err(err) if dropped(&err) => cmd.query_async(&mut conn).await,
            result => result,
        }
    }

    pub async fn query_pipe<T: FromRedisValue>(&self, pipe: &Pipeline) -> RedisResult<T> {
        let mut conn = self.conn.clone();
        match pipe.query_async(&mut conn).await {
            Err(err) if dropped(&err) => pipe.query_async(&mut conn).await,
            result => result,
        }
    }
}

/// Conexão que caiu enquanto estava parada: o comando falha, o
/// `ConnectionManager` reconecta, e vale tentar de novo uma vez.
fn dropped(err: &RedisError) -> bool {
    err.is_connection_dropped() || err.is_io_error()
}
