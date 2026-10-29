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
//! No TDM (times de 2 a 4): a formação antes do início (`formation`: ver os
//! times e os reservas, criar time, pôr e tirar jogadores; o sorteio, que lê
//! o rating, fica em `lifecycle` com `draw_teams`), o nome e o capitão e a
//! substituição durante o torneio (`roster`). A escalação de cada mapa (quem
//! está no time e na comunidade no envio) é gravada pelo banco.

mod db;
mod formation;
mod roster;
mod seed_order;
mod seeding;
mod team;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use db::{
    by_ids, clear as clear_teams, create as create_team, of_user, players, save_teams_seed, seeded,
    withdraw, withdraw_departing,
};
pub use formation::{editable as formation_editable, formation};
pub use seeding::finalize as finalize_seeding;
pub use team::StoredTeam;
pub use types::{DrawMethod, DrawTeams, Team, TeamFormation, TeamMember, TeamPath};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(seeding::get_seeding, seeding::put_seeding))
        .routes(routes!(seeding::shuffle_seeding))
        .routes(routes!(formation::get_teams, formation::create_team))
        .routes(routes!(formation::put_member, formation::delete_member))
        .routes(routes!(roster::update_team))
        .routes(routes!(roster::substitute))
}
