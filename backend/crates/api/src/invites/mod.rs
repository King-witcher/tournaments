//! Links de convite: organizer ou owner gera, lista e revoga; quem recebe vê
//! a prévia sem login e aceita para entrar como player.

mod db;
mod handlers;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::create_invite, handlers::list_invites))
        .routes(routes!(handlers::revoke_invite))
        .routes(routes!(handlers::preview_invite))
        .routes(routes!(handlers::accept_invite))
}
