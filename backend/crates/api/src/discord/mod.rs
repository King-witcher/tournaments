//! Avisos da comunidade no Discord por webhook: o owner cola o endereço do
//! webhook de um canal (`https://discord.com/api/webhooks/{id}/{token}`) nas
//! configurações da comunidade, escolhe os avisos, o idioma e se quem
//! vinculou o Discord é mencionado, e o site posta lá. Nada de bot nem DM.
//!
//! - `handlers`/`types`: `GET`/`PUT`/`DELETE /tenants/{slug}/discord` e
//!   `POST .../test`, só do owner. O endereço é credencial: vai para o banco
//!   (`db`) separado em id e token e nunca volta pela API nem vai para o log.
//! - `url`: a única forma de endereço aceita (só os hosts do Discord).
//! - `catalog`: os avisos que existem (o código é o do `Notice`/`Announce` do
//!   outbox), se vêm ligados, a quem se dirigem e o texto de cada um em
//!   português e espanhol. Aviso novo = uma linha lá.
//! - `queue`: o despachante do `realtime` chama `enqueue` para cada evento do
//!   outbox, e uma tarefa própria (`spawn`) esvazia a fila em ordem,
//!   respeitando o ritmo do Discord (`sender`); `render` monta o corpo.
//!   Webhook apagado (401/404) fica `broken` e para de receber.
//!
//! Depende de `tenants` (o extractor e o `TenantId`); o `realtime` e o
//! `main` usam `enqueue`, `deliver_due` e `spawn`.

mod catalog;
mod db;
mod handlers;
mod queue;
mod render;
mod sender;
mod types;
mod url;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use queue::{deliver_due, enqueue, spawn};
pub use sender::Webhooks;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            handlers::get_discord,
            handlers::put_discord,
            handlers::delete_discord
        ))
        .routes(routes!(handlers::test_discord))
}
