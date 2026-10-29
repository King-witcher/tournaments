use anyhow::Context;
use std::sync::Arc;

use api::{
    AppState, MIGRATOR,
    auth::{AuthSettings, HttpDiscord, HttpGoogle, OAuthClient},
    config::Config,
    discord::{self, Webhooks},
    mail::{LogMailer, Mail, Mailer, ResendMailer},
    push::{HttpWebPush, WebPush},
    realtime::{self, Realtime},
    shared::{Redis, Storage},
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,api=debug")),
        )
        .init();

    let config = Config::from_env()?;
    // Antes das migrations, como o Redis abaixo: um deploy sem o Resend (ou com
    // o bucket mal configurado) falha sem mexer no banco.
    let mail = Mail::new(mailer(&config)?, config.email_daily_limit);
    let storage = storage(&config)?;

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .context("falha ao conectar no Postgres (confira DATABASE_URL)")?;

    // O rustls vem com dois provedores de criptografia (ring pelo sqlx, aws-lc
    // pelo reqwest) e assim não escolhe sozinho; o TLS do Redis (`rediss://`)
    // usa o padrão do processo.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let redis =
        Redis::open(&config.redis_url, &config.redis_prefix).context("REDIS_URL inválida")?;
    // Antes das migrations: um deploy sem Redis falha sem mexer no banco, e a
    // instância antiga continua com o schema que conhece.
    redis
        .ping()
        .await
        .context("falha ao conectar no Redis (confira REDIS_URL)")?;

    MIGRATOR
        .run(&db)
        .await
        .context("falha ao aplicar migrations")?;

    let web_origin = config.web_origin.parse().context("WEB_ORIGIN inválido")?;
    let web = config.web_origin.trim_end_matches('/');
    let push = web_push(&config)?;
    let state = AppState {
        db,
        redis,
        discord: Arc::new(HttpDiscord::new(
            config.discord_client_id.clone().unwrap_or_default(),
            config.discord_client_secret.clone().unwrap_or_default(),
        )),
        google: Arc::new(HttpGoogle::new(
            config.google_client_id.clone().unwrap_or_default(),
            config.google_client_secret.clone().unwrap_or_default(),
        )),
        auth: Arc::new(AuthSettings {
            discord: oauth_client(
                "Discord",
                config.discord_client_id.as_deref(),
                config.discord_client_secret.as_deref(),
                web,
            ),
            google: oauth_client(
                "Google",
                config.google_client_id.as_deref(),
                config.google_client_secret.as_deref(),
                web,
            ),
            web_origin: web.to_owned(),
            cookie_domain: config.cookie_domain.clone(),
            cookie_secure: config.cookie_secure,
        }),
        realtime: Realtime::new(),
        push,
        storage,
        mail,
        webhooks: Webhooks::new(),
    };
    realtime::spawn(state.clone());
    discord::spawn(state.clone());
    let app = api::app(state, web_origin);

    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    tracing::info!("API em http://{0} (docs em http://{0}/docs)", config.addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// O app do provedor `name`, se o client id e o secret existem; sem os dois,
/// o login com ele responde 503. Ele volta para a página de callback do front
/// em `web` (`/auth/{provedor}/callback`), o redirect a registrar no provedor.
fn oauth_client(
    name: &str,
    client_id: Option<&str>,
    client_secret: Option<&str>,
    web: &str,
) -> Option<OAuthClient> {
    let (Some(client_id), Some(_)) = (client_id, client_secret) else {
        let env = name.to_ascii_uppercase();
        tracing::warn!("{env}_CLIENT_ID/SECRET ausentes: login com {name} desligado");
        return None;
    };
    let provider = name.to_ascii_lowercase();
    Some(OAuthClient {
        client_id: client_id.to_owned(),
        redirect_uri: format!("{web}/auth/{provider}/callback"),
    })
}

/// Web Push com as chaves VAPID, se as duas existem (par que não confere
/// impede a subida); sem elas, desligado.
fn web_push(config: &Config) -> anyhow::Result<Option<Arc<dyn WebPush>>> {
    let (Some(public), Some(private)) = (&config.vapid_public_key, &config.vapid_private_key)
    else {
        tracing::warn!("VAPID_PUBLIC_KEY/PRIVATE_KEY ausentes: Web Push desligado");
        return Ok(None);
    };
    let subject = config
        .vapid_subject
        .clone()
        .unwrap_or_else(|| config.web_origin.trim_end_matches('/').to_owned());
    Ok(Some(Arc::new(HttpWebPush::new(public, private, subject)?)))
}

/// O bucket dos screenshots, com as variáveis `S3_*`; sem elas, desligado.
fn storage(config: &Config) -> anyhow::Result<Option<Storage>> {
    let Some(storage) = &config.storage else {
        tracing::warn!(
            "S3_BUCKET/S3_ACCESS_KEY_ID/S3_SECRET_ACCESS_KEY ausentes: screenshots desligados"
        );
        return Ok(None);
    };
    Ok(Some(Storage::new(storage.clone())?))
}

/// Resend com a chave e o remetente. Sem os dois, os emails vão para o log, o
/// que só serve em desenvolvimento: em produção (`COOKIE_SECURE`) o log
/// guardaria os links de cadastro e de recuperação, e a API não sobe.
fn mailer(config: &Config) -> anyhow::Result<Arc<dyn Mailer>> {
    match (&config.resend_api_key, &config.email_from) {
        (Some(key), Some(from)) => Ok(Arc::new(ResendMailer::new(key.clone(), from.clone()))),
        _ if config.cookie_secure => {
            anyhow::bail!("RESEND_API_KEY e EMAIL_FROM são obrigatórias em produção")
        }
        _ => {
            tracing::warn!("RESEND_API_KEY/EMAIL_FROM ausentes: emails só no log");
            Ok(Arc::new(LogMailer))
        }
    }
}

/// Encerra com Ctrl+C localmente e com SIGTERM no Render.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("falha ao instalar handler de Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("falha ao instalar handler de SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
