//! Imagens nas regras: o envio (o do logo, com cache de uma hora), quem pode,
//! o texto que só aceita as imagens registradas da comunidade, o limite por
//! torneio e a saída para a fila quando nenhum texto as usa mais. O bucket é
//! o falso de `images`.

use api::images;
use axum::{Router, http::StatusCode};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::images::{CDN, app_with_bucket, attach, key, put_to_bucket, queued, request};

const COPA: &str = "/tenants/quake-br/tournaments/copa";
const UPLOADS: &str = "/tenants/quake-br/tournaments/copa/rules/images";

/// Pede, manda e registra uma imagem das regras pela `copa`; devolve a URL.
async fn send(app: &Router, who: &User) -> String {
    let (status, upload) = request(app, who, UPLOADS, 1000).await;
    assert_eq!(status, StatusCode::CREATED, "{upload}");
    assert_eq!(upload["cache_control"], "public, max-age=3600");
    assert_eq!(put_to_bucket(&upload, vec![7; 1000]).await, StatusCode::OK);
    let (status, image) = attach(app, who, UPLOADS, &upload["id"]).await;
    assert_eq!(status, StatusCode::OK, "{image}");
    image["url"].as_str().unwrap().to_owned()
}

fn markdown(urls: &[&str]) -> String {
    let mut text = String::from("# Regras\n\n- md3");
    for url in urls {
        text.push_str(&format!("\n\n![mapa]({url})"));
    }
    text
}

async fn set_rules(app: &Router, who: &User, uri: &str, urls: &[&str]) -> (StatusCode, Value) {
    patch(app, uri, who, json!({ "rules": markdown(urls) })).await
}

async fn registered(db: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from rules_images")
        .fetch_one(db)
        .await
        .unwrap()
}

/// Registra imagens direto no banco, sem o bucket: `rules/quake-br/...`.
async fn register(db: &PgPool, n: usize) -> Vec<String> {
    let mut urls = Vec::new();
    for c in "ABCDEFGHJKLMNPQRSTUVWXYZ".chars().take(n) {
        let key = format!("rules/quake-br/abcdefghijkmnopqrstuv{c}.jpg");
        sqlx::query("insert into rules_images (tenant_id, object_key) select id, $1 from tenants where slug = 'quake-br'")
            .bind(&key)
            .execute(db)
            .await
            .unwrap();
        urls.push(format!("{CDN}/{key}"));
    }
    urls
}

#[tokio::test]
async fn the_organization_sends_rules_images() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, s3, ana) = app_with_bucket(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;
    let bia = member(&app, &db, &ana, "Bia").await;
    let (_, settings) = get(&app, "/images/settings", &bia).await;
    assert_eq!(settings["rules_image_prefix"], format!("{CDN}/rules/"));
    create_tournament(&app, &carla, duel("copa")).await;

    let url = send(&app, &carla).await;
    assert!(url.starts_with(&format!("{CDN}/rules/quake-br/")), "{url}");
    assert!(s3.object(&key(&url)).is_some());
    assert_eq!(registered(&db).await, 1);

    // O cache de uma hora está na assinatura: o de um ano é recusado.
    let (_, upload) = request(&app, &carla, UPLOADS, 1000).await;
    let mut year = upload.clone();
    year["cache_control"] = json!("public, max-age=31536000, immutable");
    assert_eq!(
        put_to_bucket(&year, vec![7; 1000]).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(put_to_bucket(&upload, vec![7; 1000]).await, StatusCode::OK);

    // Player não envia; o upload das regras não vale para a capa, nem o
    // contrário.
    let (status, body) = request(&app, &bia, UPLOADS, 1000).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let cover = format!("{COPA}/cover");
    let (_, upload) = request(&app, &carla, UPLOADS, 1000).await;
    assert_eq!(
        attach(&app, &carla, &cover, &upload["id"]).await,
        (StatusCode::NOT_FOUND, err("images.not_found"))
    );
    let (_, upload) = request(&app, &carla, &cover, 1000).await;
    assert_eq!(
        attach(&app, &carla, UPLOADS, &upload["id"]).await,
        (StatusCode::NOT_FOUND, err("images.not_found"))
    );
}

#[tokio::test]
async fn the_rules_take_only_images_sent_to_the_community() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, ana) = app_with_bucket(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let ours = send(&app, &ana).await;
    let (status, saved) = set_rules(&app, &ana, COPA, &[&ours]).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["rules"], markdown(&[&ours]));

    // A imagem de outra comunidade da mesma pessoa também não vale.
    create(&app, &ana, "quake-ar").await;
    let (status, _) = post(&app, "/tenants/quake-ar/tournaments", &ana, duel("copa")).await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, upload) = request(
        &app,
        &ana,
        "/tenants/quake-ar/tournaments/copa/rules/images",
        1000,
    )
    .await;
    assert_eq!(put_to_bucket(&upload, vec![7; 1000]).await, StatusCode::OK);
    let (_, theirs) = attach(
        &app,
        &ana,
        "/tenants/quake-ar/tournaments/copa/rules/images",
        &upload["id"],
    )
    .await;
    let theirs = theirs["url"].as_str().unwrap();

    let not_sent = format!("{CDN}/rules/quake-br/zyxwvutsrqpnmkjihgfedc.jpg");
    let html = format!("Mapa: <img src=\"{ours}\">");
    for rules in [
        markdown(&["https://example.com/mapa.jpg"]),
        markdown(&[theirs]),
        markdown(&[&not_sent]),
        html,
    ] {
        assert_eq!(
            patch(&app, COPA, &ana, json!({ "rules": rules })).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("tournament.rules_image_invalid")
            ),
            "{rules}"
        );
    }
    let (_, seen) = get(&app, COPA, &ana).await;
    assert_eq!(seen["rules"], markdown(&[&ours]));
}

#[tokio::test]
async fn ten_images_per_tournament_counting_the_translations() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, ana) = app_with_bucket(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let urls = register(&db, 11).await;
    let urls: Vec<&str> = urls.iter().map(String::as_str).collect();
    let too_many = (
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": "tournament.rules_images_too_many", "params": { "max": 10 } }),
    );
    assert_eq!(set_rules(&app, &ana, COPA, &urls).await, too_many);
    let (status, body) = set_rules(&app, &ana, COPA, &urls[..9]).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // A tradução repete as do original sem contar de novo; com duas novas,
    // passa.
    let es = format!("{COPA}/translations/es");
    let translated = |urls: &[&str]| json!({ "name": "Copa", "rules": markdown(urls) });
    let (status, body) = put(&app, &es, &ana, translated(&urls[..10])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(put(&app, &es, &ana, translated(&urls[8..])).await, too_many);
}

#[tokio::test]
async fn an_image_no_text_uses_leaves_the_bucket() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, state, s3, ana) = app_with_bucket(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let a = send(&app, &ana).await;
    let b = send(&app, &ana).await;
    let (status, body) = set_rules(&app, &ana, COPA, &[&a, &b]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Reenviar o mesmo texto não muda nada; tirar uma a põe na fila.
    set_rules(&app, &ana, COPA, &[&a, &b]).await;
    assert!(queued(&db).await.is_empty());
    let (status, body) = set_rules(&app, &ana, COPA, &[&b]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(queued(&db).await, vec![key(&a)]);

    // A tradução que usa a mesma segura a imagem até sair.
    let es = format!("{COPA}/translations/es");
    let translation = json!({ "name": "Copa", "rules": markdown(&[&b]) });
    assert_eq!(put(&app, &es, &ana, translation).await.0, StatusCode::OK);
    set_rules(&app, &ana, COPA, &[]).await;
    assert_eq!(queued(&db).await, vec![key(&a)]);
    assert_eq!(
        delete(&app, &es, &ana).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    let mut both = vec![key(&a), key(&b)];
    both.sort();
    assert_eq!(queued(&db).await, both);
    assert_eq!(registered(&db).await, 0);

    let storage = state.public_storage.as_ref().unwrap();
    let later = Utc::now() + Duration::hours(1);
    let done = images::sweep(&db, storage, later).await.unwrap();
    assert_eq!(done.deleted, 2);
    assert!(s3.keys().is_empty());
}

/// "Copiar torneio" cria outro com o texto das regras, URLs inclusive: a
/// imagem vale nos dois e só sai quando o último a deixa.
#[tokio::test]
async fn a_copy_shares_the_images_of_the_original() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _, ana) = app_with_bucket(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let image = send(&app, &ana).await;
    set_rules(&app, &ana, COPA, &[&image]).await;

    let copy = |slug: &str| {
        with(
            duel(slug),
            json!({
                "rules": markdown(&[&image]),
                "translations": [{ "locale": "es", "name": "Liga", "rules": markdown(&[&image]) }]
            }),
        )
    };
    let liga = create_tournament(&app, &ana, copy("liga")).await;
    assert_eq!(liga["rules"], markdown(&[&image]));
    // Em outra comunidade, não: a imagem é desta.
    create(&app, &ana, "quake-ar").await;
    let (status, body) = post(&app, "/tenants/quake-ar/tournaments", &ana, copy("liga")).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.rules_image_invalid")
        )
    );
    // Na tradução da criação, o erro diz o idioma.
    let external = with(
        duel("taca"),
        json!({ "translations": [{ "locale": "es", "name": "Taza", "rules": markdown(&["https://example.com/a.jpg"]) }] }),
    );
    let (status, body) = post(&app, "/tenants/quake-br/tournaments", &ana, external).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "tournament.rules_image_invalid", "params": { "locale": "es" } })
        )
    );

    let (status, _) = delete(&app, COPA, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(queued(&db).await.is_empty());
    let (status, _) = delete(&app, "/tenants/quake-br/tournaments/liga", &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(queued(&db).await, vec![key(&image)]);
}

#[tokio::test]
async fn unused_images_are_limited_and_swept() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, state, s3, ana) = app_with_bucket(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let used = send(&app, &ana).await;
    let unused = send(&app, &ana).await;
    set_rules(&app, &ana, COPA, &[&used]).await;

    // Um dia depois, a nunca usada sai; a do texto fica.
    let storage = state.public_storage.as_ref().unwrap();
    let done = images::sweep(&db, storage, Utc::now() + Duration::days(2))
        .await
        .unwrap();
    assert_eq!((done.unused, done.deleted), (1, 1));
    assert_eq!(s3.keys().into_iter().collect::<Vec<_>>(), [key(&used)]);
    assert!(s3.object(&key(&unused)).is_none());

    // Vinte registradas sem texto seguram o envio de mais.
    register(&db, 20).await;
    let (status, body) = request(&app, &ana, UPLOADS, 1000).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::TOO_MANY_REQUESTS,
            json!({ "error": "images.too_many_unused", "params": { "max": 20 } })
        )
    );
}

#[tokio::test]
async fn without_the_bucket_the_rules_take_no_image() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let image = register(&db, 1).await;
    let (_, settings) = get(&app, "/images/settings", &ana).await;
    assert_eq!(settings["rules_image_prefix"], Value::Null);
    assert_eq!(
        request(&app, &ana, UPLOADS, 1000).await,
        (
            StatusCode::SERVICE_UNAVAILABLE,
            err("images.not_configured")
        )
    );
    assert_eq!(
        set_rules(&app, &ana, COPA, &[&image[0]]).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.rules_image_invalid")
        )
    );
    let (status, body) = set_rules(&app, &ana, COPA, &[]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
