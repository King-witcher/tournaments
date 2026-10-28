use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser, db,
    session::SESSION_COOKIE,
    types::{LoginMethods, UpdateMe},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, text::clean_text, token},
};

/// Tamanho máximo do nome exibido: o do apelido, que começa com ele (e o do
/// nome do Discord).
const DISPLAY_NAME_MAX: usize = 32;

#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    responses(
        (status = OK, body = CurrentUser, description = "Usuário logado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn me(user: CurrentUser) -> Json<CurrentUser> {
    Json(user)
}

#[utoipa::path(
    patch,
    path = "/me",
    tag = "auth",
    request_body = UpdateMe,
    responses(
        (status = OK, body = CurrentUser, description = "Nome exibido trocado (os apelidos nas comunidades continuam como estão)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.display_name_invalid, request.invalid_body"),
    )
)]
pub async fn update_me(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<UpdateMe>,
) -> Result<Json<CurrentUser>, ApiError> {
    let name = clean_text(
        &body.display_name,
        DISPLAY_NAME_MAX,
        "auth.display_name_invalid",
    )?;
    db::set_display_name(&state.db, user.id, &name).await?;
    Ok(Json(db::user_by_id(&state.db, user.id).await?))
}

#[utoipa::path(
    get,
    path = "/auth/logins",
    tag = "auth",
    responses(
        (status = OK, body = LoginMethods, description = "Formas de login da conta logada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn logins(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<LoginMethods>, ApiError> {
    Ok(Json(db::login_methods(&state.db, user.id).await?))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses((status = NO_CONTENT, description = "Sessão encerrada (também sem sessão)"))
)]
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Result<Response, ApiError> {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        db::delete_session(&state.db, &token::hash_token(cookie.value())).await?;
    }
    let jar = jar.remove(state.auth.clear_session_cookie());
    Ok((jar, StatusCode::NO_CONTENT).into_response())
}
