//! Notificações: os avisos para pessoas (`outbox::Notice`) guardados um por
//! destinatário, para o sininho do cabeçalho mostrar os que já passaram (o
//! toast some sozinho, e com o site fechado só chega o Web Push) e quantos a
//! pessoa ainda não leu.
//!
//! - O despachante do `realtime` grava cada aviso que publica (`store`, na
//!   transação dele), e o WebSocket manda as não lidas ao abrir
//!   (`unread_count`); cada aviso que chega depois soma uma no front.
//! - `GET /me/notifications`: uma página das mais recentes (20, ou `limit`),
//!   todas, só as lidas ou só as não lidas, com o `next` que vira o `before`
//!   da seguinte (pelo id, que só cresce: a que chega no meio não desloca as
//!   páginas). `POST /me/notifications/read`: abrir a página de um aviso marca os
//!   dela (pelo `path` exato) e "Marcar todas como lidas" marca até um id (a
//!   que chegou depois continua não lida); se alguma mudou, emite
//!   `Event::Unread`, que leva a contagem às outras abas e aparelhos.
//! - Ficam 30 dias (`delete_expired`, pelo agendador, em `retention`) e saem
//!   com a conta excluída (`forget_user`, que `members` chama) e com o torneio
//!   a que levam (trigger `tournaments_path_forgotten`; `unread_under` e
//!   `recount`, que `tournaments` chama, acertam a contagem). O caminho
//!   acompanha a troca de endereço da comunidade (trigger `tenants_slug_moved`).
//!
//! Depende de `auth` (quem vê), do `outbox` (o aviso e o evento) e de `tenants`
//! (o `TenantId`).

mod db;
mod handlers;
mod retention;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use db::{forget_user, recount, store, unread_count, unread_under};
pub use retention::delete_expired;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::notifications))
        .routes(routes!(handlers::mark_read))
}
