//! Avisos no Discord por webhook: a configuração do owner (`settings`) e o
//! envio pela fila (`delivery`), com um Discord de mentira (um servidor axum
//! que guarda o que recebe e responde o que o teste manda).

mod delivery;
mod settings;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use api::{AppState, discord::Webhooks};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

/// O token do webhook dos testes (68 caracteres, como os do Discord).
pub const TOKEN: &str = "tok_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_xy";
pub const WEBHOOK: &str = "424242424242";

/// O endereço que o owner cola.
pub fn webhook_url() -> String {
    format!("https://discord.com/api/webhooks/{WEBHOOK}/{TOKEN}")
}

/// O que o Discord de mentira responde à próxima requisição (sem roteiro, 204).
#[derive(Clone, Copy, Debug)]
pub enum Reply {
    Status(u16),
    /// 429 com `retry_after` em segundos.
    TooMany(f64),
    /// 204, mas a janela de ritmo acabou: esperar `reset_after` segundos.
    Spent(f64),
}

/// O que o Discord de mentira recebeu: o caminho e o corpo.
#[derive(Clone, Debug)]
pub struct Hit {
    pub path: String,
    pub body: Value,
}

#[derive(Default)]
struct Inner {
    hits: Mutex<Vec<Hit>>,
    script: Mutex<VecDeque<Reply>>,
}

#[derive(Clone)]
pub struct Fake {
    pub base: String,
    inner: Arc<Inner>,
}

impl Fake {
    pub async fn start() -> Self {
        let inner = Arc::new(Inner::default());
        let app = Router::new()
            .route("/api/webhooks/{id}/{token}", post(receive))
            .with_state(inner.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self { base, inner }
    }

    /// As respostas das próximas requisições, na ordem.
    pub fn reply(&self, replies: &[Reply]) {
        self.inner
            .script
            .lock()
            .unwrap()
            .extend(replies.iter().copied());
    }

    pub fn hits(&self) -> Vec<Hit> {
        self.inner.hits.lock().unwrap().clone()
    }

    /// Os textos das mensagens recebidas.
    pub fn texts(&self) -> Vec<String> {
        self.hits()
            .iter()
            .map(|h| h.body["content"].as_str().unwrap().to_owned())
            .collect()
    }
}

async fn receive(
    State(inner): State<Arc<Inner>>,
    Path((id, token)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> Response {
    inner.hits.lock().unwrap().push(Hit {
        path: format!("/api/webhooks/{id}/{token}"),
        body,
    });
    let reply = inner.script.lock().unwrap().pop_front();
    let header = |name: &'static str, value: String| {
        (
            HeaderName::from_static(name),
            HeaderValue::from_str(&value).unwrap(),
        )
    };
    match reply {
        None => StatusCode::NO_CONTENT.into_response(),
        Some(Reply::Status(code)) => StatusCode::from_u16(code).unwrap().into_response(),
        Some(Reply::TooMany(after)) => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "message": "You are being rate limited.", "retry_after": after, "global": false })),
        )
            .into_response(),
        Some(Reply::Spent(after)) => (
            StatusCode::NO_CONTENT,
            [
                header("x-ratelimit-remaining", "0".into()),
                header("x-ratelimit-reset-after", after.to_string()),
            ],
        )
            .into_response(),
    }
}

/// O estado do app de teste falando com o Discord de mentira.
pub fn state_with(db: PgPool, fake: &Fake) -> AppState {
    AppState {
        webhooks: Webhooks::with_base(&fake.base),
        ..state(db)
    }
}

pub const URI: &str = "/tenants/quake-br/discord";

/// O corpo de configuração com `events` ligados.
pub fn config(events: &[&str]) -> Value {
    json!({
        "url": webhook_url(),
        "events": events,
        "locale": "pt",
        "mention_players": true
    })
}

/// `quake-br` com Ana (owner) e o webhook configurado com `events`.
pub struct Setup {
    pub state: AppState,
    pub app: axum::Router,
    pub fake: Fake,
    pub ana: User,
}

pub async fn configured(db: &PgPool, events: &[&str]) -> Setup {
    let fake = Fake::start().await;
    let state = state_with(db.clone(), &fake);
    let app = app_from(state.clone());
    let ana = user(db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let (status, body) = put(&app, URI, &ana, config(events)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    Setup {
        state,
        app,
        fake,
        ana,
    }
}

/// Liga o Discord de `user` (o id do usuário no Discord).
pub async fn link_discord(db: &PgPool, user: &User, discord_id: &str) {
    sqlx::query!(
        "insert into user_identities (provider, provider_user_id, user_id) values ('discord', $1, $2)",
        discord_id,
        user.id
    )
    .execute(db)
    .await
    .unwrap();
}
