//! Inscritos de um torneio: listar, o membro se inscrever ou sair (torneio
//! público, com a inscrição aberta) e o organizador inscrever ou tirar
//! membros (público ou privado, até começar). O link de convite de torneio
//! fica em `invites`, que inscreve por `register`.

mod db;
mod handlers;
mod registration;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use registration::{Entry, register};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            handlers::list_participants,
            handlers::add_participant
        ))
        .routes(routes!(handlers::remove_participant))
        .routes(routes!(handlers::register, handlers::withdraw))
}
