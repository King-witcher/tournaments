//! API HTTP. Cada domínio é um módulo com o próprio `router()`; `shared` tem
//! o que todos usam. Regras de organização em `backend/CLAUDE.md`.

pub mod auth;
pub mod config;
mod health;
mod invites;
mod members;
pub mod shared;
mod tenants;

use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use sqlx::{PgPool, migrate::Migrator};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

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

/// Monta o app completo: rotas, OpenAPI em `/openapi.json` e Swagger UI em `/docs`.
pub fn app(state: AppState, web_origin: HeaderValue) -> Router {
    let (router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(health::router())
        .merge(auth::router())
        .merge(tenants::router())
        .merge(members::router())
        .merge(invites::router())
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

    router
        .merge(SwaggerUi::new("/docs").url("/openapi.json", openapi))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
