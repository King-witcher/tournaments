//! Cadastro com username, senha e email. O pedido só guarda o cadastro no
//! Redis e manda o link; a conta nasce quando o link é aberto, então cadastro
//! que ninguém confirma não chega ao banco.

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
    kv::{self, PendingSignup},
    password::{hash_password, validate_password, verify_password},
    types::{EmailSent, LinkToken, SignedIn, Signup},
    username::validate_username,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, Locale},
};

#[utoipa::path(
    post,
    path = "/auth/signup",
    tag = "auth",
    request_body = Signup,
    responses(
        (status = ACCEPTED, body = EmailSent, description = "Link de confirmação enviado (vale 1 hora); a conta só é criada quando ele é aberto. Pedir de novo com o mesmo email troca o link; com o mesmo username e a mesma senha e outro email, troca o email (o link anterior para de valer). Se o email já é de uma conta, o dono recebe um aviso no lugar do link, e a resposta é a mesma"),
        (status = CONFLICT, body = ErrorBody, description = "auth.username_taken (por uma conta ou pelo cadastro de outra pessoa esperando confirmação)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "auth.username_invalid, auth.password_invalid, email.invalid, email.disposable, request.invalid_body"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "email.too_soon {seconds}"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "email.send_failed (o provedor de email falhou; nada fica guardado, e pedir de novo não espera os 60 s)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "email.unavailable (teto de envios do dia)"),
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    JsonBody(body): JsonBody<Signup>,
) -> Result<(StatusCode, Json<EmailSent>), ApiError> {
    let username = validate_username(&body.username)?;
    validate_password(&body.password)?;
    let email = state.mail.normalize(&body.email)?;
    let locale = body.locale.unwrap_or(Locale::Pt);

    if db::username_taken(&state.db, &username).await? {
        return Err(ApiError::conflict("auth.username_taken"));
    }
    // Daqui em diante, email novo e email de uma conta passam pelos mesmos
    // limites e pelo mesmo hash da senha: nem a resposta nem o tempo dizem
    // se o email já tem conta.
    let owner = db::contact_by_email(&state.db, &email).await?;
    let new_signup = owner.is_none();
    if new_signup {
        claim_username(&state, &username, &email, &body.password).await?;
    }
    let ticket = state.mail.reserve(&state.redis, &email).await?;
    let password_hash = hash_password(body.password).await?;

    let web = &state.auth.web_origin;
    let message = match owner {
        Some(owner) => {
            let locale = Locale::from_db(&owner.locale)?;
            emails::account_exists(web, locale, &email, owner.username.as_deref())
        }
        None => {
            let pending = PendingSignup {
                username,
                email: email.clone(),
                password_hash,
                locale,
            };
            let token = kv::store_signup(&state.redis, &pending, LINK_TTL_SECS).await?;
            emails::signup_confirmation(web, locale, &email, &pending.username, &token)
        }
    };
    if let Err(err) = ticket.send(message).await {
        // O link não chegou: o cadastro sai do Redis e o username fica livre,
        // para pedir de novo na hora (a reserva do envio já voltou).
        if new_signup {
            kv::drop_signup(&state.redis, &email).await?;
        }
        return Err(err);
    }
    Ok((StatusCode::ACCEPTED, Json(EmailSent { email })))
}

/// Reserva o username para o cadastro de `email`. Se um cadastro de outro
/// email o reservou, quem o fez (a mesma senha) troca de email: um endereço
/// digitado errado não prende o nome por 1 hora, e o cadastro anterior se
/// desfaz. Outra pessoa → 409 `auth.username_taken`.
async fn claim_username(
    state: &AppState,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(), ApiError> {
    let Some(holder) = kv::reserve_username(&state.redis, username, email, LINK_TTL_SECS).await?
    else {
        return Ok(());
    };
    let hash = kv::pending_password_hash(&state.redis, &holder).await?;
    if !verify_password(hash, password.to_owned()).await? {
        return Err(ApiError::conflict("auth.username_taken"));
    }
    kv::drop_signup(&state.redis, &holder).await?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/auth/signup/confirm",
    tag = "auth",
    request_body = LinkToken,
    responses(
        (status = CREATED, body = SignedIn, description = "Conta criada, já com a sessão no cookie. O nome exibido é o username"),
        (status = GONE, body = ErrorBody, description = "auth.link_expired (expirou, já foi usado ou foi trocado por um mais novo)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.username_taken, auth.email_taken (outra conta pegou desde o pedido)"),
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
    let user = NewPasswordUser {
        username: &pending.username,
        email: &pending.email,
        locale: pending.locale,
        password_hash: &pending.password_hash,
    };
    let user_id = db::create_password_user(&state.db, user).await?;
    let (jar, user) = start_session(&state, &headers, jar, user_id).await?;
    Ok((StatusCode::CREATED, jar, user))
}
