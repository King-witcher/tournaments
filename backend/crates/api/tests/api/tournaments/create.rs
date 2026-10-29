//! Criação com traduções e publicação numa requisição só.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

const URI: &str = "/tenants/quake-br/tournaments";

fn es(name: &str) -> Value {
    json!({ "locale": "es", "name": name, "description": "Duelo abierto.", "rules": "- bo3" })
}

async fn assert_nothing_created(app: &axum::Router, ana: &User, slug: &str) {
    assert_eq!(
        get(app, &format!("{URI}/{slug}"), ana).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_without_the_new_fields_is_a_draft(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let created = create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(created["status"], "draft");
    assert_eq!(created["translations"], json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_with_publish_opens_registration(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let created =
        create_tournament(&app, &ana, with(duel("copa"), json!({ "publish": true }))).await;
    assert_eq!(created["status"], "registration");
    let (status, seen) = get(&app, &format!("{URI}/copa"), &bia).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seen["status"], "registration");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_with_publish_inside_the_check_in_window_opens_check_in(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let soon = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();
    let body = with(
        duel("copa"),
        json!({ "publish": true, "starts_at": soon, "check_in_minutes": 30 }),
    );
    let created = create_tournament(&app, &ana, body).await;
    assert_eq!(created["status"], "check_in");
    let (_, seen) = get(&app, &format!("{URI}/copa"), &ana).await;
    assert_eq!(seen["status"], "check_in");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_with_publish_without_a_date_stays_in_registration(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let body = with(
        duel("copa"),
        json!({ "publish": true, "starts_at": null, "translations": [es("Copa de Octubre")] }),
    );
    let created = create_tournament(&app, &ana, body).await;
    assert_eq!(created["status"], "registration");
    assert_eq!(created["starts_at"], Value::Null);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_stores_the_translations(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let body = with(
        duel("copa"),
        json!({ "translations": [es("  Copa de Octubre ")] }),
    );
    let created = create_tournament(&app, &ana, body).await;
    let expected = json!([{
        "locale": "es",
        "name": "Copa de Octubre",
        "description": "Duelo abierto.",
        "rules": "- bo3"
    }]);
    assert_eq!(created["translations"], expected);
    let (_, seen) = get(&app, &format!("{URI}/copa"), &ana).await;
    assert_eq!(seen["translations"], expected);
}

#[sqlx::test(migrations = "../../migrations")]
async fn translation_in_the_original_locale_is_refused(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let pt = json!({ "locale": "pt", "name": "Copa" });
    let body = with(duel("copa"), json!({ "translations": [pt] }));
    assert_eq!(
        post(&app, URI, &ana, body).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "tournament.translation_same_locale", "params": { "locale": "pt" } })
        )
    );
    assert_nothing_created(&app, &ana, "copa").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn translation_text_errors_carry_the_locale(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let body = with(duel("copa"), json!({ "translations": [es("  ")] }));
    assert_eq!(
        post(&app, URI, &ana, body).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "tournament.name_invalid", "params": { "locale": "es" } })
        )
    );
    let long = json!({ "locale": "es", "name": "Copa", "rules": "x".repeat(20001) });
    let body = with(duel("copa"), json!({ "translations": [long] }));
    assert_eq!(
        post(&app, URI, &ana, body).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "tournament.rules_invalid", "params": { "max": 20000, "locale": "es" } })
        )
    );
    assert_nothing_created(&app, &ana, "copa").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_same_translation_locale_twice_is_refused(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let body = with(
        duel("copa"),
        json!({ "translations": [es("Uno"), es("Dos")] }),
    );
    assert_eq!(
        post(&app, URI, &ana, body).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("request.invalid_body")
        )
    );
    assert_nothing_created(&app, &ana, "copa").await;
}
