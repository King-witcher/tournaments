//! API HTTP. Cada domínio é um módulo com o próprio `router()`; `shared` tem
//! o que todos usam. Regras de organização em `backend/CLAUDE.md`.

pub mod auth;
pub mod config;
pub mod discord;
mod health;
pub mod images;
mod invites;
mod lifecycle;
pub mod mail;
mod maps;
mod matches;
mod members;
mod my_tournaments;
mod notifications;
mod organizing;
pub mod outbox;
mod participants;
pub mod push;
mod ratings;
pub mod realtime;
pub mod scanners;
pub mod screenshots;
pub mod shared;
mod teams;
mod tenants;
mod tournaments;
pub mod veto_rooms;

use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    http::{HeaderValue, Method, header},
    middleware,
    routing::get,
};
use sqlx::{PgPool, migrate::Migrator};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

/// As mensagens do WebSocket não aparecem em nenhuma rota HTTP: entram como
/// componentes para o `gen:api` do front gerar os tipos.
#[derive(OpenApi)]
#[openapi(
    info(title = "Telefrag API"),
    components(schemas(
        realtime::ClientMessage,
        realtime::ServerMessage,
        realtime::RoomMessage
    ))
)]
struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: shared::Redis,
    /// As sessões em memória, que dispensam o Redis a cada requisição.
    pub sessions: auth::SessionStore,
    /// Erros de senha por email, à parte de `limits`: emails inventados enchem
    /// o mapa, e isso não pode zerar os outros limites.
    pub password_attempts: shared::WindowLimit,
    pub discord: Arc<dyn auth::DiscordApi>,
    pub google: Arc<dyn auth::GoogleApi>,
    pub auth: Arc<auth::AuthSettings>,
    /// Canal dos sockets e o despertador do despachante.
    pub realtime: realtime::Realtime,
    /// `None`: sem chaves VAPID, o Web Push fica desligado.
    pub push: Option<Arc<dyn push::WebPush>>,
    /// Bucket dos screenshots do placar; `None` sem as variáveis `S3_*` (os
    /// screenshots ficam desligados).
    pub storage: Option<shared::Storage>,
    /// Bucket dos logos e das capas, servido pelo CDN; `None` sem
    /// `S3_PUBLIC_BUCKET` (logos e capas ficam desligados).
    pub public_storage: Option<shared::Storage>,
    /// Envio de email (Resend; sem chave, só o log).
    pub mail: mail::Mail,
    /// Webhooks do Discord das comunidades (a fila é esvaziada por `discord::spawn`).
    pub webhooks: discord::Webhooks,
    /// Limites em memória das rotas que chamam o Discord.
    pub limits: shared::WindowLimit,
    /// O canal e os limites de conexão das salas de veto.
    pub veto_rooms: veto_rooms::Live,
    /// IPs de scanners de vulnerabilidade, que recebem 404 em tudo.
    pub bans: scanners::Bans,
    /// Serve `/openapi.json` e `/docs` (`API_DOCS`).
    pub api_docs: bool,
}

/// Monta o app completo: rotas e, com `api_docs`, OpenAPI em `/openapi.json` (lido
/// pelo `gen:api` do front) e Scalar em `/docs`.
pub fn app(state: AppState, web_origin: HeaderValue) -> Router {
    let realtime = state.realtime.clone();
    let api_docs = state.api_docs;
    let scanner_state = state.clone();
    let (router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(health::router())
        .merge(auth::router())
        .merge(tenants::router())
        .merge(discord::router())
        .merge(members::router())
        .merge(invites::router())
        .merge(tournaments::router())
        .merge(participants::router())
        .merge(teams::router())
        .merge(lifecycle::router())
        .merge(maps::router())
        .merge(matches::router())
        .merge(my_tournaments::router())
        .merge(notifications::router())
        .merge(organizing::router())
        .merge(screenshots::router())
        .merge(images::router())
        .merge(push::router())
        .merge(ratings::router())
        .merge(veto_rooms::router())
        .merge(realtime::router())
        .merge(scanners::router())
        .with_state(state)
        .split_for_parts();

    let cors = CorsLayer::new()
        .allow_origin(web_origin.clone())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE, realtime::ORIGIN_HEADER])
        .allow_credentials(true)
        // Sem isto, o navegador guarda a resposta do preflight por 5 s, e cada
        // POST com JSON paga um OPTIONS antes (uma ida a mais até a
        // Virgínia). 2 h é o teto do Chrome.
        .max_age(Duration::from_secs(7200));

    let refuse_state = (scanner_state, scanners::no_routes(cors.clone()));
    let router = if api_docs {
        let spec = Json(openapi.clone());
        router
            .route("/openapi.json", get(move || async move { spec }))
            .merge(Scalar::with_url("/docs", openapi))
    } else {
        router
    };
    router
        .layer(middleware::from_fn_with_state(
            realtime,
            realtime::wake_after_emit,
        ))
        .layer(middleware::from_fn_with_state(
            web_origin,
            shared::origin::refuse_cross_origin,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            refuse_state,
            scanners::refuse_banned,
        ))
}
