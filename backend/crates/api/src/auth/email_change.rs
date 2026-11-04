//! Email da conta logada: pôr, trocar ou tirar. O email novo só vale depois
//! que o link mandado para ele é aberto ou o código é digitado; até lá, fica o
//! de antes, e o pedido morre com a sessão que o fez: quem a roubou não confirma
//! depois de o dono encerrá-la. O email é um jeito de entrar: tirar o último
//! login é recusado, com senha ele não sai, e na conta que só entra por ele a
//! troca pede antes o código do email de agora (`email_proof.rs`). O pedido não
//! diz se o email já é de outra conta (o dono dela recebe um aviso no lugar do
//! link, e o pedido anterior cai igual), e o código também não.

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser,
    attempts::confirm_password,
    db,
    email_proof::require_email_proof,
    emails::{self, LINK_TTL_SECS, Notice},
    kv::{self, ConfirmedChange},
    notify::notify,
    types::{ChangeEmail, EmailCode, EmailSent, LinkToken, LoginMethods},
};
use crate::{
    AppState,
    shared::{ApiError, ClientIp, ErrorBody, JsonBody, Locale, token},
};

#[utoipa::path(
    put,
    path = "/auth/email",
    tag = "auth",
    request_body = ChangeEmail,
    responses(
        (status = ACCEPTED, body = EmailSent, description = "Link e código de confirmação enviados ao email novo (valem 1 hora, enquanto durar a sessão que pediu; um pedido novo derruba o anterior). O email da conta só muda com um dos dois. Se o email já é de outra conta, o dono dela recebe um aviso no lugar do link, o pedido anterior cai e a resposta é a mesma, para não revelar quem tem conta"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere), auth.email_proof_required (a conta só entra pelo email, e esta sessão não digitou o código mandado a ele)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_unchanged"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}, email.too_soon {seconds}, email.ip_limited {minutes}, email.account_limited {minutes}"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "email.unavailable (teto de envios do dia)"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "email.send_failed (o provedor de email falhou; o link não fica valendo, e pedir de novo não espera os 60 s)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "email.invalid, email.disposable, request.invalid_body"),
    )
)]
pub async fn change_email(
    State(state): State<AppState>,
    user: CurrentUser,
    ip: ClientIp,
    jar: CookieJar,
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
    require_email_proof(&state, &methods, &jar).await?;
    // A espera de 60 s é por endereço: sem isto, uma conta gasta o teto do dia
    // mudando de endereço.
    state.mail.limit_ip(&state.redis, &ip).await?;
    state.mail.limit_account(&state.redis, user.id).await?;
    let session = state
        .auth
        .current_hash(&jar)
        .ok_or(ApiError::unauthenticated())?;
    send_change_email(&state, &user, &email, &session).await?;
    Ok((StatusCode::ACCEPTED, Json(EmailSent { email })))
}

/// Manda o link e o código ao endereço novo ou, se ele é de outra conta, o aviso
/// ao dono. Nos dois casos o pedido anterior cai: se ficasse só aqui, quem tem o
/// código de antes descobriria que o endereço já tem dono. Email de outra conta
/// passa pelos mesmos limites e tem a mesma resposta, como no cadastro.
async fn send_change_email(
    state: &AppState,
    user: &CurrentUser,
    email: &str,
    session: &str,
) -> Result<(), ApiError> {
    let owner = db::contact_by_email(&state.db, email).await?;
    let ticket = state.mail.reserve(&state.redis, email).await?;
    let web = &state.auth.web_origin;
    let message = match owner {
        Some(owner) => {
            kv::cancel_email_change(&state.redis, user.id).await?;
            emails::email_in_use(web, Locale::from_db(&owner.locale)?, email)
        }
        None => {
            let secrets =
                kv::create_email_change(&state.redis, user.id, email, session, LINK_TTL_SECS)
                    .await?;
            let locale = Locale::from_db(&user.locale)?;
            emails::email_change(web, locale, email, &secrets.link, &secrets.code)
        }
    };
    if let Err(err) = ticket.send(message).await {
        // O link não chegou: não fica valendo (o de antes já tinha caído).
        kv::cancel_email_change(&state.redis, user.id).await?;
        return Err(err);
    }
    Ok(())
}

#[utoipa::path(
    delete,
    path = "/auth/email",
    tag = "auth",
    responses(
        (status = OK, body = LoginMethods, description = "Email tirado da conta; o link e o código que estavam valendo deixam de valer"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.no_email, auth.password_needs_email (a conta tem senha, que entra com ele), auth.last_login (é o único jeito de entrar)"),
    )
)]
pub async fn remove_email(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<LoginMethods>, ApiError> {
    let email = db::remove_email(&state.db, user.id).await?;
    kv::cancel_email_change(&state.redis, user.id).await?;
    kv::cancel_email_login(&state.redis, user.id).await?;
    notify(&state, user.id, Notice::EmailRemoved { email });
    Ok(Json(db::login_methods(&state.db, user.id).await?))
}

#[utoipa::path(
    post,
    path = "/auth/email/confirm",
    tag = "auth",
    request_body = LinkToken,
    responses(
        (status = NO_CONTENT, description = "Email da conta trocado, e o código do mesmo email para de valer. Não pede login: o link prova que quem abriu recebe no email novo, e abre em outro navegador. A sessão que pediu a troca precisa continuar aberta"),
        (status = GONE, body = ErrorBody, description = "auth.link_expired (expirou, já foi usado, o código já confirmou, foi trocado por um mais novo ou a sessão que pediu foi encerrada)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_taken (outra conta confirmou este email antes)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn confirm_email(
    State(state): State<AppState>,
    JsonBody(body): JsonBody<LinkToken>,
) -> Result<StatusCode, ApiError> {
    let expired = ApiError::gone("auth.link_expired");
    let Some((user_id, change)) = kv::take_email_change(&state.redis, &body.token).await? else {
        return Err(expired);
    };
    apply_change(&state, user_id, change, expired).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/auth/email/code",
    tag = "auth",
    request_body = EmailCode,
    responses(
        (status = NO_CONTENT, description = "Email da conta trocado, e o link do mesmo email para de valer. A sessão que pediu a troca precisa continuar aberta"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_taken (outra conta confirmou este email antes)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.code_rejected (código errado, vencido, sem tentativas, sem troca pedida ou com a sessão que pediu encerrada: a resposta é uma só, porque pôr o email de outra conta não deixa pedido, e isso não pode aparecer; a 5ª errada mata o código, e o link continua valendo), request.invalid_body (o código não tem 6 dígitos)"),
    )
)]
pub async fn confirm_email_by_code(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<EmailCode>,
) -> Result<StatusCode, ApiError> {
    if !token::is_code(&body.code) {
        return Err(ApiError::invalid("request.invalid_body"));
    }
    let rejected = ApiError::invalid("auth.code_rejected");
    let Some(change) = kv::take_email_change_by_code(&state.redis, user.id, &body.code).await?
    else {
        return Err(rejected);
    };
    apply_change(&state, user.id, change, rejected).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Grava o email confirmado e avisa o endereço de antes, se a sessão que pediu a
/// troca ainda existe; senão, `refused`. Só a existência dela é conferida: o link
/// abre em outro navegador. Cobre logout, encerrar sessões e senha nova, que
/// encerram quem pegou a sessão.
async fn apply_change(
    state: &AppState,
    user_id: i64,
    change: ConfirmedChange,
    refused: ApiError,
) -> Result<(), ApiError> {
    if !kv::session_exists(&state.redis, &change.session).await? {
        return Err(refused);
    }
    let email = change.email;
    let saved = db::set_email(&state.db, user_id, &email).await?;
    // O link e o código de entrar que estavam valendo foram para o email de antes.
    kv::cancel_email_login(&state.redis, user_id).await?;
    // Quem põe o primeiro email não tem a quem avisar.
    if let Some(old) = saved.before.filter(|old| *old != email) {
        let notice = Notice::EmailChanged {
            old,
            new: email,
            has_password: saved.has_password,
            has_provider: saved.has_provider,
        };
        notify(state, user_id, notice);
    }
    Ok(())
}
