use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use utoipa::IntoParams;

use super::{
    CurrentUser, db,
    session::{SESSION_COOKIE, STATE_COOKIE, sanitize_return_to},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, token},
};

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

    let oauth_state = token::random_secret();
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
    if !token::constant_time_eq(cookie_state.as_bytes(), query_state.as_bytes()) {
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
        let access_token = state
            .discord
            .exchange_code(code, &auth.redirect_uri)
            .await?;
        let du = state.discord.fetch_user(&access_token).await?;
        let user_id =
            db::upsert_discord_user(&state.db, &du.id, du.display_name(), du.avatar_url()).await?;
        Ok(db::create_session(&state.db, user_id).await?)
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
        db::delete_session(&state.db, &token::hash_token(cookie.value())).await?;
    }
    let jar = jar.remove(state.auth.clear_session_cookie());
    Ok((jar, StatusCode::NO_CONTENT).into_response())
}
