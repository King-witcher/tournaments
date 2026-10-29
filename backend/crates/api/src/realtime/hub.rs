//! O canal em memória do tempo real: o despachante publica cada evento do
//! outbox e cada socket recebe todos e filtra os seus (`Connection`). E os
//! sinais que acordam o despachante e o relógio do veto.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tokio::sync::{Notify, broadcast};

use crate::outbox::{self, Event};

/// Eventos que um socket lento pode ficar devendo antes de perder algum
/// (`Lagged`: ele recarrega tudo o que assina).
const CAPACITY: usize = 1024;

#[derive(Clone)]
pub struct Realtime(Arc<Inner>);

struct Inner {
    events: broadcast::Sender<Arc<Event>>,
    wake: Notify,
    /// Acorda o relógio do veto: pode haver prazo novo.
    clock: Notify,
}

impl Default for Realtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Realtime {
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(CAPACITY);
        Self(Arc::new(Inner {
            events,
            wake: Notify::new(),
            clock: Notify::new(),
        }))
    }

    /// Acorda o despachante: há evento novo no outbox. Sem ninguém esperando,
    /// o aviso fica guardado para a próxima espera. Acorda também o relógio do
    /// veto: o que mudou pode ter criado um prazo (um passo, um jogo novo).
    pub fn wake(&self) {
        self.0.wake.notify_one();
        self.0.clock.notify_one();
    }

    /// Só o despachante (o relógio, depois de gravar, não se acorda de novo).
    pub(super) fn wake_dispatcher(&self) {
        self.0.wake.notify_one();
    }

    pub(super) async fn woken(&self) {
        self.0.wake.notified().await;
    }

    pub(super) async fn clock_woken(&self) {
        self.0.clock.notified().await;
    }

    /// Os eventos publicados daqui em diante.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Event>> {
        self.0.events.subscribe()
    }

    /// Sem socket nenhum, o evento só não vai para ninguém.
    pub(super) fn publish(&self, event: Event) {
        let _ = self.0.events.send(Arc::new(event));
    }
}

/// Camada de todas as rotas: depois de uma requisição que emitiu evento (com
/// a transação já encerrada), acorda o despachante.
pub async fn wake_after_emit(
    State(realtime): State<Realtime>,
    request: Request,
    next: Next,
) -> Response {
    let (response, emitted) = outbox::track(next.run(request)).await;
    if emitted {
        realtime.wake();
    }
    response
}
