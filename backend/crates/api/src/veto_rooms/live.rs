//! A sala em memória, numa instância só: o canal de cada sala, com os limites
//! de conexão, e a visão que o espectador lê sem ir ao banco.
//!
//! - **Canal por sala** (`join`, `changed`): só os sockets daquela sala acordam
//!   com a mudança dela. O aviso sai depois do commit; o que se perde numa queda
//!   volta com a recarga da reconexão.
//! - **Limites**: o espectador é quem pode vir em massa (o link num chat de
//!   transmissão), e tem teto por sala e no total; quem joga tem teto por sala,
//!   maior que os lugares com várias abas. Passou do teto, o socket é recusado,
//!   e a página recarrega de tempos em tempos.
//! - **Visão do espectador** (`view`, `keep_view`): igual para todos os
//!   espectadores, então uma leitura do banco por mudança serve a todos, por
//!   muitos que sejam, em vez de uma por espectador. Só entra aqui o token que já
//!   se provou de espectador no banco, então lixo no caminho não ocupa memória.

use std::{
    collections::HashMap,
    future::Future,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use tokio::sync::{Notify, broadcast};

use super::types::VetoRoomDetail;
use crate::shared::ApiError;

/// Espectadores conectados por sala.
const MAX_SPECTATORS_PER_ROOM: usize = 200;
/// Espectadores conectados em todas as salas.
const MAX_SPECTATORS: usize = 1_000;
/// Abas de quem joga e da gestão por sala: 9 links, com folga para mais de uma
/// aba e aparelho.
const MAX_PLAYERS_PER_ROOM: usize = 32;
/// Sockets de sala no total, para a memória da instância (no Render, 512 MB).
const MAX_SOCKETS: usize = 3_000;
/// Mudanças que um socket lento pode ficar devendo: ele só recarrega uma vez.
const CAPACITY: usize = 8;
/// Salas com a visão do espectador guardada; no teto, começa de novo.
const MAX_VIEWS: usize = 256;

/// Uma mudança na sala.
#[derive(Clone, Debug)]
pub struct Change {
    /// A aba que pediu (`x-realtime-connection`), que já aplicou a resposta.
    pub origin: Option<String>,
}

/// Clonar é barato; os clones dividem tudo.
#[derive(Clone, Default)]
pub struct Live(Arc<Inner>);

#[derive(Default)]
struct Inner {
    sockets: Mutex<Sockets>,
    views: Mutex<HashMap<String, Arc<SpectatorView>>>,
    /// Quantas mudanças houve, em qualquer sala (`stamp`).
    changes: AtomicU64,
    clock: Notify,
}

#[derive(Default)]
struct Sockets {
    rooms: HashMap<i64, Channel>,
    total: usize,
    spectators: usize,
}

struct Channel {
    sender: broadcast::Sender<Change>,
    players: usize,
    spectators: usize,
}

/// Um socket contado na sala. Sai da conta ao sair de escopo.
pub struct RoomSocket {
    live: Live,
    room: i64,
    spectator: bool,
    pub changes: broadcast::Receiver<Change>,
}

impl Drop for RoomSocket {
    fn drop(&mut self) {
        let mut sockets = self.live.0.sockets.lock().expect("trava dos sockets");
        sockets.total -= 1;
        if self.spectator {
            sockets.spectators -= 1;
        }
        if let Some(channel) = sockets.rooms.get_mut(&self.room) {
            if self.spectator {
                channel.spectators -= 1;
            } else {
                channel.players -= 1;
            }
            if channel.players + channel.spectators == 0 {
                sockets.rooms.remove(&self.room);
            }
        }
    }
}

impl Live {
    /// Conta mais um socket na sala `room`. 503
    /// `veto_room.too_many_connections` passando de um teto.
    pub fn join(&self, room: i64, spectator: bool) -> Result<RoomSocket, ApiError> {
        let mut sockets = self.0.sockets.lock().expect("trava dos sockets");
        let full = sockets.total >= MAX_SOCKETS
            || (spectator && sockets.spectators >= MAX_SPECTATORS)
            || sockets.rooms.get(&room).is_some_and(|c| {
                if spectator {
                    c.spectators >= MAX_SPECTATORS_PER_ROOM
                } else {
                    c.players >= MAX_PLAYERS_PER_ROOM
                }
            });
        if full {
            tracing::warn!(room, spectator, "teto de sockets de sala de veto");
            return Err(ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "veto_room.too_many_connections",
            ));
        }
        sockets.total += 1;
        if spectator {
            sockets.spectators += 1;
        }
        let channel = sockets.rooms.entry(room).or_insert_with(|| Channel {
            sender: broadcast::channel(CAPACITY).0,
            players: 0,
            spectators: 0,
        });
        if spectator {
            channel.spectators += 1;
        } else {
            channel.players += 1;
        }
        Ok(RoomSocket {
            live: self.clone(),
            room,
            spectator,
            changes: channel.sender.subscribe(),
        })
    }

    /// A sala `room` mudou (depois do commit): a visão guardada não vale mais,
    /// os sockets dela recarregam e o relógio confere o próximo prazo.
    pub fn changed(&self, room: i64, origin: Option<String>) {
        self.0.changes.fetch_add(1, Ordering::SeqCst);
        for view in self.0.views.lock().expect("trava das visões").values() {
            if view.room == room {
                view.generation.fetch_add(1, Ordering::SeqCst);
            }
        }
        let sockets = self.0.sockets.lock().expect("trava dos sockets");
        if let Some(channel) = sockets.rooms.get(&room) {
            let _ = channel.sender.send(Change { origin });
        }
        self.0.clock.notify_one();
    }

    /// Espera alguma sala mudar (o relógio dos prazos dorme nisto).
    pub async fn clock_woken(&self) {
        self.0.clock.notified().await;
    }

    /// A visão guardada do espectador `token`, se ele já se provou um.
    pub(super) fn view(&self, token: &str) -> Option<Arc<SpectatorView>> {
        self.0
            .views
            .lock()
            .expect("trava das visões")
            .get(token)
            .cloned()
    }

    /// A marca a pegar antes de ler uma sala que vai para `keep_view`.
    pub(super) fn stamp(&self) -> u64 {
        self.0.changes.load(Ordering::SeqCst)
    }

    /// Guarda `detail` como a visão do espectador `token`, lida depois de
    /// `stamp`. Com qualquer mudança desde então, não guarda: a sala pode ter
    /// mudado depois da leitura, e nada mais avisaria esta visão. A leitura
    /// seguinte tenta de novo.
    pub(super) fn keep_view(&self, token: String, room: i64, stamp: u64, detail: &VetoRoomDetail) {
        self.keep(token, room, stamp, Kept::of(0, detail));
    }

    fn keep(&self, token: String, room: i64, stamp: u64, kept: Kept) {
        let mut views = self.0.views.lock().expect("trava das visões");
        if self.stamp() != stamp {
            return;
        }
        if views.len() >= MAX_VIEWS {
            views.clear();
        }
        let view = SpectatorView {
            room,
            generation: AtomicU64::new(0),
            body: tokio::sync::Mutex::new(Some(kept)),
        };
        views.insert(token, Arc::new(view));
    }
}

/// O que o espectador de uma sala lê, enquanto nada muda.
pub(super) struct SpectatorView {
    room: i64,
    generation: AtomicU64,
    /// A trava fica presa durante a leitura do banco: uma por vez, e quem
    /// esperava encontra a visão pronta.
    body: tokio::sync::Mutex<Option<Kept>>,
}

struct Kept {
    generation: u64,
    expires_at: DateTime<Utc>,
    /// A resposta sem `now`, que sai na hora.
    body: Arc<Map<String, Value>>,
}

impl Kept {
    fn of(generation: u64, detail: &VetoRoomDetail) -> Self {
        let mut body = match serde_json::to_value(detail) {
            Ok(Value::Object(body)) => body,
            _ => Map::new(),
        };
        body.remove("now");
        Self {
            generation,
            expires_at: detail.expires_at,
            body: Arc::new(body),
        }
    }
}

impl SpectatorView {
    /// A resposta de agora: a guardada, se nada mudou desde que foi lida; senão,
    /// a de `load`, que passa a ser a guardada. A geração é lida antes da
    /// leitura: uma mudança no meio dela faz a próxima ler de novo.
    pub async fn current<F, Fut>(&self, load: F) -> Result<Arc<Map<String, Value>>, ApiError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<VetoRoomDetail, ApiError>>,
    {
        let mut kept = self.body.lock().await;
        let generation = self.generation.load(Ordering::SeqCst);
        if let Some(k) = kept
            .as_ref()
            .filter(|k| k.generation == generation && k.expires_at > Utc::now())
        {
            return Ok(k.body.clone());
        }
        let fresh = Kept::of(generation, &load().await?);
        let body = fresh.body.clone();
        *kept = Some(fresh);
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(result: Result<RoomSocket, ApiError>) -> &'static str {
        result.map_or_else(|e| e.code, |_| "ok")
    }

    #[test]
    fn a_full_room_refuses_only_its_kind_and_a_closed_socket_frees_the_place() {
        let live = Live::default();
        let mut spectators: Vec<RoomSocket> = (0..MAX_SPECTATORS_PER_ROOM)
            .map(|_| live.join(1, true).unwrap())
            .collect();
        assert_eq!(code(live.join(1, true)), "veto_room.too_many_connections");
        // Quem joga continua entrando; outra sala também.
        let player = live.join(1, false).unwrap();
        let other = live.join(2, true).unwrap();
        spectators.pop();
        assert_eq!(code(live.join(1, true)), "ok");
        drop((spectators, player, other));
        let sockets = live.0.sockets.lock().unwrap();
        assert_eq!((sockets.total, sockets.spectators), (0, 0));
        assert!(sockets.rooms.is_empty());
    }

    #[test]
    fn the_total_of_spectators_is_capped_but_players_still_get_in() {
        let live = Live::default();
        let held: Vec<RoomSocket> = (0..MAX_SPECTATORS)
            .map(|n| {
                live.join((n / MAX_SPECTATORS_PER_ROOM) as i64, true)
                    .unwrap()
            })
            .collect();
        assert_eq!(
            code(live.join(i64::MAX, true)),
            "veto_room.too_many_connections"
        );
        assert_eq!(code(live.join(i64::MAX, false)), "ok");
        drop(held);
    }

    fn kept() -> Kept {
        Kept {
            generation: 0,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            body: Arc::new(Map::new()),
        }
    }

    #[test]
    fn a_view_read_before_a_change_is_not_kept() {
        let live = Live::default();
        let stamp = live.stamp();
        live.changed(1, None);
        live.keep("t".into(), 1, stamp, kept());
        assert!(live.view("t").is_none());
        live.keep("t".into(), 1, live.stamp(), kept());
        assert!(live.view("t").is_some());
    }

    #[test]
    fn views_are_capped() {
        let live = Live::default();
        for n in 0..=MAX_VIEWS {
            live.keep(n.to_string(), 1, live.stamp(), kept());
        }
        assert_eq!(live.0.views.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn a_change_reaches_only_the_sockets_of_that_room() {
        let live = Live::default();
        let mut one = live.join(1, false).unwrap();
        let mut two = live.join(2, false).unwrap();
        live.changed(1, Some("aba".into()));
        assert_eq!(
            one.changes.recv().await.unwrap().origin.as_deref(),
            Some("aba")
        );
        assert!(two.changes.try_recv().is_err());
    }
}
