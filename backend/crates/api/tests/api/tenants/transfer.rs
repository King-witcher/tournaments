//! Transferir a posse: o owner escolhe um membro ativo, digita o slug para
//! confirmar, o novo owner respeita o limite de 3 e o antigo vira organizer.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

async fn transfer(app: &Router, by: &User, to: &User, slug: &str) -> (StatusCode, Value) {
    post(
        app,
        "/tenants/quake-br/transfer-ownership",
        by,
        json!({ "user_id": to.id, "confirm_slug": slug }),
    )
    .await
}

async fn role(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, "/tenants/quake-br", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["role"].clone()
}

async fn owners(db: &PgPool) -> Vec<i64> {
    sqlx::query_scalar!("select user_id from tenant_members where role = 'owner' order by user_id")
        .fetch_all(db)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_owner_hands_over_and_becomes_organizer(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;

    let (status, body) = transfer(&app, &ana, &bia, "quake-br").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        (&body["slug"], &body["role"]),
        (&json!("quake-br"), &json!("organizer"))
    );
    assert_eq!(role(&app, &bia).await, "owner");
    assert_eq!(role(&app, &ana).await, "organizer");
    let (_, list) = get(&app, "/tenants/quake-br/members", &ana).await;
    assert_eq!(list[0]["user_id"], bia.id);
    assert_eq!(list[0]["role"], "owner");

    // Agora Ana sai e Bia, não; Ana já não transfere.
    assert_eq!(
        transfer(&app, &ana, &bia, "quake-br").await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        delete(&app, "/tenants/quake-br/me", &bia).await,
        (StatusCode::CONFLICT, err("member.owner_must_transfer"))
    );
    assert_eq!(
        delete(&app, "/tenants/quake-br/me", &ana).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(owners(&db).await, [bia.id]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_transfer_needs_the_slug_and_an_active_member(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let caio = member(&app, &db, &ana, "Caio").await;
    let eve = user(&db, "Eve").await;

    assert_eq!(
        transfer(&app, &ana, &bia, "quake").await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tenant.confirmation_mismatch")
        )
    );
    assert_eq!(
        transfer(&app, &ana, &ana, "quake-br").await,
        (StatusCode::CONFLICT, err("member.is_owner"))
    );
    assert_eq!(
        transfer(&app, &olga, &bia, "quake-br").await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        transfer(&app, &ana, &eve, "quake-br").await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
    let (status, _) = delete(&app, "/tenants/quake-br/me", &caio).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        transfer(&app, &ana, &caio, "quake-br").await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
    assert_eq!(owners(&db).await, [ana.id]);

    // Organizer também pode receber.
    let (status, body) = transfer(&app, &ana, &olga, " quake-br ").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(owners(&db).await, [olga.id]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_new_owner_stays_within_the_limit(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    for slug in ["bia-1", "bia-2", "bia-3"] {
        create(&app, &bia, slug).await;
    }

    let (status, body) = transfer(&app, &ana, &bia, "quake-br").await;
    assert_eq!(
        (status, body),
        (
            StatusCode::CONFLICT,
            json!({ "error": "member.owner_limit_reached", "params": { "max": 3 } })
        )
    );
    assert_eq!(role(&app, &ana).await, "owner");
    assert_eq!(role(&app, &bia).await, "player");

    // Passando uma das dela adiante, cabe.
    let token = invite(&app, &bia, "bia-3").await;
    join(&app, &ana, &token).await;
    let (status, body) = post(
        &app,
        "/tenants/bia-3/transfer-ownership",
        &bia,
        json!({ "user_id": ana.id, "confirm_slug": "bia-3" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = transfer(&app, &ana, &bia, "quake-br").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(role(&app, &bia).await, "owner");
}
