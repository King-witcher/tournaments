//! O relógio do veto: o passo tem 30 s (por torneio), e o agendador de 10 s
//! seria grosso demais para o prazo. A tarefa dorme até o próximo prazo de
//! veto de todos os torneios e é acordada quando algo muda (toda requisição
//! que emite acorda também o despachante, e um passo, um "pronto" ou um jogo
//! novo sempre emitem): então recalcula o próximo prazo, sorteando antes os
//! passos que já venceram (`matches::veto_clock`). Sem veto aberto, ela só
//! dorme. O sorteio também acontece ao ler e antes de cada ação, e o agendador
//! passa a cada 10 s como rede de segurança. As salas de veto têm o relógio
//! delas (`run_rooms`): o clique numa sala não relê os torneios abertos.

use chrono::Utc;

use crate::{AppState, matches, veto_rooms};

/// Folga depois do prazo, para a passada cair depois dele.
const SLACK: std::time::Duration = std::time::Duration::from_millis(50);
/// A pausa quando o prazo seguinte das salas já venceu: a passada gravou nada, e
/// a sala que falhou continuaria vencida (o erro dela fica no log). Quem grava
/// acorda o relógio na hora, então uma fila grande não espera por isto.
const ROOMS_PAUSE: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) async fn run_rooms(state: AppState) {
    loop {
        let next = match veto_rooms::clock(&state, Utc::now()).await {
            Ok((_, next)) => next,
            Err(e) => {
                tracing::error!(code = e.code, "relógio das salas de veto falhou");
                None
            }
        };
        let sleep = next.map(|at| {
            let wait = (at - Utc::now())
                .to_std()
                .map_or(ROOMS_PAUSE, |d| d + SLACK);
            tokio::time::sleep(wait)
        });
        match sleep {
            Some(sleep) => {
                tokio::select! {
                    () = sleep => {}
                    () = state.veto_rooms.clock_woken() => {}
                }
            }
            None => state.veto_rooms.clock_woken().await,
        }
    }
}

pub(super) async fn run(state: AppState) {
    loop {
        let next = match matches::veto_clock(&state.db, Utc::now()).await {
            Ok((written, next)) => {
                if written > 0 {
                    state.realtime.wake_dispatcher();
                }
                next
            }
            Err(e) => {
                tracing::error!(code = e.code, "relógio do veto falhou");
                None
            }
        };
        let sleep = next.map(|at| {
            let wait = (at - Utc::now()).to_std().unwrap_or_default();
            tokio::time::sleep(wait + SLACK)
        });
        match sleep {
            Some(sleep) => {
                tokio::select! {
                    () = sleep => {}
                    () = state.realtime.clock_woken() => {}
                }
            }
            None => state.realtime.clock_woken().await,
        }
    }
}
