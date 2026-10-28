//! Web Push: a chave pública VAPID (`GET /push/key`), as inscrições dos
//! navegadores de quem está logado (`/push/subscriptions`, uma por endpoint,
//! com o idioma dos avisos) e o envio de um aviso (`deliver`), que o
//! despachante do `realtime` chama para cada `Notice` do outbox. O envio fica
//! atrás da trait `WebPush` (`HttpWebPush` com o `web-push-native` e o
//! `reqwest`); sem as chaves VAPID, o push fica desligado e os avisos só
//! chegam pelo WebSocket.

mod db;
mod delivery;
mod handlers;
mod sender;
mod subscription;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use delivery::deliver;
pub use sender::{HttpWebPush, Sent, Target, WebPush};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::get_key))
        .routes(routes!(
            handlers::put_subscription,
            handlers::delete_subscription
        ))
}
