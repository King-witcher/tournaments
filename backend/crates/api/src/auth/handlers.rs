use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser, db, kv,
    session::SESSION_COOKIE,
    types::{LoginMethods, SessionInfo, SessionPath, UpdateMe},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams, text::clean_text, token},
};

/// Tamanho máximo do nome exibido: o do apelido, que começa com ele (e o do
/// nome do Discord).
const DISPLAY_NAME_MAX: usize = 32;

/// Caracteres do hash (em hexadecimal) que identificam a sessão na lista: 64
/// bits, que não se repetem entre as sessões de uma conta.
const SESSION_ID_LEN: usize = 16;

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
        kv::delete_session(&state.redis, &token::hash_token(cookie.value())).await?;
    }
    let jar = jar.remove(state.auth.clear_session_cookie());
    Ok((jar, StatusCode::NO_CONTENT).into_response())
}

/// Hash (em hexadecimal) da sessão do cookie.
fn current_hash(jar: &CookieJar) -> Option<String> {
    jar.get(SESSION_COOKIE)
        .map(|c| token::hex(&token::hash_token(c.value())))
}

#[utoipa::path(
    get,
    path = "/auth/sessions",
    tag = "auth",
    responses(
        (status = OK, body = Vec<SessionInfo>, description = "Sessões abertas da conta, da mais nova para a mais antiga"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn sessions(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
) -> Result<Json<Vec<SessionInfo>>, ApiError> {
    let current = current_hash(&jar);
    let sessions = kv::user_sessions(&state.redis, user.id).await?;
    Ok(Json(
        sessions
            .into_iter()
            .map(|s| SessionInfo {
                id: s.hash[..SESSION_ID_LEN].to_owned(),
                current: current.as_ref() == Some(&s.hash),
                created_at: s.created_at,
                expires_at: s.expires_at,
                user_agent: s.user_agent,
            })
            .collect(),
    ))
}

#[utoipa::path(
    delete,
    path = "/auth/sessions",
    tag = "auth",
    responses(
        (status = NO_CONTENT, description = "As outras sessões da conta foram encerradas; a desta requisição continua"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn end_other_sessions(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
) -> Result<StatusCode, ApiError> {
    let cookie = jar.get(SESSION_COOKIE).ok_or(ApiError::unauthenticated())?;
    kv::delete_other_sessions(&state.redis, user.id, &token::hash_token(cookie.value())).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/auth/sessions/{id}",
    tag = "auth",
    params(("id" = String, Path, description = "id de GET /auth/sessions")),
    responses(
        (status = NO_CONTENT, description = "Sessão encerrada. Se era a desta requisição, o cookie sai junto, como no logout"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "auth.session_not_found"),
    )
)]
pub async fn end_session(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    PathParams(path): PathParams<SessionPath>,
) -> Result<Response, ApiError> {
    let hash = kv::session_hashes(&state.redis, user.id)
        .await?
        .into_iter()
        .find(|h| path.id.len() == SESSION_ID_LEN && h.starts_with(&path.id))
        .ok_or(ApiError::not_found("auth.session_not_found"))?;
    kv::delete_sessions(&state.redis, user.id, std::slice::from_ref(&hash)).await?;
    let jar = if current_hash(&jar) == Some(hash) {
        jar.remove(state.auth.clear_session_cookie())
    } else {
        jar
    };
    Ok((jar, StatusCode::NO_CONTENT).into_response())
}
