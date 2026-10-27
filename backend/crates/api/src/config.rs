use std::net::SocketAddr;

use anyhow::Context;

pub struct Config {
    pub database_url: String,
    pub addr: SocketAddr,
    /// Origem do frontend, liberada no CORS.
    pub web_origin: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL não definida")?,
            addr: env_or("API_ADDR", "127.0.0.1:8080")
                .parse()
                .context("API_ADDR inválido")?,
            web_origin: env_or("WEB_ORIGIN", "http://localhost:3000"),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}
