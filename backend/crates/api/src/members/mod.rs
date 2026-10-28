//! Membros de um tenant: listar, o owner trocar papéis, cada um editar o
//! próprio apelido, sair da comunidade e remover um membro (`departure`).
//! Sair ou ser removido deixa o membro inativo, com o histórico; mexe nos
//! torneios (`lifecycle`) e nos convites (`invites`) dele, então este domínio
//! fica acima deles.

mod db;
mod departure;
mod handlers;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            handlers::update_my_membership,
            handlers::leave_tenant
        ))
        .routes(routes!(handlers::list_members))
        .routes(routes!(handlers::remove_member))
        .routes(routes!(handlers::set_member_role))
}
