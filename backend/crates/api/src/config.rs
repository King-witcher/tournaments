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
    /// Nome do cookie de sessão. Outro por ambiente quando um fica dentro do
    /// domínio do outro (o staging em `staging.telefrag.com.br` recebe o
    /// cookie da produção, de `.telefrag.com.br`).
    pub cookie_name: String,
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
    /// Bucket público, o dos logos e das capas, com as mesmas chaves; `None`
    /// sem `S3_PUBLIC_BUCKET`: logos e capas ficam desligados.
    pub public_storage: Option<PublicStorageConfig>,
    /// Identifica o servidor do Next para a API; `None`: o banimento de scanners
    /// fica desligado (`scanners/`).
    pub internal_secret: Option<String>,
    /// Serve `/openapi.json` e `/docs` (`API_DOCS`). Por padrão, só sem
    /// `COOKIE_SECURE`: num ambiente publicado, é o mapa da API para um scanner.
    pub api_docs: bool,
}

/// O bucket público e o endereço do CDN que o serve.
#[derive(Debug, Clone)]
pub struct PublicStorageConfig {
    pub storage: StorageConfig,
    /// `S3_PUBLIC_URL`, sem barra no fim (`https://img.telefrag.com.br`).
    pub url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let (storage, public_storage) = storage_from_env()?;
        let cookie_secure = matches!(
            env_or("COOKIE_SECURE", "false")
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true"
        );
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
            cookie_name: cookie_name_from_env()?,
            cookie_domain: env_opt("COOKIE_DOMAIN"),
            cookie_secure,
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
            storage,
            public_storage,
            internal_secret: env_opt("INTERNAL_API_SECRET"),
            api_docs: env_flag("API_DOCS", !cookie_secure)?,
        })
    }
}

/// `COOKIE_NAME`, `session` por padrão. Só letras, dígitos, `_` e `-`: outro
/// caractere o navegador recusa ou corta, e o login falharia sem aviso.
fn cookie_name_from_env() -> anyhow::Result<String> {
    let name = env_or("COOKIE_NAME", "session");
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    anyhow::ensure!(
        valid,
        "COOKIE_NAME inválido: {name} (use letras, dígitos, _ e -)"
    );
    Ok(name)
}

/// Os buckets: o privado com `S3_PRIVATE_BUCKET`, o público com
/// `S3_PUBLIC_BUCKET` e `S3_PUBLIC_URL`, os dois com `S3_ACCESS_KEY_ID` e
/// `S3_SECRET_ACCESS_KEY`. Bucket sem as chaves, chaves sem bucket, o bucket
/// público sem o endereço (ou o contrário) e endereço que não é URL são
/// engano de configuração: a API não sobe.
fn storage_from_env() -> anyhow::Result<(Option<StorageConfig>, Option<PublicStorageConfig>)> {
    let private = env_opt("S3_PRIVATE_BUCKET");
    let public = match (env_opt("S3_PUBLIC_BUCKET"), env_opt("S3_PUBLIC_URL")) {
        (None, None) => None,
        (Some(bucket), Some(url)) => {
            let parsed: reqwest::Url = url.parse().context("S3_PUBLIC_URL inválida")?;
            anyhow::ensure!(
                matches!(parsed.scheme(), "https" | "http"),
                "S3_PUBLIC_URL inválida: {url} (use https://...)"
            );
            Some((bucket, url.trim_end_matches('/').to_owned()))
        }
        _ => anyhow::bail!("S3_PUBLIC_BUCKET e S3_PUBLIC_URL vão juntas"),
    };
    let keys = (env_opt("S3_ACCESS_KEY_ID"), env_opt("S3_SECRET_ACCESS_KEY"));
    let (access_key_id, secret_access_key) = match keys {
        (None, None) if private.is_none() && public.is_none() => return Ok((None, None)),
        (Some(key), Some(secret)) if private.is_some() || public.is_some() => (key, secret),
        _ => anyhow::bail!(
            "S3_ACCESS_KEY_ID e S3_SECRET_ACCESS_KEY vão juntas e com S3_PRIVATE_BUCKET ou S3_PUBLIC_BUCKET (ou nenhuma, sem screenshots, logos e capas)"
        ),
    };
    let bucket = |bucket| StorageConfig {
        endpoint: env_opt("S3_ENDPOINT"),
        region: env_or("S3_REGION", "us-east-1"),
        bucket,
        access_key_id: access_key_id.clone(),
        secret_access_key: secret_access_key.clone(),
    };
    Ok((
        private.map(bucket),
        public.map(|(name, url)| PublicStorageConfig {
            storage: bucket(name),
            url,
        }),
    ))
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
