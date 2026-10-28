//! API HTTP. Cada domínio é um módulo com o próprio `router()`; `shared` tem
//! o que todos usam. Regras de organização em `backend/CLAUDE.md`.

pub mod auth;
pub mod config;
mod health;
mod invites;
mod lifecycle;
mod members;
mod participants;
pub mod shared;
mod tenants;
mod tournaments;

use std::sync::Arc;

use axum::{
    Json, Router,
    http::{HeaderValue, Method, header},
    routing::get,
};
use sqlx::{PgPool, migrate::Migrator};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[derive(OpenApi)]
#[openapi(info(title = "Tournaments API"))]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub discord: Arc<dyn auth::DiscordApi>,
    pub auth: Arc<auth::AuthSettings>,
}

/// Monta o app completo: rotas, OpenAPI em `/openapi.json` (lido pelo `gen:api`
/// do front) e Scalar em `/docs`.
pub fn app(state: AppState, web_origin: HeaderValue) -> Router {
    let (router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(health::router())
        .merge(auth::router())
        .merge(tenants::router())
        .merge(members::router())
        .merge(invites::router())
        .merge(tournaments::router())
        .merge(participants::router())
        .merge(lifecycle::router())
        .with_state(state)
        .split_for_parts();

    let cors = CorsLayer::new()
        .allow_origin(web_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(true);

    let spec = Json(openapi.clone());
    router
        .route("/openapi.json", get(move || async move { spec }))
        .merge(Scalar::with_url("/docs", openapi))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
