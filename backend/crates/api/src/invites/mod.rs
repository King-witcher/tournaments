//! O link de convite de cada comunidade e o de inscrição de cada torneio
//! (`tournament`), que é um convite do tenant que também inscreve. Cada um é
//! uma coluna da própria linha (`invite_token`): organizer ou owner vê e troca
//! por outro; quem recebe vê a prévia sem login e aceita para entrar como
//! player. Quem tinha saído do tenant volta por qualquer link
//! (`join_as_player`). Na comunidade com a exclusão agendada, nenhum link vale
//! (prévia e aceite respondem `tenant.deletion_scheduled`). O token novo é um
//! código de 8 caracteres, que dá para digitar (`token::random_invite_code`); os
//! de 22 de antes continuam valendo. Como código curto se adivinha, a prévia e o
//! aceite têm limite por IP (`limit`).

mod db;
mod handlers;
mod limit;
mod tournament;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_invite_link))
        .routes(routes!(handlers::rotate_invite_link))
        .routes(routes!(handlers::preview_invite))
        .routes(routes!(handlers::accept_invite))
        .routes(routes!(tournament::get_tournament_invite_link))
        .routes(routes!(tournament::rotate_tournament_invite_link))
}
