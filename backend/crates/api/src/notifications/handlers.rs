use axum::{Json, extract::State};

use super::{
    db,
    types::{ListQuery, MarkRead, Notification, Unread},
};
use crate::{
    AppState,
    auth::CurrentUser,
    outbox::{self, Event},
    shared::{ApiError, ErrorBody, JsonBody, QueryParams},
};

/// Notificações em cada lista do sininho.
const LIST_LIMIT: i64 = 50;

#[utoipa::path(
    get,
    path = "/me/notifications",
    tag = "notifications",
    params(ListQuery),
    responses(
        (status = OK, body = Vec<Notification>, description = "As 50 notificações mais recentes de quem vê, da mais nova, cada uma dizendo se já foi lida. O filtro vale antes do limite: uma não lida antiga não fica atrás das recentes. Quantas faltam ler chega pelo WebSocket (`unread` ao conectar e ao ler noutra aba; cada `notify` soma uma)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_query"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn notifications(
    State(state): State<AppState>,
    user: CurrentUser,
    QueryParams(query): QueryParams<ListQuery>,
) -> Result<Json<Vec<Notification>>, ApiError> {
    Ok(Json(
        db::recent(&state.db, user.id, query.read, LIST_LIMIT).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/me/notifications/read",
    tag = "notifications",
    request_body = MarkRead,
    responses(
        (status = OK, body = Unread, description = "As notificações até `up_to` (\"Marcar todas como lidas\") ou as do `path` exato (a página que a pessoa abriu) foram lidas, menos as de outra pessoa; devolve quantas ainda faltam, que também chega às outras abas pelo WebSocket (`unread`), se alguma mudou"),
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
    let changed = match body {
        MarkRead::UpTo { up_to } => db::mark_read(&mut tx, user.id, up_to).await?,
        MarkRead::Path { path } => db::mark_read_at_path(&mut tx, user.id, &path).await?,
    };
    let unread = db::unread_count(&mut *tx, user.id).await?;
    // Abrir uma página sem aviso nenhum é o caso comum: nada a avisar às abas.
    if changed > 0 {
        outbox::emit(
            &mut tx,
            Event::Unread {
                user: user.id,
                count: unread,
            },
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Json(Unread { unread }))
}
