//! Ciclo de vida do torneio depois de publicar: abrir o check-in antes da
//! hora, reabrir a inscrição, iniciar e desfazer o início, todas por
//! `transition::apply`, e quem deixa os torneios no meio do caminho
//! (`withdrawal`: o membro que sai da comunidade). Fica acima de
//! `tournaments`, `participants` e `matches` porque iniciar fecha o seeding e
//! gera a chave.

mod handlers;
mod transition;
mod withdrawal;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use withdrawal::leave_tournaments;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::open_check_in))
        .routes(routes!(handlers::reopen_registration))
        .routes(routes!(handlers::start, handlers::undo_start))
}
