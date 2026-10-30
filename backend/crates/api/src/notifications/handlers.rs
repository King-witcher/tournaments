use axum::{Json, extract::State};

use super::{
    db,
    types::{MarkRead, Notification, Unread},
};
use crate::{
    AppState,
    auth::CurrentUser,
    outbox::{self, Event},
    shared::{ApiError, ErrorBody, JsonBody},
};

/// Notificações na lista do sininho (as de 30 dias, que é o que fica).
const LIST_LIMIT: i64 = 50;

#[utoipa::path(
    get,
    path = "/me/notifications",
    tag = "notifications",
    responses(
        (status = OK, body = Vec<Notification>, description = "As 50 notificações mais recentes de quem vê (ficam 30 dias), da mais nova, cada uma dizendo se já foi vista. Quantas faltam ver chega pelo WebSocket (`unread` ao conectar e ao ver noutra aba; cada `notify` soma uma)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn notifications(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<Notification>>, ApiError> {
    Ok(Json(db::recent(&state.db, user.id, LIST_LIMIT).await?))
}

#[utoipa::path(
    post,
    path = "/me/notifications/read",
    tag = "notifications",
    request_body = MarkRead,
    responses(
        (status = OK, body = Unread, description = "As notificações até `up_to` (as da lista aberta) foram vistas; devolve quantas ainda faltam, que também chega às outras abas pelo WebSocket (`unread`)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn mark_read(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<MarkRead>,
) -> Result<Json<Unread>, ApiError> {
    let mut tx = state.db.begin().await?;
    let unread = db::mark_read(&mut tx, user.id, body.up_to).await?;
    outbox::emit(
        &mut tx,
        Event::Unread {
            user: user.id,
            count: unread,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(Json(Unread { unread }))
}
