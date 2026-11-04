//! Entrar por email, com ou sem senha na conta: o pedido manda um email com um
//! link e um código, e vale o que vier primeiro. É também como quem esqueceu a
//! senha volta: ao entrar, pode definir uma nova, sem precisar da atual. Nada
//! aqui diz se o email tem conta: o pedido responde igual, e o código errado
//! responde igual a não haver pedido nem conta.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;

use super::{
    credentials::start_session,
    db::{self, AccountContact},
    emails::{self, EMAIL_LOGIN_TTL_SECS, Notice},
    handlers::sessions_ended,
    kv,
    notify::notify,
    password::{hash_password, validate_password},
    types::{EmailLoginCode, EmailLoginLink, EmailLoginRequest, SignedIn},
};
use crate::{
    AppState,
    shared::{ApiError, ClientIp, ErrorBody, JsonBody, Locale, token},
};

/// Nenhuma conta tem este id (a identidade começa em 1): é o dono do pedido do
/// email sem conta, que nunca existe, e o caminho no Redis é o mesmo.
const NO_ACCOUNT: i64 = 0;

#[utoipa::path(
    post,
    path = "/auth/email/login",
    tag = "auth",
    request_body = EmailLoginRequest,
    responses(
        (status = ACCEPTED, description = "Se a conta existe, o email dela recebe um link e um código de 6 dígitos (valem 15 minutos, e o que vier primeiro gasta os dois; um pedido novo derruba o anterior). A resposta é a mesma em todos os casos, até fora dos limites de envio do endereço, para não revelar quem tem conta"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "email.ip_limited {minutes} (pedidos de email deste IP, somados com o cadastro; conta todo pedido, exista a conta ou não)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "email.invalid (fora do formato de email; não conta no limite), request.invalid_body"),
    )
)]
pub async fn request_email_login(
    State(state): State<AppState>,
    ip: ClientIp,
    JsonBody(body): JsonBody<EmailLoginRequest>,
) -> Result<StatusCode, ApiError> {
    // O formato não diz nada sobre quem tem conta.
    let email = state.mail.canonical(&body.email)?;
    // Todo pedido conta, antes de saber se a conta existe: o 429 não diz nada.
    state.mail.limit_ip(&state.redis, &ip).await?;
    if let Some(contact) = db::contact_by_email(&state.db, &email).await? {
        // Em segundo plano: a resposta sai no mesmo tempo, exista a conta ou não.
        tokio::spawn(async move {
            if let Err(err) = send_login_email(&state, contact).await {
                tracing::error!(error = ?err, "email de entrar por email falhou");
            }
        });
    }
    Ok(StatusCode::ACCEPTED)
}

/// Manda o link e o código, se o endereço está dentro dos limites de envio.
async fn send_login_email(state: &AppState, contact: AccountContact) -> Result<(), ApiError> {
    let AccountContact {
        user_id,
        email: Some(email),
        locale,
    } = contact
    else {
        return Ok(());
    };
    let Some(ticket) = state.mail.reserve_quietly(&state.redis, &email).await? else {
        return Ok(());
    };
    let secrets =
        kv::create_email_login(&state.redis, user_id, &email, EMAIL_LOGIN_TTL_SECS).await?;
    let locale = Locale::from_db(&locale)?;
    let web = &state.auth.web_origin;
    let sent = ticket
        .send(emails::email_login(
            web,
            locale,
            &email,
            &secrets.link,
            &secrets.code,
        ))
        .await;
    if sent.is_err() {
        // O email não chegou: nem o link nem o código ficam valendo.
        kv::cancel_email_login(&state.redis, user_id).await?;
    }
    sent
}

#[utoipa::path(
    post,
    path = "/auth/email/login/confirm",
    tag = "auth",
    request_body = EmailLoginLink,
    responses(
        (status = OK, body = SignedIn, description = "Sessão criada no cookie, e o código do mesmo email para de valer. Com `password`, a senha da conta passa a ser essa, todas as outras sessões são encerradas e o email recebe um aviso. Se a exclusão da conta estava agendada, entrar a cancelou"),
        (status = GONE, body = ErrorBody, description = "auth.link_expired (expirou, já foi usado, o código já entrou ou foi trocado por um mais novo)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.password_invalid (não gasta o link), request.invalid_body"),
    )
)]
pub async fn email_login_by_link(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<EmailLoginLink>,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    // Antes de gastar o link: senha fora do formato não o consome.
    validate_new_password(body.password.as_deref())?;
    let expired = ApiError::gone("auth.link_expired");
    let Some((user_id, sent_to)) = kv::take_email_login(&state.redis, &body.token).await? else {
        return Err(expired);
    };
    let attempt = Entering {
        user_id,
        sent_to,
        password: body.password,
        refused: expired,
    };
    enter(&state, &headers, jar, attempt).await
}

#[utoipa::path(
    post,
    path = "/auth/email/login/code",
    tag = "auth",
    request_body = EmailLoginCode,
    responses(
        (status = OK, body = SignedIn, description = "Como entrar pelo link: sessão no cookie, o link do mesmo email para de valer e, com `password`, a senha nova"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.code_rejected (código errado, vencido, sem tentativas ou de um email sem conta: a resposta é uma só, para não revelar quem tem conta; a 5ª errada mata o código, e o link continua valendo), auth.password_invalid (não gasta tentativa), email.invalid, request.invalid_body (o código não tem 6 dígitos)"),
    )
)]
pub async fn email_login_by_code(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<EmailLoginCode>,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    if !token::is_code(&body.code) {
        return Err(ApiError::invalid("request.invalid_body"));
    }
    validate_new_password(body.password.as_deref())?;
    let email = state.mail.canonical(&body.email)?;
    let owner = db::contact_by_email(&state.db, &email)
        .await?
        .map_or(NO_ACCOUNT, |contact| contact.user_id);
    let rejected = ApiError::invalid("auth.code_rejected");
    let Some(sent_to) = kv::take_email_login_by_code(&state.redis, owner, &body.code).await? else {
        return Err(rejected);
    };
    let attempt = Entering {
        user_id: owner,
        sent_to,
        password: body.password,
        refused: rejected,
    };
    enter(&state, &headers, jar, attempt).await
}

fn validate_new_password(password: Option<&str>) -> Result<(), ApiError> {
    password.map_or(Ok(()), validate_password)
}

/// Quem acabou de provar que recebeu o link ou o código.
struct Entering {
    user_id: i64,
    /// O endereço para onde o link e o código foram.
    sent_to: String,
    password: Option<String>,
    /// A resposta se a conta não tem mais esse endereço.
    refused: ApiError,
}

/// Abre a sessão de quem provou receber no email da conta, com a senha nova, se
/// veio. Só vale o endereço de agora: o pedido pode ter ido ao de antes, se a
/// troca aconteceu entre ler a conta e guardar o pedido, e a conta pode ter
/// perdido o email (ou sido anonimizada) desde então.
async fn enter(
    state: &AppState,
    headers: &HeaderMap,
    jar: CookieJar,
    attempt: Entering,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    let user_id = attempt.user_id;
    let contact = db::contact_by_id(&state.db, user_id).await?;
    let current = contact.and_then(|contact| contact.email);
    if current.as_deref() != Some(attempt.sent_to.as_str()) {
        return Err(attempt.refused);
    }
    if let Some(password) = attempt.password {
        set_new_password(state, user_id, &attempt.sent_to, password).await?;
    }
    start_session(state, headers, jar, user_id).await
}

/// A senha nova, criada ou no lugar da de antes. Quem a sabia pode estar logado:
/// todas as sessões saem (a de quem entra abre depois), e os erros de senha do
/// email, que eram contra a antiga, também. E a troca de email que ele tenha
/// pedido: confirmada, poria o email dele na conta, e o próximo email de entrar
/// iria para lá. Nessa ordem: se o Redis falhar antes do fim, a senha ainda é a
/// de antes, e não uma nova com as sessões de quem sabia a antiga abertas.
async fn set_new_password(
    state: &AppState,
    user_id: i64,
    email: &str,
    password: String,
) -> Result<(), ApiError> {
    let hash = hash_password(password).await?;
    kv::cancel_email_change(&state.redis, user_id).await?;
    let ended = kv::delete_all_sessions(&state.redis, user_id).await?;
    sessions_ended(state, user_id, &ended).await?;
    db::set_password(&state.db, user_id, &hash).await?;
    kv::clear_password_failures(&state.redis, email).await?;
    notify(state, user_id, Notice::PasswordSetByEmail);
    Ok(())
}
