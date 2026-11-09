//! Testes de `images`: o logo e a capa da comunidade, a capa do torneio e a foto de
//! perfil (pedir a URL, mandar ao bucket, registrar, trocar e remover), quem
//! pode, os limites e a rotina diária. O bucket é o S3 falso dos screenshots,
//! com o HTTP de verdade. As regras do banco ficam em `schema::images`; as
//! imagens das regras do torneio, com os helpers daqui, em
//! `tournaments::rules_images`.

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

pub const CDN: &str = "https://img.test";
const LOGO: &str = "/tenants/quake-br/logo";
const COVER: &str = "/tenants/quake-br/tournaments/copa/cover";
const TENANT_COVER: &str = "/tenants/quake-br/cover";
const CACHE: &str = "public, max-age=31536000, immutable";

/// App com o bucket público falso; `ana` é a owner de `quake-br`.
pub async fn app_with_bucket(db: &PgPool) -> (Router, AppState, FakeS3, User) {
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
pub async fn request(app: &Router, who: &User, base: &str, size: usize) -> (StatusCode, Value) {
    let body = json!({ "content_type": "image/jpeg", "size": size });
    post(app, &format!("{base}/uploads"), who, body).await
}

/// O PUT do navegador, com os cabeçalhos que a resposta pede.
pub async fn put_to_bucket(upload: &Value, body: Vec<u8>) -> StatusCode {
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

pub async fn attach(app: &Router, who: &User, base: &str, id: &Value) -> (StatusCode, Value) {
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
pub fn key(url: &str) -> String {
    url.strip_prefix(&format!("{CDN}/")).unwrap().to_owned()
}

/// Na ordem de bytes, a do `sort` do Rust (os nomes têm letras aleatórias, e a
/// collation do banco ordena maiúsculas de outro jeito).
pub async fn queued(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar(r#"select object_key from image_deletions order by object_key collate "C""#)
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

/// A capa da comunidade é do owner, fica na pasta das capas, sem a do torneio,
/// e chega a quem monta a prévia do convite e os tiles sem capa.
#[tokio::test]
async fn the_owner_sets_the_community_cover() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, ana) = app_with_bucket(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;
    let (status, body) = request(&app, &carla, TENANT_COVER, 1000).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );

    let url = send(&app, &ana, TENANT_COVER).await;
    assert!(url.starts_with(&format!("{CDN}/covers/quake-br/")), "{url}");
    assert_eq!(key(&url).matches('/').count(), 2, "{url}");
    let (_, tenant) = get(&app, "/tenants/quake-br", &ana).await;
    assert_eq!(tenant["cover_url"], url);
    assert_eq!(tenant["logo_url"], Value::Null);
    let (_, mine) = get(&app, "/tenants", &ana).await;
    assert_eq!(mine[0]["cover_url"], url);
    let token = invite(&app, &ana, "quake-br").await;
    let (_, preview) = get(&app, &format!("/invites/{token}"), &ana).await;
    assert_eq!(preview["tenant_cover_url"], url);

    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    let (_, mine) = get(&app, "/me/tournaments", &ana).await;
    assert_eq!(mine["open"][0]["tenant_cover_url"], url, "{mine}");

    // Um upload da capa não vale para o logo, e o do logo não vale para a capa.
    let (_, upload) = request(&app, &ana, TENANT_COVER, 1000).await;
    let (status, body) = attach(&app, &ana, LOGO, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("images.not_found"))
    );
    let (_, upload) = request(&app, &ana, LOGO, 1000).await;
    let (status, body) = attach(&app, &ana, TENANT_COVER, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("images.not_found"))
    );

    let (status, body) = delete(&app, TENANT_COVER, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, tenant) = get(&app, "/tenants/quake-br", &ana).await;
    assert_eq!(tenant["cover_url"], Value::Null);
    assert_eq!(queued(&db).await, vec![key(&url)]);
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
    // Só JPEG.
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

/// Objetos não se movem: o que foi enviado com o endereço de antes continua
/// valendo, e o que foi pedido antes da troca ainda pode ser registrado.
#[tokio::test]
async fn logo_and_cover_outlive_a_new_address() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, state, s3, ana) = app_with_bucket(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let logo = send(&app, &ana, LOGO).await;
    let cover = send(&app, &ana, COVER).await;
    let (_, pending_logo) = request(&app, &ana, LOGO, 1000).await;
    let (_, pending_cover) = request(&app, &ana, COVER, 1000).await;
    for upload in [&pending_logo, &pending_cover] {
        assert_eq!(put_to_bucket(upload, vec![7; 1000]).await, StatusCode::OK);
    }

    let moved = json!({ "slug": "quake-brasil" });
    let (status, body) = patch(&app, "/tenants/quake-br", &ana, moved).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["logo_url"], logo);
    const NEW_LOGO: &str = "/tenants/quake-brasil/logo";
    const DRAFT: &str = "/tenants/quake-brasil/tournaments/copa";
    let (_, draft) = get(&app, DRAFT, &ana).await;
    assert_eq!(draft["cover_url"], cover);
    assert!(s3.object(&key(&logo)).is_some());

    // O rascunho também troca, com a capa onde está.
    let (status, body) = patch(&app, DRAFT, &ana, json!({ "slug": "liga" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["cover_url"], cover);
    const NEW_COVER: &str = "/tenants/quake-brasil/tournaments/liga/cover";
    assert!(queued(&db).await.is_empty());

    // O pedido de antes da troca registra, na pasta que tinha.
    let (status, image) = attach(&app, &ana, NEW_LOGO, &pending_logo["id"]).await;
    assert_eq!(status, StatusCode::OK, "{image}");
    let second_logo = image["url"].as_str().unwrap().to_owned();
    assert!(
        second_logo.starts_with(&format!("{CDN}/logos/quake-br/")),
        "{second_logo}"
    );
    let (status, image) = attach(&app, &ana, NEW_COVER, &pending_cover["id"]).await;
    assert_eq!(status, StatusCode::OK, "{image}");
    let second_cover = image["url"].as_str().unwrap().to_owned();
    assert!(
        second_cover.starts_with(&format!("{CDN}/covers/quake-br/copa/")),
        "{second_cover}"
    );

    // O que for enviado agora vai para a pasta do endereço novo.
    let third_logo = send(&app, &ana, NEW_LOGO).await;
    assert!(
        third_logo.starts_with(&format!("{CDN}/logos/quake-brasil/")),
        "{third_logo}"
    );
    let third_cover = send(&app, &ana, NEW_COVER).await;
    assert!(
        third_cover.starts_with(&format!("{CDN}/covers/quake-brasil/liga/")),
        "{third_cover}"
    );

    // A fila leva todos pelo nome exato, de qualquer pasta, sem tocar na
    // comunidade que pegar o endereço antigo.
    let eva = user(&db, "Eva").await;
    create(&app, &eva, "quake-br").await;
    let hers = send(&app, &eva, LOGO).await;
    assert!(
        hers.starts_with(&format!("{CDN}/logos/quake-br/")),
        "{hers}"
    );
    delete(&app, NEW_LOGO, &ana).await;
    delete(&app, NEW_COVER, &ana).await;
    let storage = state.public_storage.as_ref().unwrap();
    let later = Utc::now() + Duration::hours(1);
    let done = images::sweep(&db, storage, later).await.unwrap();
    assert_eq!((done.deleted, done.drained), (6, true));
    assert_eq!(s3.keys().into_iter().collect::<Vec<_>>(), [key(&hers)]);
}

const AVATAR: &str = "/me/avatar";
const DISCORD: &str = "https://cdn.discordapp.com/avatars/1/a.png";

/// O avatar e a foto de quem vê (`/me`) e o avatar dele na lista de membros.
async fn avatars(app: &Router, who: &User) -> (Value, Value, Value) {
    let (status, me) = get(app, "/me", who).await;
    assert_eq!(status, StatusCode::OK, "{me}");
    let (status, list) = get(app, "/tenants/quake-br/members", who).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let row = list
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["user_id"] == me["id"])
        .unwrap()
        .clone();
    (
        me["avatar_url"].clone(),
        me["photo_url"].clone(),
        row["avatar_url"].clone(),
    )
}

#[tokio::test]
async fn the_photo_replaces_the_discord_avatar_until_removed() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, state, s3, ana) = app_with_bucket(&db).await;
    sqlx::query("update users set avatar_url = $2 where id = $1")
        .bind(ana.id)
        .bind(DISCORD)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        avatars(&app, &ana).await,
        (json!(DISCORD), Value::Null, json!(DISCORD))
    );

    let first = send(&app, &ana, AVATAR).await;
    assert!(first.starts_with(&format!("{CDN}/avatars/")), "{first}");
    assert_eq!(
        avatars(&app, &ana).await,
        (json!(first), json!(first), json!(first))
    );
    let second = send(&app, &ana, AVATAR).await;
    assert_eq!(queued(&db).await, vec![key(&first)]);

    // Sem a foto, volta o do Discord.
    let (status, body) = delete(&app, AVATAR, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(
        avatars(&app, &ana).await,
        (json!(DISCORD), Value::Null, json!(DISCORD))
    );
    let mut both = vec![key(&first), key(&second)];
    both.sort();
    assert_eq!(queued(&db).await, both);
    let storage = state.public_storage.as_ref().unwrap();
    let done = images::sweep(&db, storage, Utc::now() + Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(done.deleted, 2);
    assert!(s3.keys().is_empty());
}

/// A foto só muda quando o arquivo chegou ao bucket, e só a própria pessoa
/// registra o upload dela; o que não chegou sai na rotina.
#[tokio::test]
async fn the_photo_changes_only_with_the_file_in_the_bucket() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, state, s3, ana) = app_with_bucket(&db).await;
    let bia = member(&app, &db, &ana, "Bia").await;
    let kept = send(&app, &ana, AVATAR).await;

    let (status, upload) = request(&app, &ana, AVATAR, 1000).await;
    assert_eq!(status, StatusCode::CREATED, "{upload}");
    let (status, body) = attach(&app, &ana, AVATAR, &upload["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("images.upload_missing"))
    );
    assert_eq!(avatars(&app, &ana).await.1, json!(kept));

    // O upload de outra pessoa, e o de um logo pela rota da foto (e o
    // contrário), não existem.
    assert_eq!(put_to_bucket(&upload, vec![7; 1000]).await, StatusCode::OK);
    let not_found = (StatusCode::NOT_FOUND, err("images.not_found"));
    assert_eq!(attach(&app, &bia, AVATAR, &upload["id"]).await, not_found);
    assert_eq!(attach(&app, &ana, LOGO, &upload["id"]).await, not_found);
    let (_, logo) = request(&app, &ana, LOGO, 1000).await;
    assert_eq!(put_to_bucket(&logo, vec![7; 1000]).await, StatusCode::OK);
    assert_eq!(attach(&app, &ana, AVATAR, &logo["id"]).await, not_found);
    assert_eq!(avatars(&app, &bia).await.1, Value::Null);

    // Nunca registrados, a foto e o logo saem do bucket; a foto de agora fica.
    let storage = state.public_storage.as_ref().unwrap();
    let done = images::sweep(&db, storage, Utc::now() + Duration::days(2))
        .await
        .unwrap();
    assert_eq!((done.abandoned, done.deleted), (2, 2));
    assert_eq!(s3.keys().into_iter().collect::<Vec<_>>(), vec![key(&kept)]);
    assert_eq!(avatars(&app, &ana).await.1, json!(kept));
}
