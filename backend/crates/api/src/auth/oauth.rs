//! Rotas dos provedores externos (Discord e Google), com PKCE gerado no
//! navegador: o front gera o state e o code verifier e guarda os dois na aba;
//! o provedor volta para a página de callback do front, que confere o state e
//! manda aqui o code com o verifier, para entrar (`login`) ou ligar à conta
//! logada (`link`). O provedor confere o verifier contra o challenge na troca,
//! então o code só serve a quem abriu o fluxo. Para entrar, a API não guarda
//! nada entre as duas pontas, e o front monta a URL de autorização sozinho
//! (com o client id público, o mesmo escopo e o mesmo redirect de
//! `discord::authorize_url` e `google::authorize_url`), indo direto ao
//! provedor. Para ligar, passa antes por `authorize`, que pede a senha atual
//! (se a conta tem: quem pegou a sessão não põe o próprio login nela), guarda
//! o challenge com a sessão (`kv::authorize_link`) e devolve a URL; `link` só
//! aceita o code com o verifier dele, nessa sessão. As rotas de cada provedor só passam o
//! dele para o fluxo comum, aqui embaixo; o que muda entre eles (URL de
//! autorização, leitura do usuário) fica em `discord.rs` e `google.rs`.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser,
    attempts::confirm_password_if_set,
    credentials::start_session,
    db::{self, ExternalLogin},
    discord,
    emails::Notice,
    google, kv,
    notify::notify,
    pkce,
    provider::Provider,
    session::current_hash,
    types::{LoginMethods, OAuthCode, OAuthRedirect, OAuthStart, SignedIn, UnlinkLogin},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody},
};

/// Tamanho máximo do code que o front repassa (os dos provedores têm menos
/// de 100 caracteres).
const CODE_MAX: usize = 512;

/// Prazo para voltar do provedor e ligar: o mesmo do fluxo guardado na aba
/// (`features/auth/lib/oauth-flow.ts`, no front).
const LINK_FLOW_SECS: u64 = 10 * 60;

#[utoipa::path(
    post,
    path = "/auth/discord/authorize",
    tag = "auth",
    request_body = OAuthStart,
    responses(
        (status = OK, body = OAuthRedirect, description = "Só para ligar o Discord à conta logada (para entrar, o front vai direto ao Discord). A autorização no Discord (escopo identify), com o state e o challenge; o Discord volta para /auth/discord/callback no front, com code e state (ou error). Esta sessão fica autorizada por 10 minutos a ligar o code deste challenge"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.discord_already_linked (a conta já tem um)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body (também state ou challenge fora do formato)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.discord_not_configured"),
    )
)]
pub async fn discord_authorize(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    JsonBody(body): JsonBody<OAuthStart>,
) -> Result<Json<OAuthRedirect>, ApiError> {
    authorize(&state, user, &jar, Provider::Discord, body).await
}

#[utoipa::path(
    post,
    path = "/auth/discord/login",
    tag = "auth",
    request_body = OAuthCode,
    responses(
        (status = OK, body = SignedIn, description = "Sessão criada no cookie (e a conta, se o Discord ainda não entrou aqui). Se a exclusão da conta estava agendada, entrar a cancelou (`deletion_cancelled`)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body (também code vazio ou verifier fora do formato)"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "auth.discord_failed (o Discord não respondeu ou recusou o code, por exemplo com outro verifier)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.discord_not_configured"),
    )
)]
pub async fn discord_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<OAuthCode>,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    login(&state, Provider::Discord, &headers, jar, body).await
}

#[utoipa::path(
    post,
    path = "/auth/discord/link",
    tag = "auth",
    request_body = OAuthCode,
    responses(
        (status = OK, body = LoginMethods, description = "Discord ligado à conta logada; o avatar passa a ser o dele"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.state_mismatch (o verifier não é do fluxo que /auth/discord/authorize abriu para ligar nesta sessão nos últimos 10 minutos; a autorização vale uma vez)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.discord_taken (é login de outra conta), auth.discord_already_linked"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body (também code vazio ou verifier fora do formato)"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "auth.discord_failed"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.discord_not_configured"),
    )
)]
pub async fn discord_link(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    JsonBody(body): JsonBody<OAuthCode>,
) -> Result<Json<LoginMethods>, ApiError> {
    link(&state, &user, &jar, Provider::Discord, body).await
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
    post,
    path = "/auth/google/authorize",
    tag = "auth",
    request_body = OAuthStart,
    responses(
        (status = OK, body = OAuthRedirect, description = "Só para ligar o Google à conta logada (para entrar, o front vai direto ao Google). A autorização no Google (escopos openid e profile), com o state e o challenge; o Google volta para /auth/google/callback no front, com code e state (ou error). Esta sessão fica autorizada por 10 minutos a ligar o code deste challenge"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.google_already_linked (a conta já tem uma)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body (também state ou challenge fora do formato)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.google_not_configured"),
    )
)]
pub async fn google_authorize(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    JsonBody(body): JsonBody<OAuthStart>,
) -> Result<Json<OAuthRedirect>, ApiError> {
    authorize(&state, user, &jar, Provider::Google, body).await
}

#[utoipa::path(
    post,
    path = "/auth/google/login",
    tag = "auth",
    request_body = OAuthCode,
    responses(
        (status = OK, body = SignedIn, description = "Sessão criada no cookie (e a conta, com o nome do Google, se ele ainda não entrou aqui). Se a exclusão da conta estava agendada, entrar a cancelou (`deletion_cancelled`)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body (também code vazio ou verifier fora do formato)"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "auth.google_failed (o Google não respondeu ou recusou o code, por exemplo com outro verifier)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.google_not_configured"),
    )
)]
pub async fn google_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    JsonBody(body): JsonBody<OAuthCode>,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    login(&state, Provider::Google, &headers, jar, body).await
}

#[utoipa::path(
    post,
    path = "/auth/google/link",
    tag = "auth",
    request_body = OAuthCode,
    responses(
        (status = OK, body = LoginMethods, description = "Conta Google ligada à conta logada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.state_mismatch (o verifier não é do fluxo que /auth/google/authorize abriu para ligar nesta sessão nos últimos 10 minutos; a autorização vale uma vez)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.google_taken (é login de outra conta), auth.google_already_linked"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body (também code vazio ou verifier fora do formato)"),
        (status = BAD_GATEWAY, body = ErrorBody, description = "auth.google_failed"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "auth.google_not_configured"),
    )
)]
pub async fn google_link(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    JsonBody(body): JsonBody<OAuthCode>,
) -> Result<Json<LoginMethods>, ApiError> {
    link(&state, &user, &jar, Provider::Google, body).await
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

/// A URL de autorização do provedor para ligá-lo à conta logada, com o state
/// e o challenge do navegador. Confere antes se a conta já tem um dele (aí a
/// pessoa nem sai do site) e a senha atual, se a conta tem, e autoriza esta
/// sessão a ligar o code deste challenge.
async fn authorize(
    state: &AppState,
    user: CurrentUser,
    jar: &CookieJar,
    provider: Provider,
    body: OAuthStart,
) -> Result<Json<OAuthRedirect>, ApiError> {
    let codes = provider.codes();
    let client = state.auth.client(provider).ok_or(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        codes.not_configured,
    ))?;
    if !pkce::is_state(&body.state) || !pkce::is_challenge(&body.code_challenge) {
        return Err(ApiError::invalid("request.invalid_body"));
    }
    let session = current_hash(jar).ok_or(ApiError::unauthenticated())?;
    let methods = db::login_methods(&state.db, user.id).await?;
    let linked = match provider {
        Provider::Discord => methods.discord,
        Provider::Google => methods.google,
    };
    if linked {
        return Err(ApiError::conflict(codes.already_linked));
    }
    // Com senha, quem pegou a sessão não põe o próprio login na conta (para
    // entrar depois por ele, mesmo com a senha trocada).
    confirm_password_if_set(state, user.id, body.current_password).await?;
    kv::authorize_link(
        &state.redis,
        &session,
        provider.as_str(),
        &body.code_challenge,
        LINK_FLOW_SECS,
    )
    .await?;

    let url = match provider {
        Provider::Discord => discord::authorize_url(client, &body.state, &body.code_challenge),
        Provider::Google => google::authorize_url(client, &body.state, &body.code_challenge),
    };
    Ok(Json(OAuthRedirect { url: url.into() }))
}

/// Entra pela conta do provedor, criando a conta daqui se é a primeira vez,
/// com a sessão no cookie, como no login com senha (que também cancela a
/// exclusão agendada).
async fn login(
    state: &AppState,
    provider: Provider,
    headers: &HeaderMap,
    jar: CookieJar,
    body: OAuthCode,
) -> Result<(CookieJar, Json<SignedIn>), ApiError> {
    let login = fetch_login(state, provider, &body).await?;
    let user_id = db::upsert_external_user(&state.db, &login).await?;
    start_session(state, headers, jar, user_id).await
}

/// Liga a conta do provedor à conta logada e avisa por email. Só o code do
/// fluxo que `authorize` abriu para ligar nesta sessão (o verifier é o do
/// challenge guardado), uma vez.
async fn link(
    state: &AppState,
    user: &CurrentUser,
    jar: &CookieJar,
    provider: Provider,
    body: OAuthCode,
) -> Result<Json<LoginMethods>, ApiError> {
    let session = current_hash(jar).ok_or(ApiError::unauthenticated())?;
    let challenge = pkce::challenge_of(&body.code_verifier);
    if !kv::take_link_authorization(&state.redis, &session, provider.as_str(), &challenge).await? {
        return Err(ApiError::forbidden("auth.state_mismatch"));
    }
    let login = fetch_login(state, provider, &body).await?;
    if db::link_external(&state.db, user.id, &login).await? {
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
    Ok(Json(db::login_methods(&state.db, user.id).await?))
}

/// Troca o code, com o verifier, no provedor e diz quem entrou.
async fn fetch_login(
    state: &AppState,
    provider: Provider,
    body: &OAuthCode,
) -> Result<ExternalLogin, ApiError> {
    let codes = provider.codes();
    let client = state.auth.client(provider).ok_or(ApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        codes.not_configured,
    ))?;
    let OAuthCode {
        code,
        code_verifier: verifier,
    } = body;
    if code.is_empty() || code.len() > CODE_MAX || !pkce::is_verifier(verifier) {
        return Err(ApiError::invalid("request.invalid_body"));
    }

    let redirect_uri = &client.redirect_uri;
    let result = match provider {
        Provider::Discord => {
            async {
                let access_token = state
                    .discord
                    .exchange_code(code, verifier, redirect_uri)
                    .await?;
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
                let user = state
                    .google
                    .fetch_user(code, verifier, redirect_uri)
                    .await?;
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
        tracing::error!(
            error = format_args!("{err:#}"),
            provider = provider.as_str(),
            "provedor falhou na troca do code"
        );
        ApiError::new(StatusCode::BAD_GATEWAY, codes.failed)
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
    let password = methods.password;
    let notice = match provider {
        Provider::Discord => Notice::DiscordUnlinked { password },
        Provider::Google => Notice::GoogleUnlinked { password },
    };
    notify(state, user.id, notice);
    Ok(Json(methods))
}
