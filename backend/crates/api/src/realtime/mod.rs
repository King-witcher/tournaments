//! Tempo real: o WebSocket de cada aba (`GET /ws`, `socket`, que abre com as
//! notificações não lidas), o canal em memória por onde os eventos chegam aos
//! sockets (`Realtime`, em `hub`), o despachante que tira os eventos do
//! outbox, publica, guarda os avisos no sininho e manda os Web Push
//! (`dispatcher`) e o agendador que faz o que vence com o tempo
//! (`scheduler`: confirmação por tempo, exclusões de comunidade e de conta, a rotina diária dos screenshots,
//! rating das comunidades marcadas, notificações vencidas) e o relógio dos
//! passos do veto (`clock`), que dorme até o próximo prazo e, ao acordar, relê
//! só os torneios que os eventos alcançaram ou cujo prazo venceu. Fica acima
//! de todos os domínios: confere o acesso
//! como na leitura (`tenants`, `tournaments`), o socket e o despachante
//! chamam `notifications`, e o agendador chama `tournaments`, `matches`,
//! `tenants`, `members`, `screenshots`, `ratings` e `notifications`.
//! Quem muda o estado emite pelo `outbox`, que fica embaixo de todos. A sala de
//! veto, sem login e sem avisos, tem o próprio socket por token (`room_socket`),
//! o próprio canal (`veto_rooms::Live`) e o próprio relógio (`clock::run_rooms`).

mod clock;
mod connection;
mod db;
mod dispatcher;
mod hub;
mod room_socket;
mod scheduler;
mod socket;
mod types;

use chrono::{DateTime, Utc};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, shared::ApiError};

pub use connection::MAX_TOPICS;
pub use dispatcher::dispatch_once;
pub use hub::{ORIGIN_HEADER, Published, Realtime, wake_after_emit};
pub use scheduler::{Tick, tick};
pub use types::{Changed, ClientMessage, RoomMessage, ServerMessage, Topic};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(socket::ws))
        .routes(routes!(room_socket::veto_room_socket))
}

/// Uma passada do relógio do veto em `now` (os testes; em produção, a tarefa
/// de `spawn`). Devolve quantos passos sorteou.
pub async fn veto_clock(state: &AppState, now: DateTime<Utc>) -> Result<u64, ApiError> {
    Ok(clock::pass(state, now).await?.written)
}

/// Sobe o despachante, o agendador e os relógios do veto (o `main`; os testes
/// chamam `dispatch_once`, `tick` e `veto_clock` direto).
pub fn spawn(state: AppState) {
    let dispatcher = state.clone();
    tokio::spawn(async move {
        loop {
            let wait = if dispatcher::drain(&dispatcher).await {
                dispatcher::IDLE
            } else {
                dispatcher::RETRY
            };
            let _ = tokio::time::timeout(wait, dispatcher.realtime.woken()).await;
        }
    });
    // O relógio é o único que sorteia os passos de quem ninguém lê: se a tarefa
    // cair (pânico), recomeça, relendo todos os torneios.
    let ticking = state.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = tokio::spawn(clock::run(ticking.clone())).await {
                tracing::error!(error = %e, "relógio do veto caiu; recomeça");
            }
            ticking.realtime.wake();
            tokio::time::sleep(clock::PAUSE).await;
        }
    });
    tokio::spawn(clock::run_rooms(state.clone()));
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
