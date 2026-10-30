//! Tempo real: o WebSocket de cada aba (`GET /ws`, `socket`, que abre com as
//! notificações não lidas), o canal em memória por onde os eventos chegam aos
//! sockets (`Realtime`, em `hub`), o despachante que tira os eventos do
//! outbox, publica, guarda os avisos no sininho e manda os Web Push
//! (`dispatcher`) e o agendador que faz o que vence com o tempo
//! (`scheduler`: check-in na hora, hora de iniciar, confirmação por tempo,
//! sorteio dos passos de veto vencidos, exclusões de comunidade e de conta,
//! a rotina diária dos screenshots, rating das comunidades marcadas,
//! notificações vencidas) e o relógio dos passos do veto (`clock`), que dorme
//! até o próximo prazo. Fica acima de todos os domínios: confere o acesso
//! como na leitura (`tenants`, `tournaments`), o socket e o despachante
//! chamam `notifications`, e o agendador chama `tournaments`, `matches`,
//! `tenants`, `members`, `screenshots`, `ratings` e `notifications`.
//! Quem muda o estado emite pelo `outbox`, que fica embaixo de todos.

mod clock;
mod connection;
mod db;
mod dispatcher;
mod hub;
mod scheduler;
mod socket;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use connection::MAX_TOPICS;
pub use dispatcher::dispatch_once;
pub use hub::{Realtime, wake_after_emit};
pub use scheduler::{Tick, tick};
pub use types::{Changed, ClientMessage, ServerMessage, Topic};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(socket::ws))
}

/// Sobe o despachante, o agendador e o relógio do veto (o `main`; os testes
/// chamam `dispatch_once` e `tick` direto).
pub fn spawn(state: AppState) {
    let dispatcher = state.clone();
    tokio::spawn(async move {
        loop {
            dispatcher::drain(&dispatcher).await;
            dispatcher.realtime.woken().await;
        }
    });
    tokio::spawn(clock::run(state.clone()));
    tokio::spawn(async move {
        let mut every = tokio::time::interval(scheduler::EVERY);
        every.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            every.tick().await;
            if let Err(e) = tick(&state, chrono::Utc::now()).await {
                tracing::error!(code = e.code, "agendador falhou");
            }
        }
    });
}
