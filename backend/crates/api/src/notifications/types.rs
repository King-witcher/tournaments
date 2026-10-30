use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::outbox::Notice;

/// Um aviso que chegou para quem vê.
#[derive(Serialize, ToSchema, Debug)]
pub struct Notification {
    pub id: i64,
    /// O aviso, como chega pelo WebSocket (`notify`) e pelo Web Push.
    pub notice: Notice,
    pub created_at: DateTime<Utc>,
    /// Se a pessoa já abriu o sininho com ele na lista.
    pub read: bool,
}

/// As notificações vistas: as da lista que a pessoa abriu.
#[derive(Deserialize, ToSchema)]
pub struct MarkRead {
    /// O id da mais recente da lista: as que chegaram depois continuam novas.
    pub up_to: i64,
}

/// Quantas notificações faltam ver.
#[derive(Serialize, ToSchema)]
pub struct Unread {
    pub unread: i64,
}
