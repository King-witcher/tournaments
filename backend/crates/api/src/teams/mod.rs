//! Times de um torneio: quem ocupa um lado da chave em todos os modos. No
//! duelo, cada inscrito é um time de um jogador, que o banco cria junto com a
//! inscrição e apaga com ela (`participants` não sabe de time). Aqui ficam o
//! time com os jogadores (`StoredTeam`, que a chave em `matches` carrega por
//! `seeded` e a página da pessoa em `ratings` por `by_ids`) e como a API o
//! mostra (`Team`), o seeding antes de iniciar (`seeding`; `lifecycle` fecha a
//! ordem por `finalize_seeding` ao iniciar) e a desistência, que é do time
//! (`withdraw`, marcada pelo organizador; `withdraw_departing`, quando o
//! último jogador do time sai da comunidade). A rota da desistência fica em
//! `lifecycle`, que dá os W.O. pela chave.
//!
//! A formação de times (TDM) entra aqui: criar e desfazer times, nome e
//! capitão, e quem joga cada mapa (hoje o banco grava a escalação com os
//! jogadores do time no momento do mapa).

mod db;
mod seed_order;
mod seeding;
mod team;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use db::{by_ids, seeded, withdraw, withdraw_departing};
pub use seeding::finalize as finalize_seeding;
pub use team::StoredTeam;
pub use types::{Team, TeamPath};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(seeding::get_seeding, seeding::put_seeding))
        .routes(routes!(seeding::shuffle_seeding))
}
