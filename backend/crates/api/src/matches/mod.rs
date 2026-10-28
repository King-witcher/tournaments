//! Chave e resultados: ver a chave e uma partida, enviar o placar, confirmar
//! ou contestar, e o organizador resolver, lançar, corrigir e dar W.O.
//!
//! A chave nunca é gravada: sai do `replay` do `engine` com a configuração
//! da fase, os participantes pelo seed e os resultados confirmados, na ordem
//! em que passaram a valer (`stage`). Gerar e descartar a chave fica no
//! início e no desfazer do `lifecycle` (`start`, `discard`); encerrar e
//! reabrir o torneio conforme a chave, aqui (`stage::sync_status`). A
//! confirmação por tempo é aplicada ao ler e agir (`due`).

mod db;
mod detail;
mod due;
mod handlers;
mod key;
mod result;
mod stage;
mod types;
mod view;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use stage::{discard, start};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_bracket))
        .routes(routes!(handlers::get_match))
}
