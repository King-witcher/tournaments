//! Ciclo de vida do torneio depois de publicar: abrir o check-in antes da
//! hora, iniciar e desfazer o início, todas por
//! `transition::apply`, e quem deixa um torneio no meio do caminho
//! (`withdrawal`: a desistência do time marcada pelo organizador e o membro
//! que sai da comunidade). Fica acima de `tournaments`, `participants`,
//! `teams` e `matches` porque iniciar fecha o seeding e gera a chave, e a
//! desistência dá W.O. Também o sorteio dos times do TDM (`draw`), que lê o
//! rating da comunidade.

mod draw;
mod handlers;
mod transition;
mod withdrawal;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use withdrawal::leave_tournaments;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::open_check_in))
        .routes(routes!(handlers::start, handlers::undo_start))
        .routes(routes!(withdrawal::withdraw_team))
        .routes(routes!(draw::draw_teams))
}
