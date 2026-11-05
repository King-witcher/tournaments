//! O relógio do veto: o passo tem 30 s (por torneio), e o agendador de 10 s
//! seria grosso demais para o prazo. A tarefa dorme até o próximo prazo de
//! veto de todos os torneios e é acordada quando um evento alcança um torneio
//! (toda requisição que emite passa por `Realtime::emitted`, e um passo, um
//! "pronto", um jogo novo ou um elenco novo sempre emitem): então passa de novo
//! (`matches::VetoClock`), relendo só os torneios que mudaram ou cujo prazo
//! venceu, e todos a cada minuto. A passada que falha tenta de novo em 10 s, e
//! a tarefa que cai recomeça (`spawn`), relendo tudo. O sorteio também acontece
//! ao ler e antes de cada ação. As salas de veto têm o relógio delas
//! (`run_rooms`): o clique numa sala não relê os torneios abertos.

use chrono::{DateTime, Utc};

use crate::{AppState, matches::VetoPass, outbox, shared::ApiError, veto_rooms};

/// Folga depois do prazo, para a passada cair depois dele.
const SLACK: std::time::Duration = std::time::Duration::from_millis(50);
/// A pausa quando o prazo seguinte das salas já venceu: a passada gravou nada, e
/// a sala que falhou continuaria vencida (o erro dela fica no log). Quem grava
/// acorda o relógio na hora, então uma fila grande não espera por isto.
const ROOMS_PAUSE: std::time::Duration = std::time::Duration::from_secs(10);
/// A pausa depois de uma passada dos torneios que falhou, ou cujo prazo
/// continua vencido depois de aplicado (não havia o que sortear).
pub(super) const PAUSE: std::time::Duration = std::time::Duration::from_secs(10);

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
        let now = Utc::now();
        let wait = match pass(&state, now).await {
            Ok(VetoPass { wake_at, .. }) if wake_at > now => {
                (wake_at - Utc::now()).to_std().unwrap_or_default() + SLACK
            }
            Ok(_) => PAUSE,
            Err(e) => {
                tracing::error!(code = e.code, "relógio do veto falhou");
                PAUSE
            }
        };
        tokio::select! {
            () = tokio::time::sleep(wait) => {}
            () = state.realtime.clock_woken() => {}
        }
    }
}

/// Uma passada em `now`, com os torneios que os eventos alcançaram desde a
/// anterior. O que ela emite (passos sorteados, a série que começa sozinha)
/// acorda o despachante, não o relógio: a passada já sabe o que gravou.
pub(super) async fn pass(state: &AppState, now: DateTime<Utc>) -> Result<VetoPass, ApiError> {
    let (mut clock, changed) = state.realtime.veto_clock().await;
    let (pass, touched) = outbox::track(None, clock.pass(&state.db, now, &changed)).await;
    if pass.is_err() {
        state.realtime.missed();
    }
    if touched.any() {
        state.realtime.wake_dispatcher();
    }
    pass
}
