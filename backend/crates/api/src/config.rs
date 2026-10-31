use std::net::SocketAddr;

use anyhow::Context;

use crate::shared::StorageConfig;

pub struct Config {
    pub database_url: String,
    /// Redis das sessões e do limite de tentativas (Upstash: `rediss://...`).
    pub redis_url: String,
    /// Prefixo das chaves, para mais de um ambiente dividir o mesmo Redis.
    pub redis_prefix: String,
    pub addr: SocketAddr,
    /// Origem do frontend, liberada no CORS. Os provedores de login voltam
    /// para ela (`/auth/{provedor}/callback`).
    pub web_origin: String,
    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
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
    /// Endereço na forma canônica, sem descartáveis (`EMAIL_STRICT`, ligado por
    /// padrão). Desligado em dev e staging, as variações do mesmo endereço
    /// (`+etiqueta`, pontos no Gmail) abrem contas diferentes.
    pub email_strict: bool,
    /// Bucket privado, o dos screenshots do placar (`S3_*`); `None` sem bucket
    /// nem chaves: os screenshots ficam desligados.
    pub storage: Option<StorageConfig>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL não definida")?,
            redis_url: std::env::var("REDIS_URL").context("REDIS_URL não definida")?,
            redis_prefix: env_or("REDIS_PREFIX", "telefrag:"),
            addr: env_or("API_ADDR", "127.0.0.1:4000")
                .parse()
                .context("API_ADDR inválido")?,
            web_origin: env_or("WEB_ORIGIN", "http://localhost:4001"),
            discord_client_id: env_opt("DISCORD_CLIENT_ID"),
            discord_client_secret: env_opt("DISCORD_CLIENT_SECRET"),
            google_client_id: env_opt("GOOGLE_CLIENT_ID"),
            google_client_secret: env_opt("GOOGLE_CLIENT_SECRET"),
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
            email_strict: env_flag("EMAIL_STRICT", true)?,
            storage: storage_from_env()?,
        })
    }
}

/// O bucket privado, se `S3_PRIVATE_BUCKET`, `S3_ACCESS_KEY_ID` e
/// `S3_SECRET_ACCESS_KEY` existem. Só parte delas é engano de configuração: a
/// API não sobe.
fn storage_from_env() -> anyhow::Result<Option<StorageConfig>> {
    let bucket = env_opt("S3_PRIVATE_BUCKET");
    let key = env_opt("S3_ACCESS_KEY_ID");
    let secret = env_opt("S3_SECRET_ACCESS_KEY");
    match (bucket, key, secret) {
        (None, None, None) => Ok(None),
        (Some(bucket), Some(access_key_id), Some(secret_access_key)) => Ok(Some(StorageConfig {
            endpoint: env_opt("S3_ENDPOINT"),
            region: env_or("S3_REGION", "us-east-1"),
            bucket,
            access_key_id,
            secret_access_key,
        })),
        _ => anyhow::bail!(
            "S3_PRIVATE_BUCKET, S3_ACCESS_KEY_ID e S3_SECRET_ACCESS_KEY vão juntas (ou nenhuma, sem screenshots)"
        ),
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

/// Liga ou desliga: `true`/`1` ou `false`/`0` (sem diferenciar maiúsculas);
/// ausente ou vazia, `default`. Outro valor é engano: a API não sobe.
fn env_flag(key: &str, default: bool) -> anyhow::Result<bool> {
    match env_opt(key).map(|v| v.to_ascii_lowercase()).as_deref() {
        None => Ok(default),
        Some("1" | "true") => Ok(true),
        Some("0" | "false") => Ok(false),
        Some(other) => anyhow::bail!("{key} inválida: {other} (use true ou false)"),
    }
}

/// Variável opcional; vazia conta como ausente.
fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}
