use axum::{Json, extract::State, http::StatusCode};

use super::{
    db,
    types::{ListQuery, MarkRead, NotificationPage, Unread},
};
use crate::{
    AppState,
    auth::CurrentUser,
    outbox::{self, Event},
    shared::{ApiError, ErrorBody, JsonBody, QueryParams},
};

/// Notificações numa página do sininho, sem `limit`.
const PAGE_SIZE: i64 = 20;
/// O maior `limit`: o sininho, ao recarregar a lista aberta, pede de uma vez
/// as páginas que já mostrava.
const MAX_PAGE_SIZE: i64 = 100;

#[utoipa::path(
    get,
    path = "/me/notifications",
    tag = "notifications",
    params(ListQuery),
    responses(
        (status = OK, body = NotificationPage, description = "Uma página das notificações de quem vê, da mais nova, cada uma dizendo se já foi lida, e o `before` da seguinte. O filtro vale antes do limite: uma não lida antiga não fica atrás das recentes. Quantas faltam ler chega pelo WebSocket (`unread` ao conectar e ao ler noutra aba; cada `notify` soma uma)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_query"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn notifications(
    State(state): State<AppState>,
    user: CurrentUser,
    QueryParams(query): QueryParams<ListQuery>,
) -> Result<Json<NotificationPage>, ApiError> {
    let limit = query.limit.unwrap_or(PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&limit) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "request.invalid_query",
        ));
    }
    Ok(Json(
        db::page(&state.db, user.id, query.read, query.before, limit).await?,
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
