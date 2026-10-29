//! Rotas dos provedores externos (Discord e Google): entrar, ligar à conta
//! logada, o callback das duas e desligar. As de cada provedor só passam o
//! dele para o fluxo comum, aqui embaixo; o que muda entre eles (URL de
//! autorização, leitura do usuário) fica em `discord.rs` e `google.rs`.

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser,
    attempts::confirm_password_if_set,
    db::{self, ExternalLogin},
    discord,
    emails::Notice,
    google, kv,
    notify::notify,
    provider::Provider,
    session::{
        AuthSettings, OAuthFlow, OAuthIntent, SESSION_COOKIE, STATE_COOKIE, sanitize_return_to,
        user_agent, with_auth_error,
    },
    types::{CallbackQuery, LoginMethods, LoginQuery, UnlinkLogin},
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
    let return_to = q.return_to.as_deref();
    authorize(
        &state.auth,
        jar,
        Provider::Discord,
        OAuthIntent::Login,
        return_to,
    )
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
    link(&state, &user, Provider::Discord, q, jar).await
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
    callback(&state, Provider::Discord, q, &headers, jar).await
}

#[utoipa::path(
    delete,
    path = "/auth/discord",
    tag = "auth",
    request_body = UnlinkLogin,
    responses(
        (status = OK, body = LoginMethods, description = "Discord desligado; o avatar, que vinha dele, sai junto"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.last_login (a conta não tem senha nem Google), auth.discord_not_linked"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn unlink_discord(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<UnlinkLogin>,
) -> Result<Json<LoginMethods>, ApiError> {
    unlink(&state, &user, Provider::Discord, body).await
}

#[utoipa::path(
    get,
    path = "/auth/google/login",
    tag = "auth",
    params(LoginQuery),
    responses(
        (status = SEE_OTHER, description = "Redireciona para o Google (escopos openid e profile) e grava o cookie oauth_state"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.google_not_configured"),
    )
)]
pub async fn google_login(
    State(state): State<AppState>,
    Query(q): Query<LoginQuery>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let return_to = q.return_to.as_deref();
    authorize(
        &state.auth,
        jar,
        Provider::Google,
        OAuthIntent::Login,
        return_to,
    )
}

#[utoipa::path(
    get,
    path = "/auth/google/link",
    tag = "auth",
    params(LoginQuery),
    responses(
        (status = SEE_OTHER, description = "Redireciona para o Google; o callback liga a conta e volta para return_to (em erro, com ?auth_error=<código>)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "auth.google_already_linked"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.google_not_configured"),
    )
)]
pub async fn google_link(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(q): Query<LoginQuery>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    link(&state, &user, Provider::Google, q, jar).await
}

#[utoipa::path(
    get,
    path = "/auth/google/callback",
    tag = "auth",
    params(CallbackQuery),
    responses(
        (status = SEE_OTHER, description = "Entra (cria a sessão, e a conta com o nome do Google) ou liga o Google e redireciona para o front. Erro ao entrar volta para /?auth_error=<código>; ao ligar, para return_to com ?auth_error=<código>"),
    )
)]
pub async fn google_callback(
    State(state): State<AppState>,
    Query(q): Query<CallbackQuery>,
    headers: HeaderMap,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    callback(&state, Provider::Google, q, &headers, jar).await
}

#[utoipa::path(
    delete,
    path = "/auth/google",
    tag = "auth",
    request_body = UnlinkLogin,
    responses(
        (status = OK, body = LoginMethods, description = "Google desligado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.last_login (a conta não tem senha nem Discord), auth.google_not_linked"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn unlink_google(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<UnlinkLogin>,
) -> Result<Json<LoginMethods>, ApiError> {
    unlink(&state, &user, Provider::Google, body).await
}

/// Redirect para a autorização do provedor, com o state no cookie.
fn authorize(
    auth: &AuthSettings,
    jar: CookieJar,
    provider: Provider,
    intent: OAuthIntent,
    return_to: Option<&str>,
) -> Result<(CookieJar, Redirect), ApiError> {
    let Some(client) = auth.client(provider) else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            provider.codes().not_configured,
        ));
    };

    let flow = OAuthFlow {
        state: token::random_secret(),
        provider,
        intent,
        return_to: sanitize_return_to(return_to),
    };
    let url = match provider {
        Provider::Discord => discord::authorize_url(client, &flow.state),
        Provider::Google => google::authorize_url(client, &flow.state),
    };
    Ok((
        jar.add(auth.state_cookie(flow.encode())),
        Redirect::to(url.as_str()),
    ))
}

/// Ligar o provedor à conta logada: a que já tem um dele nem sai do site.
async fn link(
    state: &AppState,
    user: &CurrentUser,
    provider: Provider,
    q: LoginQuery,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let methods = db::login_methods(&state.db, user.id).await?;
    let linked = match provider {
        Provider::Discord => methods.discord,
        Provider::Google => methods.google,
    };
    if linked {
        return Err(ApiError::conflict(provider.codes().already_linked));
    }
    let return_to = q.return_to.as_deref();
    authorize(&state.auth, jar, provider, OAuthIntent::Link, return_to)
}

async fn callback(
    state: &AppState,
    provider: Provider,
    q: CallbackQuery,
    headers: &HeaderMap,
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

    let agent = user_agent(headers);
    let done = complete_callback(
        state,
        provider,
        &q,
        flow.as_ref(),
        session,
        agent.as_deref(),
    )
    .await;
    match done {
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
    provider: Provider,
    q: &CallbackQuery,
    flow: Option<&OAuthFlow>,
    session: Option<[u8; 32]>,
    user_agent: Option<&str>,
) -> Result<Option<String>, &'static str> {
    let codes = provider.codes();
    if q.error.is_some() {
        return Err(codes.denied);
    }
    let (Some(flow), Some(query_state)) = (flow, q.state.as_deref()) else {
        return Err("auth.state_mismatch");
    };
    if flow.provider != provider
        || !token::constant_time_eq(flow.state.as_bytes(), query_state.as_bytes())
    {
        return Err("auth.state_mismatch");
    }
    let code = q.code.as_deref().ok_or(codes.failed)?;

    match flow.intent {
        OAuthIntent::Login => {
            let login = fetch_login(state, provider, code).await?;
            let result = async {
                let user_id = db::upsert_external_user(&state.db, &login).await?;
                anyhow::Ok(kv::create_session(&state.redis, user_id, user_agent).await?)
            }
            .await;
            result.map(Some).map_err(|err| {
                tracing::error!(error = %err, provider = provider.as_str(), "login pelo provedor falhou");
                codes.failed
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
            let login = fetch_login(state, provider, code).await?;
            let linked = db::link_external(&state.db, user.id, &login)
                .await
                .map_err(|err| err.code)?;
            if linked {
                let notice = match provider {
                    Provider::Discord => Notice::DiscordLinked {
                        discord: login.account,
                    },
                    Provider::Google => Notice::GoogleLinked {
                        google: login.account,
                    },
                };
                notify(state, user.id, notice);
            }
            Ok(None)
        }
    }
}

/// Troca o code no provedor e diz quem entrou.
async fn fetch_login(
    state: &AppState,
    provider: Provider,
    code: &str,
) -> Result<ExternalLogin, &'static str> {
    let failed = provider.codes().failed;
    let redirect_uri = &state.auth.client(provider).ok_or(failed)?.redirect_uri;
    let result = match provider {
        Provider::Discord => {
            async {
                let access_token = state.discord.exchange_code(code, redirect_uri).await?;
                let user = state.discord.fetch_user(&access_token).await?;
                anyhow::Ok(ExternalLogin {
                    provider,
                    display_name: user.display_name().to_owned(),
                    avatar_url: user.avatar_url(),
                    account: user.username,
                    id: user.id,
                })
            }
            .await
        }
        Provider::Google => {
            async {
                let user = state.google.fetch_user(code, redirect_uri).await?;
                let display_name = user.display_name();
                anyhow::Ok(ExternalLogin {
                    provider,
                    account: display_name.clone(),
                    display_name,
                    avatar_url: None,
                    id: user.id,
                })
            }
            .await
        }
    };
    result.map_err(|err| {
        tracing::error!(error = ?err, provider = provider.as_str(), "provedor falhou no callback");
        failed
    })
}

/// Desliga o provedor. Com senha na conta, pede a atual: quem pegou a sessão
/// não tira um login para pôr o seu.
async fn unlink(
    state: &AppState,
    user: &CurrentUser,
    provider: Provider,
    body: UnlinkLogin,
) -> Result<Json<LoginMethods>, ApiError> {
    confirm_password_if_set(state, user.id, body.current_password).await?;
    db::unlink_external(&state.db, user.id, provider).await?;
    let methods = db::login_methods(&state.db, user.id).await?;
    // Sem senha, o aviso diz que a conta segue pelo outro provedor.
    let username = methods.username.clone();
    let notice = match provider {
        Provider::Discord => Notice::DiscordUnlinked { username },
        Provider::Google => Notice::GoogleUnlinked { username },
    };
    notify(state, user.id, notice);
    Ok(Json(methods))
}
