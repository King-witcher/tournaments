use anyhow::Context;
use std::sync::Arc;

use api::{
    AppState, MIGRATOR,
    auth::{AuthSettings, HttpDiscord},
    config::Config,
    shared::Redis,
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
    let configured = config.discord_client_id.is_some() && config.discord_client_secret.is_some();
    if !configured {
        tracing::warn!("DISCORD_CLIENT_ID/SECRET ausentes: login com Discord desligado");
    }
    let state = AppState {
        db,
        redis,
        discord: Arc::new(HttpDiscord::new(
            config.discord_client_id.clone().unwrap_or_default(),
            config.discord_client_secret.clone().unwrap_or_default(),
        )),
        auth: Arc::new(AuthSettings {
            discord_client_id: config.discord_client_id.clone().filter(|_| configured),
            redirect_uri: config.discord_redirect_uri.clone(),
            web_origin: config.web_origin.trim_end_matches('/').to_owned(),
            cookie_domain: config.cookie_domain.clone(),
            cookie_secure: config.cookie_secure,
        }),
    };
    let app = api::app(state, web_origin);

    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    tracing::info!("API em http://{0} (docs em http://{0}/docs)", config.addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
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
