use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::outbox::Notice;

/// Quais notificações listar e a partir de onde.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    /// `false`: só as não lidas. `true`: só as lidas. Sem ele, todas.
    pub read: Option<bool>,
    /// Só as de id menor: o `next` da página anterior.
    pub before: Option<i64>,
    /// Quantas, de 1 a 100. Sem ele, 20.
    #[param(minimum = 1, maximum = 100)]
    pub limit: Option<i64>,
}

/// Uma página de notificações, da mais nova.
#[derive(Serialize, ToSchema, Debug)]
pub struct NotificationPage {
    pub notifications: Vec<Notification>,
    /// O `before` da página seguinte; `null` quando não há mais.
    pub next: Option<i64>,
}

/// Um aviso que chegou para quem vê.
#[derive(Serialize, ToSchema, Debug)]
pub struct Notification {
    pub id: i64,
    /// O aviso, como chega pelo WebSocket (`notify`) e pelo Web Push.
    pub notice: Notice,
    pub created_at: DateTime<Utc>,
    /// Se a pessoa já marcou como lida: abrindo a página dele ou com "Marcar
    /// todas como lidas".
    pub read: bool,
}

/// As notificações lidas: até `up_to`, a mais recente que a pessoa tem à vista
/// em "Marcar todas como lidas" (as que chegaram depois continuam não lidas),
/// ou as do `path` exato, a página que ela abriu.
#[derive(Deserialize, ToSchema)]
#[serde(untagged)]
pub enum MarkRead {
    UpTo { up_to: i64 },
    Path { path: String },
}

/// Quantas notificações faltam ler.
#[derive(Serialize, ToSchema)]
pub struct Unread {
    pub unread: i64,
}
