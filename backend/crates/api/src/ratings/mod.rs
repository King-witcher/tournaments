//! Rating da comunidade (decisões em "Rating" no docs/decisions.md): Elo por
//! mapa confirmado dos torneios que contam, mais os ajustes manuais da
//! organização, reprocessados do zero pelo engine. As tabelas `ratings` e
//! `rating_history` são cache: um trigger marca o tenant como desatualizado
//! quando a fonte muda, e o recálculo (`refresh`) roda no agendador, antes de
//! ler e nas rotas daqui. Rotas: o ranking, o rating de um membro (com os
//! últimos mapas e, para a organização, os ajustes), o ajuste manual e a
//! configuração do Elo. Lê mapas, torneios e membros direto no banco, sem
//! chamar `matches`: fica abaixo do `realtime`, que chama `refresh_stale`.

mod db;
mod handlers;
mod history;
mod refresh;
mod settings;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use refresh::refresh_stale;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_ranking))
        .routes(routes!(handlers::get_settings, handlers::put_settings))
        .routes(routes!(handlers::get_member_rating))
        .routes(routes!(handlers::create_adjustment))
}
