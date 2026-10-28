use axum::{Json, extract::State, http::StatusCode};

use super::{
    db, subscription,
    types::{DeleteSubscription, PushKey, PutSubscription},
};
use crate::{
    AppState,
    auth::CurrentUser,
    shared::{ApiError, ErrorBody, JsonBody},
};

#[utoipa::path(
    get,
    path = "/push/key",
    tag = "push",
    responses(
        (status = OK, body = PushKey, description = "Chave pública VAPID para inscrever o navegador"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "push.not_configured (a API está sem as chaves VAPID: só há avisos com o site aberto)"),
    )
)]
pub async fn get_key(
    State(state): State<AppState>,
    _user: CurrentUser,
) -> Result<Json<PushKey>, ApiError> {
    let push = state.push.as_ref().ok_or(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "push.not_configured",
    ))?;
    Ok(Json(PushKey {
        public_key: push.public_key().to_owned(),
    }))
}

#[utoipa::path(
    put,
    path = "/push/subscriptions",
    tag = "push",
    request_body = PutSubscription,
    responses(
        (status = NO_CONTENT, description = "Navegador inscrito para os avisos de quem está logado (o endpoint que era de outra conta passa a ser desta), com o idioma dos avisos"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "push.subscription_invalid (endpoint fora de https num domínio, chaves com outro tamanho), request.invalid_body"),
    )
)]
pub async fn put_subscription(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<PutSubscription>,
) -> Result<StatusCode, ApiError> {
    let (endpoint, p256dh, auth) =
        subscription::validate(&body.endpoint, &body.keys.p256dh, &body.keys.auth)?;
    let keys = (endpoint.as_str(), p256dh.as_str(), auth.as_str());
    db::upsert(&state.db, user.id, keys, body.locale).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/push/subscriptions",
    tag = "push",
    request_body = DeleteSubscription,
    responses(
        (status = NO_CONTENT, description = "Inscrição do navegador removida, se era de quem está logado (também se não havia)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn delete_subscription(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<DeleteSubscription>,
) -> Result<StatusCode, ApiError> {
    db::delete(&state.db, user.id, &body.endpoint).await?;
    Ok(StatusCode::NO_CONTENT)
}
