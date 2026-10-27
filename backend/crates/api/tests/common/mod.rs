//! Helpers dos testes de rotas que exigem login.
#![allow(dead_code)] // cada binário de teste usa uma parte

use std::sync::Arc;

use api::{
    AppState,
    auth::{AuthSettings, hash_token},
    discord::HttpDiscord,
};
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const WEB: &str = "http://localhost:3000";

pub fn app(db: PgPool) -> Router {
    api::app(
        AppState {
            db,
            discord: Arc::new(HttpDiscord::new(String::new(), String::new())),
            auth: Arc::new(AuthSettings {
                discord_client_id: None,
                redirect_uri: String::new(),
                web_origin: WEB.into(),
                cookie_domain: None,
                cookie_secure: false,
            }),
        },
        HeaderValue::from_static(WEB),
    )
}

pub struct User {
    pub id: i64,
    pub cookie: String,
}

/// Cria um usuário com sessão válida direto no banco, sem passar pelo Discord.
pub async fn user(db: &PgPool, display_name: &str) -> User {
    let id = sqlx::query_scalar!(
        "insert into users (display_name) values ($1) returning id",
        display_name
    )
    .fetch_one(db)
    .await
    .unwrap();
    let token = format!("token-{id}");
    sqlx::query!(
        "insert into sessions (id, user_id, expires_at) values ($1, $2, now() + interval '1 day')",
        &hash_token(&token)[..],
        id
    )
    .execute(db)
    .await
    .unwrap();
    User {
        id,
        cookie: format!("session={token}"),
    }
}

/// Faz a requisição e devolve status + corpo JSON (`Null` se vazio).
pub async fn call(
    app: &Router,
    method: Method,
    uri: &str,
    user: Option<&User>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(u) = user {
        req = req.header(header::COOKIE, &u.cookie);
    }
    let req = match body {
        Some(json) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| panic!("corpo não é JSON: {}", String::from_utf8_lossy(&bytes)))
    };
    (status, json)
}

pub async fn get(app: &Router, uri: &str, user: &User) -> (StatusCode, Value) {
    call(app, Method::GET, uri, Some(user), None).await
}

pub async fn post(app: &Router, uri: &str, user: &User, body: Value) -> (StatusCode, Value) {
    call(app, Method::POST, uri, Some(user), Some(body)).await
}

pub async fn post_empty(app: &Router, uri: &str, user: &User) -> (StatusCode, Value) {
    call(app, Method::POST, uri, Some(user), None).await
}

pub async fn put(app: &Router, uri: &str, user: &User, body: Value) -> (StatusCode, Value) {
    call(app, Method::PUT, uri, Some(user), Some(body)).await
}

pub async fn patch(app: &Router, uri: &str, user: &User, body: Value) -> (StatusCode, Value) {
    call(app, Method::PATCH, uri, Some(user), Some(body)).await
}

pub async fn delete(app: &Router, uri: &str, user: &User) -> (StatusCode, Value) {
    call(app, Method::DELETE, uri, Some(user), None).await
}

/// Corpo de erro da API.
pub fn err(code: &str) -> Value {
    json!({ "error": code })
}
