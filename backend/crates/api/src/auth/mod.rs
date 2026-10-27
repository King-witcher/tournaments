//! Login com Discord (OAuth2), sessão em cookie e o extractor `CurrentUser`.

mod current_user;
mod db;
mod discord;
mod handlers;
mod session;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use current_user::CurrentUser;
pub use discord::{DiscordApi, DiscordUser, HttpDiscord};
pub use session::AuthSettings;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::discord_login))
        .routes(routes!(handlers::discord_callback))
        .routes(routes!(handlers::me))
        .routes(routes!(handlers::logout))
}
