//! Membros de um tenant: listar, o owner trocar papéis e cada um editar o
//! próprio apelido.

mod db;
mod handlers;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::update_my_membership))
        .routes(routes!(handlers::list_members))
        .routes(routes!(handlers::set_member_role))
}
