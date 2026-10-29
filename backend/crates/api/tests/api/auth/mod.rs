//! Testes de `auth`, um arquivo por assunto. Aqui ficam o Discord falso e os
//! helpers que leem redirect e cookies, que os outros domínios não precisam.

mod discord;
mod email;
mod link;
mod notices;
mod password;
mod profile;
mod recovery;
mod sessions;
mod signup;

use std::sync::Arc;

use api::{
    auth::{AuthSettings, DiscordApi, DiscordUser},
    mail::Email,
    shared::{
        Redis,
        token::{hash_token, hex},
    },
};
use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Method, Request, Response, StatusCode, header},
};
use http_body_util::BodyExt;
use redis::Cmd;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use crate::common::{User, WEB, app_with, link_token, mailbox, redis};

const REDIRECT: &str = "http://localhost:4000/auth/discord/callback";

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

/// Chave no Redis da sessão do cookie `session=..`.
fn session_key(redis: &Redis, cookie: &str) -> String {
    let token = cookie.strip_prefix("session=").unwrap();
    redis.key(format_args!("session:{}", hex(&hash_token(token))))
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

/// Email do cadastro de `username` nos helpers.
fn email_of(username: &str) -> String {
    format!("{}@example.com", username.to_lowercase())
}

/// Cria a conta pelo cadastro com senha (email `email_of(username)`,
/// confirmado pelo link) e devolve o usuário com a sessão.
async fn signup(app: &Router, db: &PgPool, username: &str, password: &str) -> User {
    let email = email_of(username);
    let body = json!({ "username": username, "password": password, "email": email });
    let (status, body, _) = send_json(app, Method::POST, "/auth/signup", None, body).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    confirm_signup(app, &mailbox(db).next().await).await
}

/// Abre o link do email de cadastro e devolve a conta criada, com a sessão.
async fn confirm_signup(app: &Router, email: &Email) -> User {
    let token = link_token(email);
    let (status, body, session) = send_json(
        app,
        Method::POST,
        "/auth/signup/confirm",
        None,
        json!({ "token": token }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    User {
        id: body["id"].as_i64().unwrap(),
        cookie: session.unwrap(),
    }
}

/// Liga um Discord à conta direto no banco.
async fn add_discord(db: &PgPool, user_id: i64) {
    sqlx::query!(
        "insert into user_identities (user_id, provider, provider_user_id) values ($1, 'discord', $2)",
        user_id,
        format!("discord-{user_id}")
    )
    .execute(db)
    .await
    .unwrap();
}

/// Grava o email da conta direto no banco, como se já tivesse confirmado.
async fn set_email(db: &PgPool, user_id: i64, email: &str) {
    sqlx::query!("update users set email = $2 where id = $1", user_id, email)
        .execute(db)
        .await
        .unwrap();
}

/// Quantas sessões a conta de `who` tem abertas.
async fn session_count(app: &Router, who: &User) -> usize {
    let (status, body) = crate::common::get(app, "/auth/sessions", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().unwrap().len()
}

/// Libera um envio novo para `address` antes da espera entre envios.
async fn skip_cooldown(db: &PgPool, address: &str) {
    let redis = redis(db);
    let key = redis.key(format_args!("email_cooldown:{address}"));
    redis.query::<()>(&Cmd::del(key)).await.unwrap();
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
