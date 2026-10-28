//! O despachante publica os eventos do outbox, em ordem, para os sockets e
//! apaga o que saiu.

use api::{outbox::Event, realtime};
use axum::http::StatusCode;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::COPA;

#[sqlx::test(migrations = "../../migrations")]
async fn publishes_in_order_and_deletes(db: PgPool) {
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
    let stored = outbox(&db).await;
    assert!(!stored.is_empty());

    let mut events = state.realtime.subscribe();
    let sent = realtime::dispatch_once(&state).await.unwrap();
    assert_eq!(sent, stored.len());
    assert_eq!(outbox(&db).await, Vec::<serde_json::Value>::new());
    let mut published = Vec::new();
    while let Ok(event) = events.try_recv() {
        published.push(serde_json::to_value(&*event).unwrap());
    }
    assert_eq!(published, stored);
    let last: Event = serde_json::from_value(published.pop().unwrap()).unwrap();
    let Event::Notice { users, .. } = last else {
        panic!("o último é o aviso do check-in: {last:?}");
    };
    assert_eq!(users, [bia.id]);

    // Nada mais a sair.
    assert_eq!(realtime::dispatch_once(&state).await.unwrap(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn unreadable_events_are_dropped(db: PgPool) {
    let state = state(db.clone());
    sqlx::query(r#"insert into outbox (event) values ('{"kind": "tenant"}')"#)
        .execute(&db)
        .await
        .unwrap();
    let mut events = state.realtime.subscribe();
    assert_eq!(realtime::dispatch_once(&state).await.unwrap(), 1);
    assert!(events.try_recv().is_err());
    assert_eq!(outbox(&db).await, Vec::<serde_json::Value>::new());
}
