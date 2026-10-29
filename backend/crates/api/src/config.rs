use std::net::SocketAddr;

use anyhow::Context;

pub struct Config {
    pub database_url: String,
    /// Redis das sessões e do limite de tentativas (Upstash: `rediss://...`).
    pub redis_url: String,
    /// Prefixo das chaves, para mais de um ambiente dividir o mesmo Redis.
    pub redis_prefix: String,
    pub addr: SocketAddr,
    /// Origem do frontend, liberada no CORS.
    pub web_origin: String,
    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<String>,
    pub discord_redirect_uri: String,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: String,
    /// Domínio do cookie de sessão (em produção, `.dominio`).
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
    /// Chaves VAPID do Web Push, em base64url (sem as duas, o push fica
    /// desligado).
    pub vapid_public_key: Option<String>,
    pub vapid_private_key: Option<String>,
    /// Contato que vai na assinatura VAPID (`mailto:` ou `https:`); sem ele, a
    /// origem do front.
    pub vapid_subject: Option<String>,
    /// Chave da API do Resend; sem ela, os emails só vão para o log (em
    /// produção, com `COOKIE_SECURE`, a API não sobe).
    pub resend_api_key: Option<String>,
    /// Remetente dos emails, num domínio verificado no Resend
    /// (`Nome <nao-responda@dominio>`).
    pub email_from: Option<String>,
    /// Teto de emails por dia, abaixo do plano do provedor.
    pub email_daily_limit: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL não definida")?,
            redis_url: std::env::var("REDIS_URL").context("REDIS_URL não definida")?,
            redis_prefix: env_or("REDIS_PREFIX", "tournaments:"),
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
            google_client_id: env_opt("GOOGLE_CLIENT_ID"),
            google_client_secret: env_opt("GOOGLE_CLIENT_SECRET"),
            google_redirect_uri: env_or(
                "GOOGLE_REDIRECT_URI",
                "http://localhost:4000/auth/google/callback",
            ),
            cookie_domain: env_opt("COOKIE_DOMAIN"),
            cookie_secure: matches!(
                env_or("COOKIE_SECURE", "false")
                    .to_ascii_lowercase()
                    .as_str(),
                "1" | "true"
            ),
            vapid_public_key: env_opt("VAPID_PUBLIC_KEY"),
            vapid_private_key: env_opt("VAPID_PRIVATE_KEY"),
            vapid_subject: env_opt("VAPID_SUBJECT"),
            resend_api_key: env_opt("RESEND_API_KEY"),
            email_from: env_opt("EMAIL_FROM"),
            email_daily_limit: match env_opt("EMAIL_DAILY_LIMIT") {
                Some(limit) => limit.parse().context("EMAIL_DAILY_LIMIT inválido")?,
                None => crate::mail::DEFAULT_DAILY_LIMIT,
            },
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
