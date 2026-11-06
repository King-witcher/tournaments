//! `/me/notifications`: os avisos que o despachante guardou para cada pessoa,
//! o filtro entre lidas e não lidas, as páginas, o que abrir a página de um aviso (pelo
//! caminho) ou "Marcar todas como lidas" (até a mais recente da lista) marca e
//! o prazo de 30 dias. A contagem pelo WebSocket fica em `realtime::socket`.

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
    listed(app, who, "").await
}

async fn listed(app: &Router, who: &User, query: &str) -> Vec<Value> {
    page(app, who, query).await.0
}

/// Uma página e o `next` dela.
async fn page(app: &Router, who: &User, query: &str) -> (Vec<Value>, Value) {
    let (status, body) = get(app, &format!("/me/notifications{query}"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body["notifications"].as_array().unwrap().clone();
    (list, body["next"].clone())
}

/// Um aviso direto no banco que leva a `path`.
async fn stored_at(db: &PgPool, who: &User, path: &str) {
    sqlx::query(
        r#"insert into notifications (user_id, notice)
           values ($1, jsonb_build_object(
               'code', 'tournament.start_due',
               'params', jsonb_build_object('tournament', 'Liga'),
               'path', $2::text))"#,
    )
    .bind(who.id)
    .bind(path)
    .execute(db)
    .await
    .unwrap();
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
    // Nada de outra pessoa muda, e nada muda: as abas não precisam saber.
    clear(&db).await;
    for body in [
        json!({ "up_to": first + 1 }),
        json!({ "path": "/quake-br/liga" }),
    ] {
        let (_, body) = post(&app, "/me/notifications/read", &ana, body).await;
        assert_eq!(body, json!({ "unread": 0 }));
    }
    assert_eq!(outbox(&db).await, Vec::<Value>::new());
    assert_eq!(notifications(&app, &bia).await[0]["read"], false);
}

#[tokio::test]
async fn opening_a_page_marks_only_the_notices_of_that_exact_path() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let (ana, bia) = (user(&db, "Ana").await, user(&db, "Bia").await);
    stored_at(&db, &ana, "/quake-br/liga").await;
    stored_at(&db, &ana, "/quake-br/liga").await;
    stored_at(&db, &ana, "/quake-br/liga/matches/wb-1-1").await;
    stored_at(&db, &bia, "/quake-br/liga").await;

    let (status, body) = post(
        &app,
        "/me/notifications/read",
        &ana,
        json!({ "path": "/quake-br/liga" }),
    )
    .await;
    assert_eq!((status, body), (StatusCode::OK, json!({ "unread": 1 })));
    assert_eq!(
        outbox(&db).await,
        [json!({ "kind": "unread", "user": ana.id, "count": 1 })]
    );
    let unread = listed(&app, &ana, "?read=false").await;
    assert_eq!(unread.len(), 1);
    assert_eq!(unread[0]["notice"]["path"], "/quake-br/liga/matches/wb-1-1");
    assert_eq!(listed(&app, &bia, "?read=false").await.len(), 1);
}

#[tokio::test]
async fn the_filter_comes_before_the_limit() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    stored(&db, &ana, 2).await;
    let old = notifications(&app, &ana).await[0]["id"].as_i64().unwrap();
    for _ in 0..20 {
        stored(&db, &ana, 1).await;
    }
    sqlx::query("update notifications set read_at = now() where id > $1")
        .bind(old)
        .execute(&db)
        .await
        .unwrap();

    // A não lida antiga não fica atrás das 20 recentes.
    let unread = listed(&app, &ana, "?read=false").await;
    assert_eq!(unread.len(), 1);
    assert_eq!(unread[0]["id"], old);
    assert_eq!(unread[0]["read"], false);
    let (read, next) = page(&app, &ana, "?read=true").await;
    assert_eq!((read.len(), next), (20, Value::Null));
    assert!(read.iter().all(|n| n["read"] == true));
    assert!(
        notifications(&app, &ana)
            .await
            .iter()
            .all(|n| n["id"] != old)
    );

    let (status, body) = get(&app, "/me/notifications?read=talvez", &ana).await;
    assert_eq!(
        (status, body),
        (StatusCode::BAD_REQUEST, err("request.invalid_query"))
    );
}

#[tokio::test]
async fn pages_go_by_id() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    for _ in 0..25 {
        stored(&db, &ana, 0).await;
    }
    let (first, next) = page(&app, &ana, "").await;
    assert_eq!(first.len(), 20);
    assert_eq!(next, first[19]["id"]);

    // A que chega no meio fica na primeira página: a seguinte continua de onde
    // a anterior parou.
    stored(&db, &ana, 0).await;
    let (rest, end) = page(&app, &ana, &format!("?before={next}")).await;
    assert_eq!((rest.len(), end), (5, Value::Null));
    let ids: Vec<i64> = first
        .iter()
        .chain(&rest)
        .map(|n| n["id"].as_i64().unwrap())
        .collect();
    assert!(ids.is_sorted_by(|a, b| a > b), "{ids:?}");

    // `limit` muda o tamanho; exatamente o que há não deixa `next`.
    let (some, next) = page(&app, &ana, "?limit=3").await;
    assert_eq!((some.len(), next), (3, some[2]["id"].clone()));
    let (all, end) = page(&app, &ana, "?limit=26").await;
    assert_eq!((all.len(), end), (26, Value::Null));
    for bad in ["?limit=0", "?limit=101", "?before=x"] {
        let (status, body) = get(&app, &format!("/me/notifications{bad}"), &ana).await;
        assert_eq!(
            (status, body),
            (StatusCode::BAD_REQUEST, err("request.invalid_query")),
            "{bad}"
        );
    }
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
