use anyhow::Context;
use api::{AppState, MIGRATOR, config::Config};
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

    MIGRATOR
        .run(&db)
        .await
        .context("falha ao aplicar migrations")?;

    let web_origin = config.web_origin.parse().context("WEB_ORIGIN inválido")?;
    let app = api::app(AppState { db }, web_origin);

    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    tracing::info!(
        "API em http://{0} (Swagger UI em http://{0}/docs)",
        config.addr
    );
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
