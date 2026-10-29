//! Outbox: os eventos de mudança que quem muda o estado grava na própria
//! transação (`emit`), e que o despachante do `realtime` lê depois do commit
//! (`claim`), publica no WebSocket e no Web Push e apaga (`remove`). Só
//! depende de `shared`, para todo domínio poder emitir (inclusive `auth`, que
//! fecha os sockets de uma sessão encerrada); cada domínio monta os eventos
//! dele (`tenants::changed`, `tournaments::changed`, ...). `track` diz se uma
//! requisição emitiu, para a camada do `lib.rs` acordar o despachante.

mod db;
mod event;
mod tracking;

pub use db::{Stored, claim, emit, remove};
pub use event::{Announce, AnnounceKind, Event, Notice, NoticeKind, Placement};
pub use tracking::track;
