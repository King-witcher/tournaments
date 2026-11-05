//! Conta em que o email é o único jeito de entrar: sem senha nem provedor, a
//! sessão sozinha não muda os logins. Quem a roubasse trocaria o único login, e
//! o dono ficaria sem saída; ou criaria uma senha ou vincularia o próprio
//! provedor, que também tirariam a conta do "só email". Por isso as três coisas
//! pedem antes um código mandado ao email de agora. O código vale para a sessão
//! que o digitou: a roubada não aproveita o dono ter provado na dele.

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser, db,
    emails::{self, EMAIL_LOGIN_TTL_SECS},
    kv,
    types::{EmailCode, EmailProofStatus, LoginMethods},
};
use crate::{
    AppState,
    shared::{ApiError, ClientIp, ErrorBody, JsonBody, Locale, token},
};

/// Quanto a prova vale para a sessão: o tempo de fazer a mudança com calma.
const PROVEN_SECS: u64 = 10 * 60;

/// O email é o único jeito de entrar na conta.
fn only_email(methods: &LoginMethods) -> bool {
    methods.email.is_some() && !methods.password && !methods.discord && !methods.google
}

/// Exige a prova, se a conta só entra pelo email. Recusa com 403
/// `auth.email_proof_required`.
pub(super) async fn require_email_proof(
    state: &AppState,
    methods: &LoginMethods,
    jar: &CookieJar,
) -> Result<(), ApiError> {
    if !only_email(methods) {
        return Ok(());
    }
    let session = state
        .auth
        .current_hash(jar)
        .ok_or(ApiError::unauthenticated())?;
    if kv::email_is_proven(&state.redis, &session).await? {
        Ok(())
    } else {
        Err(ApiError::forbidden("auth.email_proof_required"))
    }
}

#[utoipa::path(
    get,
    path = "/auth/email/proof",
    tag = "auth",
    responses(
        (status = OK, body = EmailProofStatus, description = "Se a conta só entra pelo email e se esta sessão já digitou o código (vale 10 minutos)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn email_proof_status(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
) -> Result<Json<EmailProofStatus>, ApiError> {
    let methods = db::login_methods(&state.db, user.id).await?;
    let needed = only_email(&methods);
    let proven = needed && require_email_proof(&state, &methods, &jar).await.is_ok();
    Ok(Json(EmailProofStatus { needed, proven }))
}

#[utoipa::path(
    post,
    path = "/auth/email/proof",
    tag = "auth",
    responses(
        (status = ACCEPTED, description = "Código de 6 dígitos mandado ao email da conta (vale 15 minutos; um pedido novo derruba o anterior). Dentro da espera de 60 s do endereço, o email só não sai, e o código anterior continua"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_proof_not_needed (a conta entra de outro jeito além do email)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "email.ip_limited {minutes}, email.account_limited {minutes}"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "email.send_failed (o provedor de email falhou; o código não fica valendo, e pedir de novo não espera os 60 s)"),
    )
)]
pub async fn request_email_proof(
    State(state): State<AppState>,
    user: CurrentUser,
    ip: ClientIp,
) -> Result<StatusCode, ApiError> {
    let methods = db::login_methods(&state.db, user.id).await?;
    let Some(email) = methods.email.as_deref().filter(|_| only_email(&methods)) else {
        return Err(ApiError::conflict("auth.email_proof_not_needed"));
    };
    state.mail.limit_ip(&ip)?;
    state.mail.limit_account(user.id)?;
    let Some(ticket) = state.mail.reserve_quietly(&state.redis, email).await? else {
        return Ok(StatusCode::ACCEPTED);
    };
    let secrets = kv::create_email_proof(&state.redis, user.id, EMAIL_LOGIN_TTL_SECS).await?;
    let locale = Locale::from_db(&user.locale)?;
    let web = &state.auth.web_origin;
    let sent = ticket
        .send(emails::email_proof(web, locale, email, &secrets.code))
        .await;
    if let Err(err) = sent {
        kv::cancel_email_proof(&state.redis, user.id).await?;
        return Err(err);
    }
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    post,
    path = "/auth/email/proof/code",
    tag = "auth",
    request_body = EmailCode,
    responses(
        (status = NO_CONTENT, description = "Esta sessão pode mudar os logins da conta pelos próximos 10 minutos"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.code_rejected (código errado, vencido ou sem tentativas), request.invalid_body (o código não tem 6 dígitos)"),
    )
)]
pub async fn confirm_email_proof(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    JsonBody(body): JsonBody<EmailCode>,
) -> Result<StatusCode, ApiError> {
    if !token::is_code(&body.code) {
        return Err(ApiError::invalid("request.invalid_body"));
    }
    let session = state
        .auth
        .current_hash(&jar)
        .ok_or(ApiError::unauthenticated())?;
    if !kv::take_email_proof_by_code(&state.redis, user.id, &body.code).await? {
        return Err(ApiError::invalid("auth.code_rejected"));
    }
    kv::mark_email_proven(&state.redis, &session, PROVEN_SECS).await?;
    Ok(StatusCode::NO_CONTENT)
}
