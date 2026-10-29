//! Rotas de username e senha: entrar, e criar, trocar ou remover o login com
//! senha da conta logada. O cadastro fica em `signup.rs`.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser,
    attempts::{check_password, confirm_password},
    db,
    emails::Notice,
    handlers::sessions_ended,
    kv,
    notify::notify,
    password::{hash_password, is_acceptable, validate_password},
    session::{SESSION_COOKIE, user_agent},
    types::{ChangePassword, Credentials, CurrentPassword, LoginMethods, SignedIn},
    username::validate_username,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, token},
};

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = OK, body = SignedIn, description = "Sessão criada no cookie. Se a exclusão da conta estava agendada, entrar a cancelou (`deletion_cancelled`)"),
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
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
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

/// Sessão nova para `user_id`, no cookie, e o usuário para a resposta.
/// Entrar cancela a exclusão da conta, se estava agendada.
pub(super) async fn start_session(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    user_id: i64,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    let agent = user_agent(headers);
    let token = kv::create_session(&state.redis, user_id, agent.as_deref()).await?;
    let deletion_cancelled = db::cancel_deletion(&state.db, user_id).await?;
    let user = db::user_by_id(&state.db, user_id).await?;
    let signed_in = SignedIn {
        user,
        deletion_cancelled,
    };
    Ok((jar.add(state.auth.session_cookie(token)), Json(signed_in)))
}

#[utoipa::path(
    post,
    path = "/auth/password",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = CREATED, body = LoginMethods, description = "Username e senha criados para a conta logada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.password_exists, auth.email_required (a conta não tem email, que recuperaria a senha), auth.username_taken (por uma conta ou por um cadastro esperando confirmação)"),
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
    refuse_reserved(&state, &username).await?;
    let hash = hash_password(body.password).await?;
    db::add_password(&state.db, user.id, &username, &hash).await?;
    notify(&state, user.id, Notice::PasswordCreated { username });
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
        (status = CONFLICT, body = ErrorBody, description = "auth.no_password, auth.username_taken (por uma conta ou por um cadastro esperando confirmação)"),
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
    if let Some(username) = &username {
        refuse_reserved(&state, username).await?;
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
        let keep = token::hash_token(cookie.value());
        let ended = kv::delete_other_sessions(&state.redis, user.id, &keep).await?;
        sessions_ended(&state, user.id, &ended).await?;
    }
    let methods = db::login_methods(&state.db, user.id).await?;
    let renamed = username.is_some();
    if let Some(notice) = change_notice(methods.username.clone(), renamed, hash.is_some()) {
        notify(&state, user.id, notice);
    }
    Ok(Json(methods))
}

/// O aviso da troca de username e/ou senha (`username` é o de agora), se
/// algo mudou.
fn change_notice(username: Option<String>, renamed: bool, password: bool) -> Option<Notice> {
    let username = username?;
    if password {
        Some(Notice::PasswordChanged { username, renamed })
    } else if renamed {
        Some(Notice::UsernameChanged { username })
    } else {
        None
    }
}

#[utoipa::path(
    delete,
    path = "/auth/password",
    tag = "auth",
    request_body = CurrentPassword,
    responses(
        (status = OK, body = LoginMethods, description = "Username e senha removidos; a conta continua entrando pelo Discord ou pelo Google"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password"),
        (status = CONFLICT, body = ErrorBody, description = "auth.no_password, auth.last_login (a conta não tem Discord nem Google)"),
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
    notify(&state, user.id, Notice::PasswordRemoved);
    Ok(Json(db::login_methods(&state.db, user.id).await?))
}

/// Username reservado por um cadastro esperando confirmação → 409
/// `auth.username_taken`, como se já fosse de uma conta.
async fn refuse_reserved(state: &AppState, username: &str) -> Result<(), ApiError> {
    if kv::username_reserved(&state.redis, username).await? {
        return Err(ApiError::conflict("auth.username_taken"));
    }
    Ok(())
}
