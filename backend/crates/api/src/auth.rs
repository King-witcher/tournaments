use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use utoipa::IntoParams;

use crate::{
    AppState,
    current_user::CurrentUser,
    discord::DiscordUser,
    error::{ApiError, ErrorBody},
};

pub const SESSION_COOKIE: &str = "session";
const STATE_COOKIE: &str = "oauth_state";
const SESSION_DAYS: i64 = 30;

/// Configuração de auth que os handlers precisam em tempo de execução.
#[derive(Clone, Debug)]
pub struct AuthSettings {
    /// `None` desliga o login com Discord (503).
    pub discord_client_id: Option<String>,
    pub redirect_uri: String,
    /// Origem do front, sem barra final.
    pub web_origin: String,
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
}

/// SHA-256 do token de sessão, que é o que vai para o banco.
pub fn hash_token(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Só caminhos internos; qualquer outra coisa vira `/` (evita open redirect).
fn sanitize_return_to(raw: Option<&str>) -> String {
    match raw {
        Some(p)
            if p.starts_with('/')
                && !p.starts_with("//")
                && !p.starts_with("/\\")
                && !p.chars().any(char::is_control) =>
        {
            p.to_owned()
        }
        _ => "/".to_owned(),
    }
}

impl AuthSettings {
    fn session_cookie(&self, value: String) -> Cookie<'static> {
        let mut cookie = Cookie::build((SESSION_COOKIE, value))
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .secure(self.cookie_secure)
            .max_age(time::Duration::days(SESSION_DAYS));
        if let Some(domain) = &self.cookie_domain {
            cookie = cookie.domain(domain.clone());
        }
        cookie.build()
    }

    fn clear_session_cookie(&self) -> Cookie<'static> {
        let mut cookie = Cookie::build((SESSION_COOKIE, "")).path("/");
        if let Some(domain) = &self.cookie_domain {
            cookie = cookie.domain(domain.clone());
        }
        cookie.build()
    }

    fn state_cookie(&self, value: String) -> Cookie<'static> {
        Cookie::build((STATE_COOKIE, value))
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/auth")
            .secure(self.cookie_secure)
            .max_age(time::Duration::seconds(600))
            .build()
    }

    fn clear_state_cookie(&self) -> Cookie<'static> {
        Cookie::build((STATE_COOKIE, "")).path("/auth").build()
    }

    fn web_redirect(&self, path: &str) -> Redirect {
        Redirect::to(&format!("{}{}", self.web_origin, path))
    }
}

#[derive(Deserialize, IntoParams)]
pub struct LoginQuery {
    /// Caminho interno para voltar depois do login (ex.: `/pt/torneios`).
    return_to: Option<String>,
}

#[utoipa::path(
    get,
    path = "/auth/discord/login",
    tag = "auth",
    params(LoginQuery),
    responses(
        (status = SEE_OTHER, description = "Redireciona para o Discord e grava o cookie oauth_state"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.discord_not_configured"),
    )
)]
pub async fn discord_login(
    State(state): State<AppState>,
    Query(q): Query<LoginQuery>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let auth = &state.auth;
    let Some(client_id) = &auth.discord_client_id else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "auth.discord_not_configured",
        ));
    };

    let oauth_state = random_token();
    let return_to = sanitize_return_to(q.return_to.as_deref());
    // base64url não tem '.', então ele separa os dois campos.
    let cookie_value = format!("{oauth_state}.{}", URL_SAFE_NO_PAD.encode(&return_to));

    let mut url = reqwest::Url::parse("https://discord.com/oauth2/authorize").expect("url fixa");
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("scope", "identify")
        .append_pair("state", &oauth_state)
        .append_pair("redirect_uri", &auth.redirect_uri);

    Ok((
        jar.add(auth.state_cookie(cookie_value)),
        Redirect::to(url.as_str()),
    ))
}

#[derive(Deserialize, IntoParams)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[utoipa::path(
    get,
    path = "/auth/discord/callback",
    tag = "auth",
    params(CallbackQuery),
    responses(
        (status = SEE_OTHER, description = "Cria a sessão e redireciona para o front; em erro, para /?auth_error=<código>"),
    )
)]
pub async fn discord_callback(
    State(state): State<AppState>,
    Query(q): Query<CallbackQuery>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let auth = &state.auth;
    let stored = jar.get(STATE_COOKIE).map(|c| c.value().to_owned());
    let jar = jar.remove(auth.clear_state_cookie());
    let fail =
        |jar: CookieJar, code: &str| (jar, auth.web_redirect(&format!("/?auth_error={code}")));

    if q.error.is_some() {
        return fail(jar, "auth.discord_denied");
    }

    let parsed = stored.as_deref().and_then(|s| s.split_once('.'));
    let (Some((cookie_state, return_to_b64)), Some(query_state)) = (parsed, q.state.as_deref())
    else {
        return fail(jar, "auth.state_mismatch");
    };
    if !constant_time_eq(cookie_state.as_bytes(), query_state.as_bytes()) {
        return fail(jar, "auth.state_mismatch");
    }
    let return_to = URL_SAFE_NO_PAD
        .decode(return_to_b64)
        .ok()
        .and_then(|b| String::from_utf8(b).ok());
    let return_to = sanitize_return_to(return_to.as_deref());

    let Some(code) = q.code.as_deref() else {
        return fail(jar, "auth.discord_failed");
    };

    let result: anyhow::Result<String> = async {
        let token = state
            .discord
            .exchange_code(code, &auth.redirect_uri)
            .await?;
        let user = state.discord.fetch_user(&token).await?;
        let user_id = upsert_user(&state.db, &user).await?;
        create_session(&state.db, user_id).await
    }
    .await;

    match result {
        Ok(session_token) => (
            jar.add(auth.session_cookie(session_token)),
            auth.web_redirect(&return_to),
        ),
        Err(err) => {
            tracing::error!(error = ?err, "login com Discord falhou");
            fail(jar, "auth.discord_failed")
        }
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn upsert_user(db: &sqlx::PgPool, du: &DiscordUser) -> Result<i64, sqlx::Error> {
    let avatar_url = du
        .avatar
        .as_ref()
        .map(|a| format!("https://cdn.discordapp.com/avatars/{}/{a}.png", du.id));
    let display_name = du.global_name.as_deref().unwrap_or(&du.username);

    let mut tx = db.begin().await?;
    let existing = sqlx::query_scalar!(
        "select user_id from user_identities where provider = 'discord' and provider_user_id = $1",
        du.id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let user_id = match existing {
        Some(id) => {
            sqlx::query!(
                "update users set avatar_url = $1 where id = $2",
                avatar_url,
                id
            )
            .execute(&mut *tx)
            .await?;
            id
        }
        None => {
            let id = sqlx::query_scalar!(
                "insert into users (display_name, avatar_url) values ($1, $2) returning id",
                display_name,
                avatar_url
            )
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query!(
                "insert into user_identities (user_id, provider, provider_user_id) values ($1, 'discord', $2)",
                id,
                du.id
            )
            .execute(&mut *tx)
            .await?;
            id
        }
    };
    tx.commit().await?;
    Ok(user_id)
}

/// Cria a sessão e devolve o token (só o hash vai para o banco).
async fn create_session(db: &sqlx::PgPool, user_id: i64) -> anyhow::Result<String> {
    let token = random_token();
    let hash = hash_token(&token);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(SESSION_DAYS);
    sqlx::query!(
        "insert into sessions (id, user_id, expires_at) values ($1, $2, $3)",
        &hash[..],
        user_id,
        expires_at
    )
    .execute(db)
    .await?;
    Ok(token)
}

#[utoipa::path(
    get,
    path = "/me",
    tag = "auth",
    responses(
        (status = OK, body = CurrentUser, description = "Usuário logado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn me(user: CurrentUser) -> Json<CurrentUser> {
    Json(user)
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses((status = NO_CONTENT, description = "Sessão encerrada (também sem sessão)"))
)]
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Result<Response, ApiError> {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        let hash = hash_token(cookie.value());
        sqlx::query!("delete from sessions where id = $1", &hash[..])
            .execute(&state.db)
            .await?;
    }
    let jar = jar.remove(state.auth.clear_session_cookie());
    Ok((jar, StatusCode::NO_CONTENT).into_response())
}
