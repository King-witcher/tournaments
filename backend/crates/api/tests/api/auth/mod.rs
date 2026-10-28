//! Testes de `auth`, um arquivo por assunto. Aqui ficam o Discord falso e os
//! helpers que leem redirect e cookies, que os outros domínios não precisam.

mod discord;
mod link;
mod password;
mod profile;

use std::sync::Arc;

use api::auth::{AuthSettings, DiscordApi, DiscordUser};
use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Method, Request, Response, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::common::{User, WEB, app_with};

const REDIRECT: &str = "http://localhost:8080/auth/discord/callback";

struct FakeDiscord {
    user: Option<DiscordUser>,
}

#[async_trait]
impl DiscordApi for FakeDiscord {
    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> anyhow::Result<String> {
        assert_eq!(redirect_uri, REDIRECT);
        anyhow::ensure!(self.user.is_some(), "falha simulada");
        Ok(format!("token-{code}"))
    }

    async fn fetch_user(&self, _access_token: &str) -> anyhow::Result<DiscordUser> {
        self.user.clone().ok_or_else(|| anyhow::anyhow!("falha"))
    }
}

/// Usuário do Discord de id `123`.
fn discord_user(avatar: Option<&str>) -> DiscordUser {
    DiscordUser {
        id: "123".into(),
        username: "quaker".into(),
        global_name: Some("Quaker Global".into()),
        avatar: avatar.map(Into::into),
    }
}

/// App cujo Discord responde com `user` (`None` simula falha).
fn app(db: PgPool, user: Option<DiscordUser>, configured: bool) -> Router {
    app_with(
        db,
        Arc::new(FakeDiscord { user }),
        AuthSettings {
            discord_client_id: configured.then(|| "client-1".to_owned()),
            redirect_uri: REDIRECT.into(),
            web_origin: WEB.into(),
            cookie_domain: None,
            cookie_secure: false,
        },
    )
}

async fn send(app: &Router, req: Request<Body>) -> Response<Body> {
    app.clone().oneshot(req).await.unwrap()
}

fn get(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut b = Request::get(uri);
    if let Some(c) = cookie {
        b = b.header(header::COOKIE, c);
    }
    b.body(Body::empty()).unwrap()
}

fn location(res: &Response<Body>) -> String {
    res.headers()[header::LOCATION].to_str().unwrap().to_owned()
}

/// Set-Cookie completo (com atributos) do cookie de nome `name`.
fn set_cookie(res: &Response<Body>, name: &str) -> Option<String> {
    res.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .find(|c| c.starts_with(&format!("{name}=")))
}

/// Só `nome=valor` de um Set-Cookie.
fn pair(set_cookie: &str) -> String {
    set_cookie.split(';').next().unwrap().to_owned()
}

async fn body_text(res: Response<Body>) -> String {
    String::from_utf8(res.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap()
}

/// Requisição JSON que também devolve o cookie `session=..` criado, se houver.
async fn send_json(
    app: &Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value, Option<String>) {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(c) = cookie {
        req = req.header(header::COOKIE, c);
    }
    let res = send(app, req.body(Body::from(body.to_string())).unwrap()).await;
    let status = res.status();
    let session = set_cookie(&res, "session").map(|c| pair(&c));
    let text = body_text(res).await;
    let json = if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap()
    };
    (status, json, session)
}

/// Cria a conta pelo cadastro com senha e devolve o usuário com a sessão.
async fn signup(app: &Router, username: &str, password: &str) -> User {
    let (status, body, session) = send_json(
        app,
        Method::POST,
        "/auth/signup",
        None,
        serde_json::json!({ "username": username, "password": password }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    User {
        id: body["id"].as_i64().unwrap(),
        cookie: session.unwrap(),
    }
}

/// Abre o OAuth em `start` (login ou link) e devolve o cookie `oauth_state=..`
/// e o state que foi para o Discord.
async fn start_oauth(app: &Router, start: &str, session: Option<&str>) -> (String, String) {
    let res = send(app, get(start, session)).await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER, "{start}");
    let state_cookie = pair(&set_cookie(&res, "oauth_state").unwrap());
    let state = location(&res)
        .split("state=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap()
        .to_owned();
    (state_cookie, state)
}

/// Abre o OAuth em `start` e volta pelo callback, com a mesma sessão. Devolve
/// o cookie `session=..` criado (vazio se nenhum) e a resposta do callback.
async fn oauth_round_trip(
    app: &Router,
    start: &str,
    session: Option<&str>,
) -> (String, Response<Body>) {
    let (state_cookie, state) = start_oauth(app, start, session).await;
    let cookies = match session {
        Some(s) => format!("{state_cookie}; {s}"),
        None => state_cookie,
    };
    let res = send(
        app,
        get(
            &format!("/auth/discord/callback?code=abc&state={state}"),
            Some(&cookies),
        ),
    )
    .await;
    let session = set_cookie(&res, "session")
        .map(|c| pair(&c))
        .unwrap_or_default();
    (session, res)
}

/// Login com Discord voltando para `return_to`.
async fn do_login(app: &Router, return_to: &str) -> (String, Response<Body>) {
    oauth_round_trip(
        app,
        &format!("/auth/discord/login?return_to={return_to}"),
        None,
    )
    .await
}
