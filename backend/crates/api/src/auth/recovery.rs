//! Recuperação da senha pelo email da conta: o pedido manda o link, e o link
//! leva a uma senha nova. O pedido responde igual exista a conta ou não.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;

use super::{
    credentials::start_session,
    db::{self, AccountContact},
    emails::{self, LINK_TTL_SECS, Notice},
    handlers::sessions_ended,
    kv,
    notify::notify,
    password::{hash_password, validate_password},
    types::{PasswordReset, PasswordResetRequest, SignedIn},
    username::{username_key, validate_username},
};
use crate::{
    AppState, mail,
    shared::{ApiError, ErrorBody, JsonBody, Locale},
};

#[utoipa::path(
    post,
    path = "/auth/password/reset",
    tag = "auth",
    request_body = PasswordResetRequest,
    responses(
        (status = ACCEPTED, description = "Se a conta existe e tem senha e email, o link foi para o email dela (vale 1 hora; um pedido novo derruba o anterior). A resposta é a mesma em todos os casos, até fora dos limites de envio, para não revelar quem tem conta"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn request_password_reset(
    State(state): State<AppState>,
    JsonBody(body): JsonBody<PasswordResetRequest>,
) -> Result<StatusCode, ApiError> {
    let login = body.login.trim();
    // Fora do formato, a conta não pode existir.
    let contact = if login.contains('@') {
        match mail::normalize(login) {
            Ok(email) => db::contact_by_email(&state.db, &email).await?,
            Err(_) => None,
        }
    } else {
        match validate_username(login) {
            Ok(username) => db::contact_by_username(&state.db, &username).await?,
            Err(_) => None,
        }
    };
    if let Some(contact) = contact {
        // Em segundo plano: a resposta sai no mesmo tempo, exista a conta ou não.
        tokio::spawn(async move {
            if let Err(err) = send_reset_link(&state, contact).await {
                tracing::error!(error = ?err, "link de recuperação de senha falhou");
            }
        });
    }
    Ok(StatusCode::ACCEPTED)
}

/// Manda o link, se a conta tem senha e email e o endereço está dentro dos
/// limites de envio.
async fn send_reset_link(state: &AppState, contact: AccountContact) -> Result<(), ApiError> {
    let AccountContact {
        user_id,
        email: Some(email),
        username: Some(username),
        locale,
    } = contact
    else {
        return Ok(());
    };
    let Some(ticket) = state.mail.reserve_quietly(&state.redis, &email).await? else {
        return Ok(());
    };
    let token = kv::create_password_reset(&state.redis, user_id, LINK_TTL_SECS).await?;
    let locale = Locale::from_db(&locale)?;
    let web = &state.auth.web_origin;
    ticket.send(emails::password_reset(
        web, locale, &email, &username, &token,
    ));
    Ok(())
}

#[utoipa::path(
    post,
    path = "/auth/password/reset/confirm",
    tag = "auth",
    request_body = PasswordReset,
    responses(
        (status = OK, body = SignedIn, description = "Senha trocada. Todas as sessões da conta foram encerradas, e esta entra numa nova (cookie); se a exclusão da conta estava agendada, entrar a cancelou"),
        (status = GONE, body = ErrorBody, description = "auth.link_expired (expirou, já foi usado ou foi trocado por um mais novo)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.no_password (a senha foi removida depois do pedido)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.password_invalid, request.invalid_body"),
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<PasswordReset>,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    // Antes de gastar o link: senha fora do formato não o consome.
    validate_password(&body.password)?;
    let user_id = kv::take_password_reset(&state.redis, &body.token)
        .await?
        .ok_or(ApiError::gone("auth.link_expired"))?;
    let hash = hash_password(body.password).await?;
    db::update_password(&state.db, user_id, None, Some(&hash)).await?;

    // Quem tinha a senha antiga pode estar logado: todas as sessões saem, e
    // os erros de senha do username, que eram contra a antiga, também. E o
    // link de troca de email que ele tenha pedido: aberto, poria o email dele
    // na conta, e a recuperação seguinte iria para lá.
    kv::cancel_email_change(&state.redis, user_id).await?;
    let ended = kv::delete_all_sessions(&state.redis, user_id).await?;
    sessions_ended(&state, user_id, &ended).await?;
    if let Some(login) = db::password_login_of_user(&state.db, user_id).await? {
        kv::clear_password_failures(&state.redis, &username_key(&login.username)).await?;
        let username = login.username;
        notify(&state, user_id, Notice::PasswordReset { username });
    }
    start_session(&state, &headers, jar, user_id).await
}
