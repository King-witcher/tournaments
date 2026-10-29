//! Testes de `screenshots`: pedir a URL, mandar ao bucket e registrar, quem
//! anexa e os limites (`uploads`), quem remove e o que leva os screenshots
//! junto (`removal`), o screenshot obrigatório no envio (`required`) e a rotina
//! diária (`sweep`). O bucket é um S3 falso (`fake_s3`), com o HTTP de verdade.
//! As regras do banco ficam em `schema::screenshots`.

mod fake_s3;
mod removal;
mod required;
mod sweep;
mod uploads;

use api::{AppState, shared::Storage};
use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{COPA, copa, players, single, start};

pub use fake_s3::FakeS3;

/// App com o bucket falso.
pub async fn app_with_bucket(db: &PgPool) -> (Router, AppState, FakeS3) {
    let s3 = FakeS3::start().await;
    let state = AppState {
        storage: Some(Storage::new(s3.config()).unwrap()),
        ..state(db.clone())
    };
    (app_from(state.clone()), state, s3)
}

/// `quake-br` com bucket e a final `wb-1-1` em md`best_of` entre `p1` (lado
/// `a`) e `p2`, já iniciada; `ana` é a owner e não joga; `bia`, um player que
/// não joga.
pub struct Cup {
    pub app: Router,
    pub state: AppState,
    pub s3: FakeS3,
    pub ana: User,
    pub bia: User,
    pub p1: User,
    pub p2: User,
}

pub async fn cup(db: &PgPool, best_of: u8, changes: Value) -> Cup {
    let (app, state, s3) = app_with_bucket(db).await;
    let ana = user(db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let bia = member(&app, db, &ana, "Bia").await;
    let mut p = players(&app, db, &ana, 2).await;
    let body = with(copa(single(best_of, best_of)), changes);
    start(&app, &ana, body, &[&p[0], &p[1]]).await;
    let p2 = p.pop().unwrap();
    let p1 = p.pop().unwrap();
    Cup {
        app,
        state,
        s3,
        ana,
        bia,
        p1,
        p2,
    }
}

pub const FINAL: &str = "wb-1-1";

pub fn screenshots_uri(key: &str, n: u8) -> String {
    format!("{COPA}/matches/{key}/games/{n}/screenshots")
}

/// Pede a URL de upload de um arquivo de `size` bytes.
pub async fn request_upload(
    app: &Router,
    who: &User,
    key: &str,
    n: u8,
    content_type: &str,
    size: usize,
) -> (StatusCode, Value) {
    let body = json!({ "content_type": content_type, "size": size });
    post(app, &screenshots_uri(key, n), who, body).await
}

/// O PUT do navegador na URL do upload.
pub async fn put_to_bucket(upload: &Value, content_type: &str, body: Vec<u8>) -> StatusCode {
    let res = reqwest::Client::new()
        .put(upload["url"].as_str().unwrap())
        .header("content-type", content_type)
        .body(body)
        .send()
        .await
        .unwrap();
    res.status()
}

pub async fn attach(app: &Router, who: &User, key: &str, n: u8, id: &Value) -> (StatusCode, Value) {
    post_empty(
        app,
        &format!("{}/{id}/attach", screenshots_uri(key, n)),
        who,
    )
    .await
}

/// Pede a URL, manda um WebP de 1000 bytes ao bucket e registra.
pub async fn attach_screenshot(app: &Router, who: &User, key: &str, n: u8) -> (StatusCode, Value) {
    let (status, upload) = request_upload(app, who, key, n, "image/webp", 1000).await;
    if status != StatusCode::CREATED {
        return (status, upload);
    }
    let sent = put_to_bucket(&upload, "image/webp", vec![7; 1000]).await;
    assert_eq!(sent, StatusCode::OK);
    attach(app, who, key, n, &upload["id"]).await
}

/// Os screenshots da partida, como `who` vê.
pub async fn listing(app: &Router, who: &User, key: &str) -> Value {
    let (status, body) = get(app, &format!("{COPA}/matches/{key}/screenshots"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// O mapa `n` da listagem.
pub fn game(listing: &Value, n: i64) -> &Value {
    listing["games"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["number"] == n)
        .unwrap_or_else(|| panic!("sem o mapa {n}: {listing}"))
}

/// Os números dos mapas da listagem.
pub fn numbers(listing: &Value) -> Vec<i64> {
    listing["games"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["number"].as_i64().unwrap())
        .collect()
}

/// Nomes de objeto na fila de exclusão.
pub async fn queued(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar("select object_key from screenshot_deletions order by object_key")
        .fetch_all(db)
        .await
        .unwrap()
}

/// Nomes de objeto dos screenshots gravados (anexados ou não).
pub async fn stored(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar("select object_key from match_screenshots order by object_key")
        .fetch_all(db)
        .await
        .unwrap()
}

/// O nome do objeto do screenshot `id`.
pub async fn object_key(db: &PgPool, id: &Value) -> String {
    sqlx::query_scalar("select object_key from match_screenshots where id = $1")
        .bind(id.as_i64().unwrap())
        .fetch_one(db)
        .await
        .unwrap()
}
