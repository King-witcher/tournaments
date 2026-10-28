//! Login com Discord (OAuth2) e com username e senha, sessão em cookie e o
//! extractor `CurrentUser`. A conta pode ter os dois logins, ligar e desligar
//! cada um, mas nunca fica sem nenhum. Sessões e erros de senha ficam no Redis
//! (`kv/`); contas e logins, no banco (`db/`).

mod attempts;
mod credentials;
mod current_user;
mod db;
mod discord;
mod handlers;
mod kv;
mod oauth;
mod password;
mod session;
mod types;
mod username;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use current_user::CurrentUser;
pub use discord::{DiscordApi, DiscordUser, HttpDiscord};
/// Os testes criam sessões por aqui, sem passar pelo login.
pub use kv::create_session;
pub use session::AuthSettings;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(oauth::discord_login))
        .routes(routes!(oauth::discord_link))
        .routes(routes!(oauth::discord_callback))
        .routes(routes!(oauth::unlink_discord))
        .routes(routes!(credentials::signup))
        .routes(routes!(credentials::login))
        .routes(routes!(
            credentials::create_password,
            credentials::change_password,
            credentials::remove_password
        ))
        .routes(routes!(handlers::me, handlers::update_me))
        .routes(routes!(handlers::logins))
        .routes(routes!(handlers::logout))
        .routes(routes!(handlers::sessions, handlers::end_other_sessions))
        .routes(routes!(handlers::end_session))
}
