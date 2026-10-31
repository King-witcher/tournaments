//! Email da conta logada: pôr, trocar ou tirar. O email novo só vale depois
//! que o link mandado para ele é aberto; até lá, fica o de antes. Só a conta
//! sem senha fica sem email. O pedido não diz se o email já é de outra conta
//! (o dono dela recebe um aviso no lugar do link).

use axum::{Json, extract::State, http::StatusCode};

use super::{
    CurrentUser,
    attempts::confirm_password,
    db,
    emails::{self, LINK_TTL_SECS, Notice},
    kv,
    notify::notify,
    types::{ChangeEmail, EmailSent, LinkToken, LoginMethods},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, Locale},
};

#[utoipa::path(
    put,
    path = "/auth/email",
    tag = "auth",
    request_body = ChangeEmail,
    responses(
        (status = ACCEPTED, body = EmailSent, description = "Link de confirmação enviado ao email novo (vale 1 hora; um pedido novo derruba o anterior). O email da conta só muda quando ele é aberto. Se o email já é de outra conta, o dono dela recebe um aviso no lugar do link, e a resposta é a mesma, para não revelar quem tem conta"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_unchanged"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}, email.too_soon {seconds}"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "email.unavailable (teto de envios do dia)"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "email.send_failed (o provedor de email falhou; o link não fica valendo, e pedir de novo não espera os 60 s)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "email.invalid, email.disposable, request.invalid_body"),
    )
)]
pub async fn change_email(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<ChangeEmail>,
) -> Result<(StatusCode, Json<EmailSent>), ApiError> {
    let email = state.mail.normalize(&body.email)?;
    let methods = db::login_methods(&state.db, user.id).await?;
    if methods.email.as_deref() == Some(email.as_str()) {
        return Err(ApiError::conflict("auth.email_unchanged"));
    }
    // Com senha, quem pegou a sessão não troca o email (o login) para depois
    // recuperar a senha por ele.
    if methods.password {
        let current = body
            .current_password
            .ok_or(ApiError::forbidden("auth.wrong_password"))?;
        confirm_password(&state, user.id, current, "auth.no_password").await?;
    }
    // Email de outra conta passa pelos mesmos limites e tem a mesma resposta:
    // só o dono dela fica sabendo, como no cadastro.
    let owner = db::contact_by_email(&state.db, &email).await?;
    let new_link = owner.is_none();
    let ticket = state.mail.reserve(&state.redis, &email).await?;
    let web = &state.auth.web_origin;
    let message = match owner {
        Some(owner) => {
            let locale = Locale::from_db(&owner.locale)?;
            emails::email_in_use(web, locale, &email, owner.has_password)
        }
        None => {
            let token =
                kv::create_email_change(&state.redis, user.id, &email, LINK_TTL_SECS).await?;
            emails::email_change(web, Locale::from_db(&user.locale)?, &email, &token)
        }
    };
    if let Err(err) = ticket.send(message).await {
        // O link não chegou: não fica valendo (o de antes já tinha caído).
        if new_link {
            kv::cancel_email_change(&state.redis, user.id).await?;
        }
        return Err(err);
    }
    Ok((StatusCode::ACCEPTED, Json(EmailSent { email })))
}

#[utoipa::path(
    delete,
    path = "/auth/email",
    tag = "auth",
    responses(
        (status = OK, body = LoginMethods, description = "Email tirado da conta; o link de troca que estava valendo deixa de valer"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.no_email, auth.password_needs_email (a conta tem senha, que só se recupera por ele)"),
    )
)]
pub async fn remove_email(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<LoginMethods>, ApiError> {
    let email = db::remove_email(&state.db, user.id).await?;
    kv::cancel_email_change(&state.redis, user.id).await?;
    notify(&state, user.id, Notice::EmailRemoved { email });
    Ok(Json(db::login_methods(&state.db, user.id).await?))
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
    let (before, has_password) = db::set_email(&state.db, user_id, &email).await?;
    // O link de recuperação que estava valendo foi para o email de antes.
    kv::cancel_password_reset(&state.redis, user_id).await?;
    // Quem põe o primeiro email não tem a quem avisar.
    if let Some(old) = before.filter(|old| *old != email) {
        let notice = Notice::EmailChanged {
            old,
            new: email,
            has_password,
        };
        notify(&state, user_id, notice);
    }
    Ok(StatusCode::NO_CONTENT)
}
