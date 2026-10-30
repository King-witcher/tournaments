//! Inscritos de um torneio: listar, o membro se inscrever ou sair (com a
//! inscrição aberta, até o início), o organizador inscrever ou tirar membros
//! (com qualquer tipo de inscrição, até começar) e o check-in (`check_in`).
//! Quem se inscreve durante o check-in já entra com o check-in feito. A
//! inscrição é individual, por `user_id`; quem ocupa a chave é o time do
//! inscrito (`teams`, que no duelo o banco cria e apaga junto com a
//! inscrição). O link de convite de torneio fica em `invites`, que inscreve
//! por `register`. Quem sai do tenant sai das inscrições abertas por
//! `unregister`, e desfazer o início tira quem desistiu (pelo time) ou saiu
//! por `delete_departed`. O perfil do membro (`members`) lê as inscrições
//! dele por `registrations_of`.

mod check_in;
mod db;
mod handlers;
mod registration;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use db::{delete as unregister, delete_departed, registrations_of};
pub use registration::{Entry, register};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            handlers::list_participants,
            handlers::add_participant
        ))
        .routes(routes!(handlers::remove_participant))
        .routes(routes!(handlers::register, handlers::withdraw))
        .routes(routes!(check_in::check_in, check_in::undo_check_in))
        .routes(routes!(
            check_in::check_in_participant,
            check_in::undo_check_in_participant
        ))
}
