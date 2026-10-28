//! Email da conta logada: pôr ou trocar. O email novo só vale depois que o
//! link mandado para ele é aberto; até lá, fica o de antes.

use axum::{Json, extract::State, http::StatusCode};

use super::{
    CurrentUser,
    attempts::confirm_password,
    db,
    emails::{self, LINK_TTL_SECS},
    kv,
    types::{ChangeEmail, EmailSent, LinkToken},
};
use crate::{
    AppState, mail,
    shared::{ApiError, ErrorBody, JsonBody, Locale},
};

#[utoipa::path(
    put,
    path = "/auth/email",
    tag = "auth",
    request_body = ChangeEmail,
    responses(
        (status = ACCEPTED, body = EmailSent, description = "Link de confirmação enviado ao email novo (vale 1 hora; um pedido novo derruba o anterior). O email da conta só muda quando ele é aberto"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_unchanged, auth.email_taken (é de outra conta)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}, email.too_soon {seconds}"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "email.unavailable (teto de envios do dia)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "email.invalid, email.disposable, request.invalid_body"),
    )
)]
pub async fn change_email(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<ChangeEmail>,
) -> Result<(StatusCode, Json<EmailSent>), ApiError> {
    let email = mail::normalize(&body.email)?;
    let methods = db::login_methods(&state.db, user.id).await?;
    if methods.email.as_deref() == Some(email.as_str()) {
        return Err(ApiError::conflict("auth.email_unchanged"));
    }
    // Com senha, quem pegou a sessão não troca o email para depois recuperar
    // a senha por ele.
    if methods.username.is_some() {
        let current = body
            .current_password
            .ok_or(ApiError::forbidden("auth.wrong_password"))?;
        confirm_password(&state, user.id, current, "auth.no_password").await?;
    }
    if db::email_taken(&state.db, &email).await? {
        return Err(ApiError::conflict("auth.email_taken"));
    }

    let ticket = state.mail.reserve(&state.redis, &email).await?;
    let token = kv::create_email_change(&state.redis, user.id, &email, LINK_TTL_SECS).await?;
    let locale = Locale::from_db(&user.locale)?;
    ticket.send(emails::email_change(
        &state.auth.web_origin,
        locale,
        &email,
        &token,
    ));
    Ok((StatusCode::ACCEPTED, Json(EmailSent { email })))
}

#[utoipa::path(
    post,
    path = "/auth/email/confirm",
    tag = "auth",
    request_body = LinkToken,
    responses(
        (status = NO_CONTENT, description = "Email da conta trocado. Não pede login: o link prova que quem abriu recebe no email novo"),
        (status = GONE, body = ErrorBody, description = "auth.link_expired (expirou, já foi usado ou foi trocado por um mais novo)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_taken (outra conta confirmou este email antes)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn confirm_email(
    State(state): State<AppState>,
    JsonBody(body): JsonBody<LinkToken>,
) -> Result<StatusCode, ApiError> {
    let (user_id, email) = kv::take_email_change(&state.redis, &body.token)
        .await?
        .ok_or(ApiError::gone("auth.link_expired"))?;
    db::set_email(&state.db, user_id, &email).await?;
    Ok(StatusCode::NO_CONTENT)
}
