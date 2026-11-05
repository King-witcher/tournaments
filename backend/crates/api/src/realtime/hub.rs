//! O canal em memória do tempo real: o despachante publica cada evento do
//! outbox e cada socket recebe todos e filtra os seus (`Connection`). E os
//! sinais que acordam o despachante e o relógio do veto, com o que o relógio
//! precisa entre uma passada e outra: os torneios que os eventos alcançaram
//! desde a anterior e o que ele já sabe dos abertos (`matches::VetoClock`).

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
};

use axum::{
    extract::{Request, State},
    http::HeaderName,
    middleware::Next,
    response::Response,
};
use tokio::sync::{Notify, broadcast};

use crate::{
    matches::VetoClock,
    outbox::{self, Event, Touched},
};

/// Eventos que um socket lento pode ficar devendo antes de perder algum
/// (`Lagged`: ele recarrega tudo o que assina).
const CAPACITY: usize = 1024;

/// O cabeçalho em que a aba manda o id da própria conexão (o mesmo de
/// `/ws?connection=`) quando aplica sozinha a resposta da ação: as mudanças de
/// partida que a ação emitir não voltam a essa conexão.
pub const ORIGIN_HEADER: HeaderName = HeaderName::from_static("x-realtime-connection");

/// O id de conexão que a aba escolheu, se tem a forma aceita (16 a 64 de
/// `A-Z a-z 0-9 _ -`, como um UUID); outro valor é ignorado.
pub fn connection_id(raw: &str) -> Option<String> {
    let valid = (16..=64).contains(&raw.len())
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    valid.then(|| raw.to_owned())
}

/// Um evento do outbox como sai para os sockets.
#[derive(Debug)]
pub struct Published {
    pub event: Event,
    /// Num aviso, o id da notificação que ele virou no sininho de cada
    /// destinatário (o toast a marca como vista); vazio nos outros eventos.
    pub notifications: HashMap<i64, i64>,
    /// A conexão da aba que pediu a mudança (`ORIGIN_HEADER`), se ela disse.
    pub origin: Option<String>,
}

impl From<Event> for Published {
    fn from(event: Event) -> Self {
        Self {
            event,
            notifications: HashMap::new(),
            origin: None,
        }
    }
}

#[derive(Clone)]
pub struct Realtime(Arc<Inner>);

struct Inner {
    events: broadcast::Sender<Arc<Published>>,
    wake: Notify,
    /// Acorda o relógio do veto: pode haver prazo novo.
    clock: Notify,
    /// Os torneios que os eventos alcançaram desde a última passada do relógio.
    changed: Mutex<Touched>,
    /// O que o relógio sabe dos torneios abertos; uma passada por vez.
    veto: tokio::sync::Mutex<VetoClock>,
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
            changed: Mutex::default(),
            veto: tokio::sync::Mutex::default(),
        }))
    }

    /// Depois de um trecho que emitiu (`outbox::track`): acorda o despachante
    /// e, se os eventos alcançam torneios, o relógio do veto, que relê esses (o
    /// que mudou pode ter criado um prazo: um passo, um jogo novo, um elenco que
    /// deixa os dois lados prontos). Sem ninguém esperando, o aviso fica
    /// guardado para a próxima espera.
    pub fn emitted(&self, touched: Touched) {
        if !touched.any() {
            return;
        }
        self.0.wake.notify_one();
        if touched.reaches_some() {
            self.changed().merge(touched);
            self.0.clock.notify_one();
        }
    }

    /// Para quem emitiu fora de `outbox::track`: acorda o despachante e o
    /// relógio do veto, que relê todos os torneios abertos.
    pub fn wake(&self) {
        self.emitted(Touched::everything());
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

    fn changed(&self) -> std::sync::MutexGuard<'_, Touched> {
        self.0
            .changed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// O relógio do veto, só para uma passada por vez, e o que os eventos
    /// alcançaram desde a anterior (fica vazio: quem tira passa a responder por
    /// ele, e o devolve com `missed` se a passada não foi até o fim).
    pub(super) async fn veto_clock(&self) -> (tokio::sync::MutexGuard<'_, VetoClock>, Touched) {
        let clock = self.0.veto.lock().await;
        let changed = std::mem::take(&mut *self.changed());
        (clock, changed)
    }

    /// A passada do relógio falhou antes de ver tudo: a próxima relê todos.
    pub(super) fn missed(&self) {
        self.changed().merge(Touched::everything());
    }

    /// Os eventos publicados daqui em diante.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Published>> {
        self.0.events.subscribe()
    }

    /// Sem socket nenhum, o evento só não vai para ninguém.
    pub(super) fn publish(&self, published: Published) {
        let _ = self.0.events.send(Arc::new(published));
    }
}

/// Camada de todas as rotas: depois de uma requisição que emitiu evento (com
/// a transação já encerrada), acorda o despachante e, para os torneios que ela
/// alcançou, o relógio do veto. Os eventos levam a conexão de `ORIGIN_HEADER`,
/// se veio.
pub async fn wake_after_emit(
    State(realtime): State<Realtime>,
    request: Request,
    next: Next,
) -> Response {
    let origin = request
        .headers()
        .get(ORIGIN_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(connection_id);
    let (response, touched) = outbox::track(origin, next.run(request)).await;
    realtime.emitted(touched);
    response
}
