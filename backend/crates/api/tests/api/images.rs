//! Testes de `images`: o logo da comunidade e a capa do torneio (pedir a URL,
//! mandar ao bucket, registrar, trocar e remover), quem pode, os limites e a
//! rotina diária. O bucket é o S3 falso dos screenshots, com o HTTP de
//! verdade. As regras do banco ficam em `schema::images`.

use api::{
    AppState, images,
    shared::{Storage, set_public_url},
};
use axum::{Router, http::StatusCode};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::screenshots::FakeS3;

const CDN: &str = "https://img.test";
const LOGO: &str = "/tenants/quake-br/logo";
const COVER: &str = "/tenants/quake-br/tournaments/copa/cover";
const CACHE: &str = "public, max-age=31536000, immutable";

/// App com o bucket público falso; `ana` é a owner de `quake-br`.
async fn app_with_bucket(db: &PgPool) -> (Router, AppState, FakeS3, User) {
    let s3 = FakeS3::start().await;
    set_public_url(CDN);
    let state = AppState {
        public_storage: Some(Storage::new(s3.config()).unwrap()),
        ..state(db.clone())
    };
    let app = app_from(state.clone());
    let ana = user(db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    (app, state, s3, ana)
}

/// Pede a URL de upload de um arquivo de `size` bytes para `base` (o logo ou
/// a capa).
async fn request(app: &Router, who: &User, base: &str, size: usize) -> (StatusCode, Value) {
    let body = json!({ "content_type": "image/jpeg", "size": size });
    post(app, &format!("{base}/uploads"), who, body).await
}

/// O PUT do navegador, com os cabeçalhos que a resposta pede.
async fn put_to_bucket(upload: &Value, body: Vec<u8>) -> StatusCode {
    reqwest::Client::new()
        .put(upload["url"].as_str().unwrap())
        .header("content-type", upload["content_type"].as_str().unwrap())
        .header("cache-control", upload["cache_control"].as_str().unwrap())
        .header("if-none-match", "*")
        .body(body)
        .send()
        .await
        .unwrap()
        .status()
}

async fn attach(app: &Router, who: &User, base: &str, id: &Value) -> (StatusCode, Value) {
    post_empty(app, &format!("{base}/uploads/{id}/attach"), who).await
}

/// Pede, manda 1000 bytes e registra; devolve a URL no CDN.
async fn send(app: &Router, who: &User, base: &str) -> String {
    let (status, upload) = request(app, who, base, 1000).await;
    assert_eq!(status, StatusCode::CREATED, "{upload}");
    assert_eq!(upload["cache_control"], CACHE);
    assert_eq!(put_to_bucket(&upload, vec![7; 1000]).await, StatusCode::OK);
    let (status, image) = attach(app, who, base, &upload["id"]).await;
    assert_eq!(status, StatusCode::OK, "{image}");
    image["url"].as_str().unwrap().to_owned()
}

/// O nome do objeto de uma URL do CDN.
fn key(url: &str) -> String {
    url.strip_prefix(&format!("{CDN}/")).unwrap().to_owned()
}

async fn queued(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar("select object_key from image_deletions order by object_key")
        .fetch_all(db)
        .await
        .unwrap()
}

async fn logo_url(app: &Router, who: &User) -> Value {
    let (status, tenant) = get(app, "/tenants/quake-br", who).await;
    assert_eq!(status, StatusCode::OK, "{tenant}");
    tenant["logo_url"].clone()
}

#[tokio::test]
async fn the_owner_sets_replaces_and_removes_the_logo() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, state, s3, ana) = app_with_bucket(&db).await;

    let first = send(&app, &ana, LOGO).await;
    assert!(
        first.starts_with(&format!("{CDN}/logos/quake-br/")),
        "{first}"
    );
    assert_eq!(logo_url(&app, &ana).await, first);
    let (_, mine) = get(&app, "/tenants", &ana).await;
    assert_eq!(mine[0]["logo_url"], first);
    assert!(s3.object(&key(&first)).is_some());

    // Trocar põe o de antes na fila; remover, o atual.
    let second = send(&app, &ana, LOGO).await;
    assert_eq!(logo_url(&app, &ana).await, second);
    assert_eq!(queued(&db).await, vec![key(&first)]);
    let (status, body) = delete(&app, LOGO, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(logo_url(&app, &ana).await, Value::Null);
    let mut both = vec![key(&first), key(&second)];
    both.sort();
    assert_eq!(queued(&db).await, both);

    // A rotina só apaga depois de a URL de upload vencer.
    let storage = state.public_storage.as_ref().unwrap();
    let early = images::sweep(&db, storage, Utc::now()).await.unwrap();
    assert_eq!(early.deleted, 0);
    let later = Utc::now() + Duration::hours(1);
    let done = images::sweep(&db, storage, later).await.unwrap();
    assert_eq!((done.deleted, done.drained), (2, true));
    assert!(s3.keys().is_empty());
    assert!(queued(&db).await.is_empty());
}

#[tokio::test]
async fn only_the_owner_changes_the_logo() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, ana) = app_with_bucket(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;
    let (status, body) = request(&app, &carla, LOGO, 1000).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let (status, body) = delete(&app, LOGO, &carla).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
}

#[tokio::test]
async fn the_organization_sets_the_cover() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, ana) = app_with_bucket(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;
    create_tournament(&app, &carla, duel("copa")).await;

    let url = send(&app, &carla, COVER).await;
    assert!(
        url.starts_with(&format!("{CDN}/covers/quake-br/copa/")),
        "{url}"
    );
    let (_, copa) = get(&app, "/tenants/quake-br/tournaments/copa", &ana).await;
    assert_eq!(copa["cover_url"], url);
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &ana).await;
    assert_eq!(list[0]["cover_url"], url);

    // Um upload da capa não vale para o logo, nem o de outra pessoa.
    let (_, upload) = request(&app, &carla, COVER, 1000).await;
    let (status, body) = attach(&app, &ana, LOGO, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("images.not_found"))
    );
    let (status, body) = attach(&app, &ana, COVER, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("images.not_found"))
    );

    let (status, body) = delete(&app, COVER, &carla).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, copa) = get(&app, "/tenants/quake-br/tournaments/copa", &ana).await;
    assert_eq!(copa["cover_url"], Value::Null);
    assert_eq!(queued(&db).await, vec![key(&url)]);
}

#[tokio::test]
async fn the_upload_must_arrive_as_requested() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, s3, ana) = app_with_bucket(&db).await;
    let (_, upload) = request(&app, &ana, LOGO, 1000).await;
    let (status, body) = attach(&app, &ana, LOGO, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("images.upload_missing"))
    );

    // Outro tipo, gravado por fora da URL: o upload sai e o objeto vai para a fila.
    let object: String = sqlx::query_scalar("select object_key from image_uploads where id = $1")
        .bind(upload["id"].as_i64().unwrap())
        .fetch_one(&db)
        .await
        .unwrap();
    s3.put(&object, "image/png", vec![7; 1000]);
    let (status, body) = attach(&app, &ana, LOGO, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("images.upload_invalid")
        )
    );
    assert_eq!(queued(&db).await, vec![object]);
    assert_eq!(logo_url(&app, &ana).await, Value::Null);
}

#[tokio::test]
async fn uploads_are_limited() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, ana) = app_with_bucket(&db).await;
    // Só JPEG: a prévia dos links não desenha WebP.
    let webp = json!({ "content_type": "image/webp", "size": 1000 });
    let (status, body) = post(&app, &format!("{LOGO}/uploads"), &ana, webp).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("request.invalid_body")
        )
    );
    let too_big = 1024 * 1024 + 1;
    for size in [0, too_big] {
        let (status, body) = request(&app, &ana, LOGO, size).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(body["error"], "images.size_invalid");
        assert_eq!(body["params"]["max"], 1024 * 1024);
    }
    for _ in 0..10 {
        let (status, body) = request(&app, &ana, LOGO, 1000).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
    }
    let (status, body) = request(&app, &ana, LOGO, 1000).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert_eq!(body["error"], "images.too_many_uploads");
    assert_eq!(body["params"]["max"], 10);
}

#[tokio::test]
async fn the_sweep_deletes_abandoned_uploads() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, state, s3, ana) = app_with_bucket(&db).await;
    let (_, upload) = request(&app, &ana, LOGO, 1000).await;
    assert_eq!(put_to_bucket(&upload, vec![7; 1000]).await, StatusCode::OK);
    let kept = send(&app, &ana, LOGO).await;

    let storage = state.public_storage.as_ref().unwrap();
    let done = images::sweep(&db, storage, Utc::now() + Duration::days(2))
        .await
        .unwrap();
    assert_eq!((done.abandoned, done.deleted), (1, 1));
    assert_eq!(s3.keys().into_iter().collect::<Vec<_>>(), vec![key(&kept)]);
}

#[tokio::test]
async fn without_the_bucket_there_are_no_uploads() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let (status, settings) = get(&app, "/images/settings", &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(settings["enabled"], false);
    let (status, body) = request(&app, &ana, LOGO, 1000).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            err("images.not_configured")
        )
    );
}
