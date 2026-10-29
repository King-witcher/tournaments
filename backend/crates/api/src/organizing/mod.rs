//! `GET /me/organizing`: o que espera a organização em todas as comunidades
//! em que quem vê é organizer ou owner ativo, com o caminho de cada item:
//! mapas contestados a resolver e torneios cujo início marcado chegou sem
//! terem sido iniciados (o critério do aviso `tournament.start_due`). É o
//! par de `/me/tournaments` para quem organiza, numa rota própria: só a home
//! de quem organiza pede (o front sabe pelos papéis de "Minhas comunidades"),
//! e a resposta não depende de refazer chave nenhuma.
//!
//! Só leitura, em duas queries (`db`) que leem direto as tabelas de
//! `tournaments`, `participants` e `matches`: passar pelo domínio de cada um
//! seria uma busca por torneio. Organizer e owner veem todos os torneios da
//! comunidade (`visibility`), então o papel basta como filtro.
//!
//! Quem muda uma dessas listas emite "Seus torneios" (`me`) para a
//! organização da comunidade (`tenants::organizing_changed`): a chave quando
//! o conjunto de mapas contestados muda (`matches::changes`), o torneio
//! quando muda na hora de iniciar (`tournaments::changes`), e o iniciar, o
//! desfazer, o adiar e o aviso do agendador.

mod db;
mod handlers;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(handlers::organizing))
}
