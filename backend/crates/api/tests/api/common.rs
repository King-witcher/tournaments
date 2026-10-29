//! Helpers dos testes: app de teste, usuários com sessão, requisições, a
//! caixa de emails e os passos que vários domínios repetem (criar tenant,
//! convidar, entrar, criar e publicar torneio).

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};

use api::{
    AppState,
    auth::{AuthSettings, HttpDiscord, HttpGoogle, create_session},
    mail::{Email, Mail, Mailer},
    realtime::Realtime,
    shared::Redis,
};
use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use redis::Commands;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::sync::Notify;
use tower::ServiceExt;

pub const WEB: &str = "http://localhost:4001";

/// Chaves de execução anterior que já podem sair do Redis local: as sessões
/// duram 30 dias, e um teste nunca passa de uma hora.
const OLD_RUN_MS: i64 = 60 * 60 * 1000;

/// Prefixo das chaves desta execução. Ao montar, apaga as de execuções com
/// mais de uma hora; as de uma execução em andamento (outro worktree) ficam.
static RUN_PREFIX: LazyLock<String> = LazyLock::new(|| {
    let now = chrono::Utc::now().timestamp_millis();
    sweep_runs_before(now - OLD_RUN_MS);
    format!("test:{now}:")
});

/// `REDIS_URL` ou o Redis local, no database 1 (o do projeto).
fn redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/1".into())
}

fn sweep_runs_before(before: i64) {
    let mut conn = redis::Client::open(redis_url())
        .and_then(|c| c.get_connection())
        .expect("os testes precisam do Redis (REDIS_URL ou redis://127.0.0.1:6379/1)");
    let old: Vec<String> = conn
        .scan_match::<_, String>("test:*")
        .unwrap()
        .map(Result::unwrap)
        .filter(|key| {
            key.split(':')
                .nth(1)
                .and_then(|run| run.parse::<i64>().ok())
                .is_some_and(|run| run < before)
        })
        .collect();
    for keys in old.chunks(500) {
        let _: () = conn.del(keys).unwrap();
    }
}

/// Redis do teste: cada banco do `sqlx::test` ganha um prefixo nesta execução,
/// então o app e os helpers do mesmo teste veem as mesmas chaves, e um teste
/// não vê as do outro.
pub fn redis(db: &PgPool) -> Redis {
    let options = db.connect_options();
    let database = options.get_database().unwrap_or("postgres");
    Redis::open(&redis_url(), &format!("{}{database}:", *RUN_PREFIX)).unwrap()
}

/// Teto de emails por dia do app de teste.
pub const DAILY_EMAILS: u32 = 20;

/// Emails que o app de teste mandaria, na ordem em que saíram.
#[derive(Default)]
pub struct Mailbox {
    sent: Mutex<VecDeque<Email>>,
    arrived: Notify,
}

#[async_trait]
impl Mailer for Mailbox {
    async fn send(&self, email: &Email) -> anyhow::Result<()> {
        self.sent.lock().unwrap().push_back(email.clone());
        self.arrived.notify_one();
        Ok(())
    }
}

impl Mailbox {
    /// O próximo email; o envio é em segundo plano, então espera um pouco.
    pub async fn next(&self) -> Email {
        let wait = async {
            loop {
                if let Some(email) = self.sent.lock().unwrap().pop_front() {
                    return email;
                }
                self.arrived.notified().await;
            }
        };
        tokio::time::timeout(Duration::from_secs(5), wait)
            .await
            .expect("nenhum email saiu")
    }

    /// Nenhum email saiu (depois de dar tempo ao envio em segundo plano).
    pub async fn assert_empty(&self) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let sent = self.sent.lock().unwrap();
        assert!(sent.is_empty(), "saiu email: {:?}", sent.front());
    }
}

/// Caixa de emails do app de teste deste banco: o app e os helpers do mesmo
/// teste veem a mesma.
pub fn mailbox(db: &PgPool) -> Arc<Mailbox> {
    static BOXES: LazyLock<Mutex<HashMap<String, Arc<Mailbox>>>> = LazyLock::new(Default::default);
    let options = db.connect_options();
    let database = options.get_database().unwrap_or("postgres").to_owned();
    BOXES.lock().unwrap().entry(database).or_default().clone()
}

/// Token do link de um email (o que vem depois do `#`).
pub fn link_token(email: &Email) -> String {
    let (_, rest) = email.text.split_once('#').expect("email sem link");
    rest.split_whitespace().next().unwrap().to_owned()
}

/// Estado do app de teste: login do Discord e do Google desligado (os testes
/// criam sessões direto no Redis), Web Push desligado, emails na
/// `mailbox(&db)` e nenhuma tarefa em segundo plano (os testes do tempo real
/// chamam o despachante e o agendador direto).
pub fn state(db: PgPool) -> AppState {
    AppState {
        redis: redis(&db),
        mail: Mail::new(mailbox(&db), DAILY_EMAILS),
        db,
        discord: Arc::new(HttpDiscord::new(String::new(), String::new())),
        google: Arc::new(HttpGoogle::new(String::new(), String::new())),
        auth: Arc::new(AuthSettings {
            discord: None,
            google: None,
            web_origin: WEB.into(),
            cookie_domain: None,
            cookie_secure: false,
        }),
        realtime: Realtime::new(),
        push: None,
    }
}

pub fn app(db: PgPool) -> Router {
    app_from(state(db))
}

pub fn app_from(state: AppState) -> Router {
    api::app(state, HeaderValue::from_static(WEB))
}

pub struct User {
    pub id: i64,
    pub cookie: String,
}

/// Cria um usuário direto no banco, com sessão no Redis, sem passar pelo login.
pub async fn user(db: &PgPool, display_name: &str) -> User {
    let id = sqlx::query_scalar!(
        "insert into users (display_name) values ($1) returning id",
        display_name
    )
    .fetch_one(db)
    .await
    .unwrap();
    let token = create_session(&redis(db), id, None).await.unwrap();
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
