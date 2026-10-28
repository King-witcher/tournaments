//! Rotas de username e senha: cadastro, entrar, e criar, trocar ou remover o
//! login com senha da conta logada.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser,
    attempts::{check_password, confirm_password},
    db, kv,
    password::{hash_password, is_acceptable, validate_password},
    session::{SESSION_COOKIE, user_agent},
    types::{ChangePassword, Credentials, CurrentPassword, LoginMethods},
    username::validate_username,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, token},
};

#[utoipa::path(
    post,
    path = "/auth/signup",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = CREATED, body = CurrentUser, description = "Conta criada, já com a sessão no cookie. O nome exibido é o username"),
        (status = CONFLICT, body = ErrorBody, description = "auth.username_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.username_invalid, auth.password_invalid, request.invalid_body"),
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<Credentials>,
) -> Result<(StatusCode, CookieJar, Json<CurrentUser>), ApiError> {
    let username = validate_username(&body.username)?;
    validate_password(&body.password)?;
    let hash = hash_password(body.password).await?;
    let user_id = db::create_password_user(&state.db, &username, &hash).await?;
    let (jar, user) = start_session(&state, &headers, jar, user_id).await?;
    Ok((StatusCode::CREATED, jar, user))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = OK, body = CurrentUser, description = "Sessão criada no cookie"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.invalid_credentials (username ou senha errados)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<Credentials>,
) -> Result<(CookieJar, Json<CurrentUser>), ApiError> {
    let invalid = ApiError::new(StatusCode::UNAUTHORIZED, "auth.invalid_credentials");
    // Fora do formato, a conta não pode existir: nem conta tentativa.
    let Ok(username) = validate_username(&body.username) else {
        return Err(invalid);
    };
    if !is_acceptable(&body.password) {
        return Err(invalid);
    }

    let found = db::password_login_by_username(&state.db, &username).await?;
    // Sem conta, confere contra um hash fixo: mesmo tempo e mesma contagem.
    let hash = found.as_ref().map(|f| f.password_hash.clone());
    let ok = check_password(&state.redis, &username, hash, body.password).await?;
    match found {
        Some(login) if ok => start_session(&state, &headers, jar, login.user_id).await,
        _ => Err(invalid),
    }
}

async fn start_session(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    user_id: i64,
) -> Result<(CookieJar, Json<CurrentUser>), ApiError> {
    let agent = user_agent(headers);
    let token = kv::create_session(&state.redis, user_id, agent.as_deref()).await?;
    let user = db::user_by_id(&state.db, user_id).await?;
    Ok((jar.add(state.auth.session_cookie(token)), Json(user)))
}

#[utoipa::path(
    post,
    path = "/auth/password",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = CREATED, body = LoginMethods, description = "Username e senha criados para a conta logada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.password_exists, auth.username_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.username_invalid, auth.password_invalid, request.invalid_body"),
    )
)]
pub async fn create_password(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<Credentials>,
) -> Result<(StatusCode, Json<LoginMethods>), ApiError> {
    let username = validate_username(&body.username)?;
    validate_password(&body.password)?;
    let hash = hash_password(body.password).await?;
    db::add_password(&state.db, user.id, &username, &hash).await?;
    let methods = db::login_methods(&state.db, user.id).await?;
    Ok((StatusCode::CREATED, Json(methods)))
}

#[utoipa::path(
    patch,
    path = "/auth/password",
    tag = "auth",
    request_body = ChangePassword,
    responses(
        (status = OK, body = LoginMethods, description = "Username e/ou senha trocados. Trocar a senha encerra as outras sessões"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password"),
        (status = CONFLICT, body = ErrorBody, description = "auth.no_password, auth.username_taken"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.username_invalid, auth.password_invalid, request.invalid_body"),
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    JsonBody(body): JsonBody<ChangePassword>,
) -> Result<Json<LoginMethods>, ApiError> {
    let username = body
        .username
        .as_deref()
        .map(validate_username)
        .transpose()?;
    if let Some(password) = &body.password {
        validate_password(password)?;
    }
    confirm_password(&state, user.id, body.current_password, "auth.no_password").await?;

    let hash = match body.password {
        Some(password) => Some(hash_password(password).await?),
        None => None,
    };
    db::update_password(&state.db, user.id, username.as_deref(), hash.as_deref()).await?;
    if hash.is_some()
        && let Some(cookie) = jar.get(SESSION_COOKIE)
    {
        kv::delete_other_sessions(&state.redis, user.id, &token::hash_token(cookie.value()))
            .await?;
    }
    Ok(Json(db::login_methods(&state.db, user.id).await?))
}

#[utoipa::path(
    delete,
    path = "/auth/password",
    tag = "auth",
    request_body = CurrentPassword,
    responses(
        (status = OK, body = LoginMethods, description = "Username e senha removidos; a conta fica só com o Discord"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password"),
        (status = CONFLICT, body = ErrorBody, description = "auth.no_password, auth.last_login (a conta não tem Discord)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn remove_password(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<CurrentPassword>,
) -> Result<Json<LoginMethods>, ApiError> {
    confirm_password(&state, user.id, body.current_password, "auth.no_password").await?;
    db::remove_password(&state.db, user.id).await?;
    Ok(Json(db::login_methods(&state.db, user.id).await?))
}
