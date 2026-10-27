pub mod auth;
pub mod config;
pub mod current_user;
pub mod discord;
pub mod error;
mod extract;
mod health;
mod ids;
mod invites;
pub mod tenant_member;
mod tenants;

use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use sqlx::{PgPool, migrate::Migrator};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

#[derive(OpenApi)]
#[openapi(info(title = "Tournaments API"))]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub discord: Arc<dyn discord::DiscordApi>,
    pub auth: Arc<auth::AuthSettings>,
}

/// Monta o app completo: rotas, OpenAPI em `/openapi.json` e Swagger UI em `/docs`.
pub fn app(state: AppState, web_origin: HeaderValue) -> Router {
    let (router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health::health))
        .routes(routes!(auth::discord_login))
        .routes(routes!(auth::discord_callback))
        .routes(routes!(auth::me))
        .routes(routes!(auth::logout))
        .routes(routes!(tenants::create_tenant, tenants::list_my_tenants))
        .routes(routes!(tenants::get_tenant))
        .routes(routes!(tenants::update_my_membership))
        .routes(routes!(tenants::list_members))
        .routes(routes!(tenants::set_member_role))
        .routes(routes!(invites::create_invite, invites::list_invites))
        .routes(routes!(invites::revoke_invite))
        .routes(routes!(invites::preview_invite))
        .routes(routes!(invites::accept_invite))
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
