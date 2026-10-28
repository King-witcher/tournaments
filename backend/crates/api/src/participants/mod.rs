//! Inscritos de um torneio: listar, o membro se inscrever ou sair (torneio
//! público, com a inscrição aberta), o organizador inscrever ou tirar
//! membros (público ou privado, até começar), o check-in (`check_in`) e o
//! seeding (`seeding`). O link de convite de torneio fica em `invites`, que
//! inscreve por `register`; iniciar o torneio fica em `lifecycle`, que fecha
//! o seeding por `finalize_seeding`. A chave (`matches`) lê os participantes
//! por `seeded`. Quem sai do tenant sai das inscrições abertas por
//! `unregister`, e desfazer o início tira quem saiu por `delete_departed`.

mod check_in;
mod db;
mod handlers;
mod registration;
mod seed_order;
mod seeding;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use db::{Seeded, delete as unregister, delete_departed, seeded};
pub use registration::{Entry, register};
pub use seeding::finalize as finalize_seeding;

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
        .routes(routes!(seeding::get_seeding, seeding::put_seeding))
        .routes(routes!(seeding::shuffle_seeding))
}
