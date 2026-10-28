//! Helpers dos testes: app de teste, usuários com sessão, requisições e os
//! passos que vários domínios repetem (criar tenant, convidar, entrar, criar
//! e publicar torneio).

use std::sync::Arc;

use api::{
    AppState,
    auth::{AuthSettings, DiscordApi, HttpDiscord},
    shared::token::hash_token,
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

pub const WEB: &str = "http://localhost:3000";

/// App com login do Discord desligado (os testes criam sessões direto no banco).
pub fn app(db: PgPool) -> Router {
    app_with(
        db,
        Arc::new(HttpDiscord::new(String::new(), String::new())),
        AuthSettings {
            discord_client_id: None,
            redirect_uri: String::new(),
            web_origin: WEB.into(),
            cookie_domain: None,
            cookie_secure: false,
        },
    )
}

pub fn app_with(db: PgPool, discord: Arc<dyn DiscordApi>, auth: AuthSettings) -> Router {
    api::app(
        AppState {
            db,
            discord,
            auth: Arc::new(auth),
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

pub async fn put_empty(app: &Router, uri: &str, user: &User) -> (StatusCode, Value) {
    call(app, Method::PUT, uri, Some(user), None).await
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

/// Cria o tenant `slug` (nome "Tenant {slug}") com `owner` como owner.
pub async fn create(app: &Router, owner: &User, slug: &str) -> Value {
    let (status, body) = post(
        app,
        "/tenants",
        owner,
        json!({ "name": format!("Tenant {slug}"), "slug": slug }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body
}

/// Gera um convite em `slug` e devolve o token.
pub async fn invite(app: &Router, by: &User, slug: &str) -> String {
    let (status, body) = post_empty(app, &format!("/tenants/{slug}/invites"), by).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["token"].as_str().unwrap().to_owned()
}

pub async fn join(app: &Router, who: &User, token: &str) -> Value {
    let (status, body) = post_empty(app, &format!("/invites/{token}/accept"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// Tenant `quake-br` com Ana (owner) e Bia (player).
pub async fn quake_br(db: &PgPool) -> (Router, User, User) {
    let app = app(db.clone());
    let ana = user(db, "Ana").await;
    let bia = user(db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    let token = invite(&app, &ana, "quake-br").await;
    join(&app, &bia, &token).await;
    (app, ana, bia)
}

/// Membro novo (player) de `quake-br`, convidado por `owner`.
pub async fn member(app: &Router, db: &PgPool, owner: &User, name: &str) -> User {
    let who = user(db, name).await;
    join(app, &who, &invite(app, owner, "quake-br").await).await;
    who
}

/// Membro novo de `quake-br` promovido a organizer por `owner`.
pub async fn organizer(app: &Router, db: &PgPool, owner: &User, name: &str) -> User {
    let who = member(app, db, owner, name).await;
    let uri = format!("/tenants/quake-br/members/{}/role", who.id);
    let (status, body) = put(app, &uri, owner, json!({ "role": "organizer" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    who
}

/// Início dos torneios de `duel`: longe o bastante para o check-in nunca
/// abrir sozinho nos testes que não mexem nele.
pub const FAR_START: &str = "2099-01-01T20:00:00Z";

/// Duelo público em dupla com reset, md3/md5, 16 vagas, 1 ban, sem não
/// repetição, começando em `FAR_START` com check-in de 30 minutos (o padrão).
pub fn duel(slug: &str) -> Value {
    json!({
        "slug": slug,
        "locale": "pt",
        "name": "Copa de Outubro",
        "description": "Duelo **aberto**.",
        "rules": "- md3\n- md5 na grande final",
        "mode": "duel",
        "max_participants": 16,
        "champion_rules": { "bans_per_game": 1, "no_repeat": null },
        "stages": [{
            "format": "elimination",
            "bracket": "double",
            "grand_final_reset": true,
            "best_of": 3,
            "finals_best_of": 5
        }],
        "starts_at": FAR_START
    })
}

/// `body` com os campos de `changes` trocados.
pub fn with(mut body: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

/// Cria o torneio (rascunho) em `quake-br`.
pub async fn create_tournament(app: &Router, by: &User, body: Value) -> Value {
    let (status, created) = post(app, "/tenants/quake-br/tournaments", by, body).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    created
}

/// Publica o torneio `slug` de `quake-br`: a inscrição abre.
pub async fn publish(app: &Router, by: &User, slug: &str) {
    let uri = format!("/tenants/quake-br/tournaments/{slug}/publish");
    let (status, body) = post_empty(app, &uri, by).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// Inscreve cada um de `who` no torneio `slug` de `quake-br` pelo botão.
pub async fn register(app: &Router, slug: &str, who: &[&User]) {
    let uri = format!("/tenants/quake-br/tournaments/{slug}/registration");
    for u in who {
        let (status, body) = post_empty(app, &uri, u).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    }
}

/// Põe o início do torneio `slug` a `minutes` de agora (negativo: já
/// passou), para a janela de check-in abrir sem esperar.
pub async fn starts_in(db: &PgPool, slug: &str, minutes: i32) {
    sqlx::query!(
        "update tournaments set starts_at = now() + make_interval(mins => $2) where slug = $1",
        slug,
        minutes
    )
    .execute(db)
    .await
    .unwrap();
}

/// Põe o torneio direto num status, sem passar pelas regras de transição
/// (ex.: encerrado, que ainda não tem rota).
pub async fn set_status(db: &PgPool, slug: &str, status: &str) {
    sqlx::query!(
        "update tournaments set status = $2 where slug = $1",
        slug,
        status
    )
    .execute(db)
    .await
    .unwrap();
}
