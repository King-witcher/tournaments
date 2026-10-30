//! `/me/notifications`: os avisos que o despachante guardou para cada pessoa,
//! o que abrir o sininho marca como visto (até a mais recente da lista) e o
//! prazo de 30 dias. A contagem pelo WebSocket fica em `realtime::socket`.

use api::realtime;
use axum::{
    Router,
    http::{Method, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::COPA;
use crate::realtime::{clear, outbox};
use crate::schema::{CHECK, exec, rejected, rule};

async fn notifications(app: &Router, who: &User) -> Vec<Value> {
    let (status, body) = get(app, "/me/notifications", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().unwrap().clone()
}

/// Um aviso direto no banco, criado há `days` dias.
async fn stored(db: &PgPool, who: &User, days: i64) {
    sqlx::query(
        r#"insert into notifications (user_id, notice, created_at)
           values ($1, '{"code": "tournament.start_due", "params": {"tournament": "Liga"}, "path": "/quake-br/liga"}', $2)"#,
    )
    .bind(who.id)
    .bind(Utc::now() - Duration::days(days))
    .execute(db)
    .await
    .unwrap();
}

#[tokio::test]
async fn requires_login() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    for (method, uri, body) in [
        (Method::GET, "/me/notifications", None),
        (
            Method::POST,
            "/me/notifications/read",
            Some(json!({ "up_to": 1 })),
        ),
    ] {
        let (status, got) = call(&app, method, uri, None, body).await;
        assert_eq!(
            (status, got),
            (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
        );
    }
}

#[tokio::test]
async fn dispatched_notices_stay_until_seen() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let (ana, bia) = (user(&db, "Ana").await, user(&db, "Bia").await);
    create(&app, &ana, "quake-br").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    let (status, _) = post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    // Só o despachante guarda.
    assert_eq!(notifications(&app, &bia).await, Vec::<Value>::new());
    realtime::dispatch_once(&state).await.unwrap();

    let list = notifications(&app, &bia).await;
    assert_eq!(list.len(), 1);
    let first = list[0]["id"].as_i64().unwrap();
    assert_eq!(
        list[0]["notice"],
        json!({
            "code": "check_in.opened",
            "params": { "tournament": "Copa de Outubro" },
            "path": "/quake-br/copa"
        })
    );
    assert_eq!(list[0]["read"], false);
    assert!(list[0]["created_at"].is_string());
    // O aviso era só da Bia.
    assert_eq!(notifications(&app, &ana).await, Vec::<Value>::new());

    // Chegou outro depois de a lista abrir: continua novo.
    stored(&db, &bia, 0).await;
    clear(&db).await;
    let (status, body) = post(
        &app,
        "/me/notifications/read",
        &bia,
        json!({ "up_to": first }),
    )
    .await;
    assert_eq!((status, body), (StatusCode::OK, json!({ "unread": 1 })));
    assert_eq!(
        outbox(&db).await,
        [json!({ "kind": "unread", "user": bia.id, "count": 1 })]
    );
    let read: Vec<(i64, bool)> = notifications(&app, &bia)
        .await
        .iter()
        .map(|n| (n["id"].as_i64().unwrap(), n["read"].as_bool().unwrap()))
        .collect();
    assert_eq!(read, [(first + 1, false), (first, true)]);
    // O id de outra pessoa não marca nada dela.
    let (_, body) = post(
        &app,
        "/me/notifications/read",
        &ana,
        json!({ "up_to": first + 1 }),
    )
    .await;
    assert_eq!(body, json!({ "unread": 0 }));
    assert_eq!(notifications(&app, &bia).await[0]["read"], false);
}

#[tokio::test]
async fn old_ones_expire_and_the_shape_is_checked() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    stored(&db, &ana, 31).await;
    stored(&db, &ana, 29).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.expired_notifications, 1);
    assert_eq!(notifications(&app, &ana).await.len(), 1);

    for bad in [
        "'[]'",
        r#"'{"code": "x.y", "params": {}}'"#,
        r#"'{"code": "x.y", "params": [], "path": "/a"}'"#,
        r#"'{"code": "x.y", "params": {}, "path": "a"}'"#,
    ] {
        let sql = format!(
            "insert into notifications (user_id, notice) values ({}, {bad})",
            ana.id
        );
        assert_eq!(
            rejected(exec(&db, &sql).await),
            rule(CHECK, "notifications_notice_valid")
        );
    }
}
