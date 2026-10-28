use std::net::SocketAddr;

use anyhow::Context;

pub struct Config {
    pub database_url: String,
    pub addr: SocketAddr,
    /// Origem do frontend, liberada no CORS.
    pub web_origin: String,
    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<String>,
    pub discord_redirect_uri: String,
    /// Domínio do cookie de sessão (em produção, `.dominio`).
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL não definida")?,
            addr: env_or("API_ADDR", "127.0.0.1:4000")
                .parse()
                .context("API_ADDR inválido")?,
            web_origin: env_or("WEB_ORIGIN", "http://localhost:4001"),
            discord_client_id: env_opt("DISCORD_CLIENT_ID"),
            discord_client_secret: env_opt("DISCORD_CLIENT_SECRET"),
            discord_redirect_uri: env_or(
                "DISCORD_REDIRECT_URI",
                "http://localhost:4000/auth/discord/callback",
            ),
            cookie_domain: env_opt("COOKIE_DOMAIN"),
            cookie_secure: matches!(
                env_or("COOKIE_SECURE", "false")
                    .to_ascii_lowercase()
                    .as_str(),
                "1" | "true"
            ),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

/// Variável opcional; vazia conta como ausente.
fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}
