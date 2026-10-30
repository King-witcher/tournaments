//! Notificações: os avisos para pessoas (`outbox::Notice`) guardados um por
//! destinatário, para o sininho do cabeçalho mostrar os que já passaram (o
//! toast some sozinho, e com o site fechado só chega o Web Push) e quantos a
//! pessoa ainda não viu.
//!
//! - O despachante do `realtime` grava cada aviso que publica (`store`, na
//!   transação dele), e o WebSocket manda as não lidas ao abrir
//!   (`unread_count`); cada aviso que chega depois soma uma no front.
//! - `GET /me/notifications`: as mais recentes. `POST /me/notifications/read`:
//!   abrir o sininho marca como vistas as da lista (até um id: a que chegou
//!   depois continua nova) e emite `Event::Unread`, que leva a contagem às
//!   outras abas e aparelhos.
//! - Ficam 30 dias (`delete_expired`, pelo agendador, em `retention`) e saem
//!   com a conta excluída (`forget_user`, que `members` chama).
//!
//! Depende de `auth` (quem vê) e do `outbox` (o aviso e o evento).

mod db;
mod handlers;
mod retention;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use db::{forget_user, store, unread_count};
pub use retention::delete_expired;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::notifications))
        .routes(routes!(handlers::mark_read))
}
