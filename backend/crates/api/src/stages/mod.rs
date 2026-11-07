//! As fases ligadas da 2 em diante: ver a fase seguinte antes e depois do
//! início (os candidatos, quem entraria e o que falta para iniciar), abrir o
//! check-in dela, confirmar ou desfazer a presença de um time, iniciar a fase
//! e desfazer o início. Fica acima de `tournaments` (o andamento de cada fase),
//! `teams` (presenças e seeds em `stage_teams`) e `matches` (quem entra, a
//! partir da classificação da anterior, e a chave nova), como o `lifecycle`.
//! A fase 1 continua nascendo no início do torneio (`lifecycle`).
//!
//! - `handlers`: as rotas, com as checagens de cada uma (`rules`).
//! - `rules`: o que impede iniciar a fase e as checagens comuns.
//! - `types`: `StageEntry` e `StageCandidate`.

mod handlers;
mod rules;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_stage))
        .routes(routes!(handlers::open_stage_check_in))
        .routes(routes!(
            handlers::check_in_team,
            handlers::undo_team_check_in
        ))
        .routes(routes!(handlers::start_stage, handlers::undo_stage_start))
}
