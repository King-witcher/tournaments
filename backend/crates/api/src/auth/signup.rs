//! Cadastro com nome, email e senha. O pedido só guarda o cadastro no Redis e
//! manda o email com o link e o código; a conta nasce com o que vier primeiro
//! (o link aberto ou o código digitado), então cadastro que ninguém confirma
//! não chega ao banco.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;

use super::{
    credentials::start_session,
    db::{self, NewPasswordUser},
    emails::{self, LINK_TTL_SECS},
    handlers::DISPLAY_NAME_MAX,
    kv::{self, CodeCheck, PendingSignup},
    password::{hash_password, validate_password},
    types::{EmailSent, LinkToken, SignedIn, Signup, SignupCode},
};
use crate::{
    AppState,
    shared::{ApiError, ClientIp, ErrorBody, JsonBody, Locale, text::clean_text, token},
};

/// Códigos errados que um cadastro aceita antes de o código morrer (o link
/// continua valendo; pedir de novo manda outro código).
const CODE_ATTEMPTS: u32 = 5;

#[utoipa::path(
    post,
    path = "/auth/signup",
    tag = "auth",
    request_body = Signup,
    responses(
        (status = ACCEPTED, body = EmailSent, description = "Email enviado com o link e um código de 6 dígitos (os dois valem 1 hora); a conta só é criada pelo link aberto ou pelo código digitado. Pedir de novo com o mesmo email troca o link e o código (os anteriores param de valer). Se o email já é de uma conta, o dono recebe um aviso no lugar do link, e a resposta é a mesma"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.display_name_invalid, auth.password_invalid, email.invalid, email.disposable, request.invalid_body"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "email.ip_limited {minutes} (pedidos de email deste IP, somados com a recuperação de senha), email.too_soon {seconds}"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "email.send_failed (o provedor de email falhou; nada fica guardado, e pedir de novo não espera os 60 s)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "email.unavailable (teto de envios do dia)"),
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    ip: ClientIp,
    JsonBody(body): JsonBody<Signup>,
) -> Result<(StatusCode, Json<EmailSent>), ApiError> {
    let display_name = clean_text(
        &body.display_name,
        DISPLAY_NAME_MAX,
        "auth.display_name_invalid",
    )?;
    validate_password(&body.password)?;
    let email = state.mail.normalize(&body.email)?;
    let locale = body.locale.unwrap_or(Locale::Pt);

    // Antes do que gasta (o hash da senha, o envio), e igual para email novo
    // e de uma conta.
    state.mail.limit_ip(&state.redis, &ip).await?;
    // Daqui em diante, email novo e email de uma conta passam pelos mesmos
    // limites e pelo mesmo hash da senha: nem a resposta nem o tempo dizem
    // se o email já tem conta.
    let owner = db::contact_by_email(&state.db, &email).await?;
    let new_signup = owner.is_none();
    let ticket = state.mail.reserve(&state.redis, &email).await?;
    let password_hash = hash_password(body.password).await?;

    let web = &state.auth.web_origin;
    let message = match owner {
        Some(owner) => {
            let locale = Locale::from_db(&owner.locale)?;
            emails::account_exists(web, locale, &email, owner.has_password)
        }
        None => {
            let pending = PendingSignup {
                display_name,
                email: email.clone(),
                password_hash,
                locale,
            };
            let secrets = kv::store_signup(&state.redis, &pending, LINK_TTL_SECS).await?;
            emails::signup_confirmation(web, locale, &email, &secrets.link, &secrets.code)
        }
    };
    if let Err(err) = ticket.send(message).await {
        // O email não chegou: o cadastro sai do Redis, para pedir de novo na
        // hora (a reserva do envio já voltou).
        if new_signup {
            kv::drop_signup(&state.redis, &email).await?;
        }
        return Err(err);
    }
    Ok((StatusCode::ACCEPTED, Json(EmailSent { email })))
}

#[utoipa::path(
    post,
    path = "/auth/signup/confirm",
    tag = "auth",
    request_body = LinkToken,
    responses(
        (status = CREATED, body = SignedIn, description = "Conta criada, já com a sessão no cookie"),
        (status = GONE, body = ErrorBody, description = "auth.link_expired (expirou, já foi usado, o código já criou a conta ou foi trocado por um mais novo)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_taken (outra conta pegou o email desde o pedido)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn confirm_signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<LinkToken>,
) -> Result<(StatusCode, CookieJar, Json<SignedIn>), ApiError> {
    let pending = kv::take_signup(&state.redis, &body.token)
        .await?
        .ok_or(ApiError::gone("auth.link_expired"))?;
    create_account(&state, &headers, jar, pending).await
}

#[utoipa::path(
    post,
    path = "/auth/signup/code",
    tag = "auth",
    request_body = SignupCode,
    responses(
        (status = CREATED, body = SignedIn, description = "Conta criada, já com a sessão no cookie, e o link do mesmo email para de valer"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.code_invalid {attempts_left} (código errado; com 0, ele morreu, e sobra o link ou pedir de novo), email.invalid, request.invalid_body (o código não tem 6 dígitos)"),
        (status = GONE, body = ErrorBody, description = "auth.code_expired (sem cadastro esperando para o email: expirou, o link já criou a conta, foi trocado por um mais novo ou as tentativas acabaram)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.email_taken (outra conta pegou o email desde o pedido)"),
    )
)]
pub async fn confirm_signup_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<SignupCode>,
) -> Result<(StatusCode, CookieJar, Json<SignedIn>), ApiError> {
    if !token::is_code(&body.code) {
        return Err(ApiError::invalid("request.invalid_body"));
    }
    let email = state.mail.normalize(&body.email)?;
    let check = kv::take_signup_by_code(&state.redis, &email, &body.code, CODE_ATTEMPTS).await?;
    let pending = match check {
        CodeCheck::Confirmed(pending) => pending,
        CodeCheck::Wrong { attempts_left } => {
            return Err(ApiError::invalid("auth.code_invalid")
                .with_params([("attempts_left", i64::from(attempts_left))]));
        }
        CodeCheck::Expired => return Err(ApiError::gone("auth.code_expired")),
    };
    create_account(&state, &headers, jar, pending).await
}

/// Cria a conta do cadastro confirmado e entra nela.
async fn create_account(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    pending: PendingSignup,
) -> Result<(StatusCode, CookieJar, Json<SignedIn>), ApiError> {
    let user = NewPasswordUser {
        display_name: &pending.display_name,
        email: &pending.email,
        locale: pending.locale,
        password_hash: &pending.password_hash,
    };
    let user_id = db::create_password_user(&state.db, user).await?;
    let (jar, user) = start_session(state, headers, jar, user_id).await?;
    Ok((StatusCode::CREATED, jar, user))
}
