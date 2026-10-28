//! Ciclo de vida do torneio depois de publicar: abrir o check-in antes da
//! hora, reabrir a inscrição, iniciar e desfazer o início, todas por
//! `transition::apply`. Fica acima de `tournaments` e `participants` porque
//! iniciar fecha o seeding (e, no próximo item, gera a chave).

mod handlers;
mod transition;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::open_check_in))
        .routes(routes!(handlers::reopen_registration))
        .routes(routes!(handlers::start, handlers::undo_start))
}
