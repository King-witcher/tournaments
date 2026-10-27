use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::{CurrentUser, db, session::SESSION_COOKIE, types::LoginMethods};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, token},
};

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
