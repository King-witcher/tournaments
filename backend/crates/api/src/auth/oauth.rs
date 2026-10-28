//! Rotas do Discord: entrar, ligar à conta logada, o callback das duas e
//! desligar.

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser, DiscordUser,
    attempts::confirm_password,
    db, kv,
    session::{
        AuthSettings, OAuthFlow, OAuthIntent, SESSION_COOKIE, STATE_COOKIE, sanitize_return_to,
        user_agent, with_auth_error,
    },
    types::{CallbackQuery, CurrentPassword, LoginMethods, LoginQuery},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, token},
};

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
    authorize(&state.auth, jar, OAuthIntent::Login, q.return_to.as_deref())
}

#[utoipa::path(
    get,
    path = "/auth/discord/link",
    tag = "auth",
    params(LoginQuery),
    responses(
        (status = SEE_OTHER, description = "Redireciona para o Discord; o callback liga a conta e volta para return_to (em erro, com ?auth_error=<código>)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.discord_already_linked"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.discord_not_configured"),
    )
)]
pub async fn discord_link(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(q): Query<LoginQuery>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    if db::login_methods(&state.db, user.id).await?.discord {
        return Err(ApiError::conflict("auth.discord_already_linked"));
    }
    authorize(&state.auth, jar, OAuthIntent::Link, q.return_to.as_deref())
}

/// Redirect para a autorização do Discord, com o state no cookie.
fn authorize(
    auth: &AuthSettings,
    jar: CookieJar,
    intent: OAuthIntent,
    return_to: Option<&str>,
) -> Result<(CookieJar, Redirect), ApiError> {
    let Some(client_id) = &auth.discord_client_id else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "auth.discord_not_configured",
        ));
    };

    let flow = OAuthFlow {
        state: token::random_secret(),
        intent,
        return_to: sanitize_return_to(return_to),
    };

    let mut url = reqwest::Url::parse("https://discord.com/oauth2/authorize").expect("url fixa");
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("scope", "identify")
        .append_pair("state", &flow.state)
        .append_pair("redirect_uri", &auth.redirect_uri);

    Ok((
        jar.add(auth.state_cookie(flow.encode())),
        Redirect::to(url.as_str()),
    ))
}

#[utoipa::path(
    get,
    path = "/auth/discord/callback",
    tag = "auth",
    params(CallbackQuery),
    responses(
        (status = SEE_OTHER, description = "Entra (cria a sessão) ou liga o Discord e redireciona para o front. Erro ao entrar volta para /?auth_error=<código>; ao ligar, para return_to com ?auth_error=<código>"),
    )
)]
pub async fn discord_callback(
    State(state): State<AppState>,
    Query(q): Query<CallbackQuery>,
    headers: HeaderMap,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let auth = &state.auth;
    let flow = jar
        .get(STATE_COOKIE)
        .and_then(|c| OAuthFlow::decode(c.value()));
    let session = jar
        .get(SESSION_COOKIE)
        .map(|c| token::hash_token(c.value()));
    let jar = jar.remove(auth.clear_state_cookie());
    let return_to = flow.as_ref().map_or("/", |f| f.return_to.as_str());

    let agent = user_agent(&headers);
    match complete_callback(&state, &q, flow.as_ref(), session, agent.as_deref()).await {
        Ok(Some(session_token)) => (
            jar.add(auth.session_cookie(session_token)),
            auth.web_redirect(return_to),
        ),
        Ok(None) => (jar, auth.web_redirect(return_to)),
        Err(code) => {
            let back = match &flow {
                Some(f) if f.intent == OAuthIntent::Link => return_to,
                _ => "/",
            };
            (jar, auth.web_redirect(&with_auth_error(back, code)))
        }
    }
}

/// Confere o state e conclui: entrar devolve o token da sessão nova; ligar,
/// `None`. Erro sai como código.
async fn complete_callback(
    state: &AppState,
    q: &CallbackQuery,
    flow: Option<&OAuthFlow>,
    session: Option<[u8; 32]>,
    user_agent: Option<&str>,
) -> Result<Option<String>, &'static str> {
    if q.error.is_some() {
        return Err("auth.discord_denied");
    }
    let (Some(flow), Some(query_state)) = (flow, q.state.as_deref()) else {
        return Err("auth.state_mismatch");
    };
    if !token::constant_time_eq(flow.state.as_bytes(), query_state.as_bytes()) {
        return Err("auth.state_mismatch");
    }
    let code = q.code.as_deref().ok_or("auth.discord_failed")?;

    match flow.intent {
        OAuthIntent::Login => {
            let du = fetch_discord_user(state, code).await?;
            let result = async {
                let user_id =
                    db::upsert_discord_user(&state.db, &du.id, du.display_name(), du.avatar_url())
                        .await?;
                anyhow::Ok(kv::create_session(&state.redis, user_id, user_agent).await?)
            }
            .await;
            result.map(Some).map_err(|err| {
                tracing::error!(error = %err, "login com Discord falhou");
                "auth.discord_failed"
            })
        }
        OAuthIntent::Link => {
            // O state prova que o pedido saiu deste navegador; a sessão diz
            // de quem é a conta.
            let user = match session {
                Some(hash) => CurrentUser::of_session(state, &hash)
                    .await
                    .map_err(|err| err.code)?,
                None => None,
            }
            .ok_or("auth.unauthenticated")?;
            let du = fetch_discord_user(state, code).await?;
            db::link_discord(&state.db, user.id, &du.id, du.avatar_url())
                .await
                .map_err(|err| err.code)?;
            Ok(None)
        }
    }
}

async fn fetch_discord_user(state: &AppState, code: &str) -> Result<DiscordUser, &'static str> {
    let result = async {
        let access_token = state
            .discord
            .exchange_code(code, &state.auth.redirect_uri)
            .await?;
        state.discord.fetch_user(&access_token).await
    }
    .await;
    result.map_err(|err| {
        tracing::error!(error = ?err, "Discord falhou no callback");
        "auth.discord_failed"
    })
}

#[utoipa::path(
    delete,
    path = "/auth/discord",
    tag = "auth",
    request_body = CurrentPassword,
    responses(
        (status = OK, body = LoginMethods, description = "Discord desligado; o avatar, que vinha dele, sai junto"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password"),
        (status = CONFLICT, body = ErrorBody, description = "auth.last_login (a conta não tem senha), auth.email_required (a conta não tem email para recuperar a senha), auth.discord_not_linked"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn unlink_discord(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<CurrentPassword>,
) -> Result<Json<LoginMethods>, ApiError> {
    confirm_password(&state, user.id, body.current_password, "auth.last_login").await?;
    db::unlink_discord(&state.db, user.id).await?;
    Ok(Json(db::login_methods(&state.db, user.id).await?))
}
