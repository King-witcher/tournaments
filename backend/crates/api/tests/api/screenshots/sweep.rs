//! A rotina diária: os screenshots saem 30 dias depois do fim do torneio, os
//! uploads nunca registrados saem em um dia, e a fila de exclusão esvazia
//! apagando do bucket. Roda uma vez por dia, pelo agendador.

use api::{realtime, screenshots};
use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{confirm, game as map, report};

/// Depois de vencer a URL de upload de tudo o que o teste pediu (10 minutos)
/// e da folga do relógio do bucket (5).
fn after_uploads() -> DateTime<Utc> {
    Utc::now() + Duration::minutes(16)
}

async fn status_of(db: &PgPool) -> String {
    sqlx::query_scalar("select status from tournaments")
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn screenshots_leave_thirty_days_after_the_end() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 1, json!({})).await;
    let (_, screenshot) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    let key = object_key(&db, &screenshot["id"]).await;
    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = confirm(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(status_of(&db).await, "finished");
    let storage = c.state.storage.as_ref().unwrap();

    let early = screenshots::sweep(&db, storage, Utc::now() + Duration::days(29))
        .await
        .unwrap();
    assert_eq!((early.expired, early.deleted, early.drained), (0, 0, true));
    assert!(c.s3.keys().contains(&key));

    let late = screenshots::sweep(&db, storage, Utc::now() + Duration::days(31))
        .await
        .unwrap();
    assert_eq!((late.expired, late.deleted, late.drained), (1, 1, true));
    assert!(c.s3.keys().is_empty());
    assert!(stored(&db).await.is_empty());
    assert!(queued(&db).await.is_empty());
    let list = listing(&c.app, &c.p1, FINAL).await;
    assert_eq!(game(&list, 1)["screenshots"], json!([]));
}

#[tokio::test]
async fn abandoned_uploads_leave_after_a_day() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let (_, upload) = request_upload(&c.app, &c.p1, FINAL, 1, "image/webp", 10).await;
    assert_eq!(
        put_to_bucket(&upload, "image/webp", vec![1; 10]).await,
        StatusCode::OK
    );
    let (_, kept) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    let storage = c.state.storage.as_ref().unwrap();

    let soon = screenshots::sweep(&db, storage, Utc::now() + Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(soon.abandoned, 0);
    let later = screenshots::sweep(&db, storage, Utc::now() + Duration::hours(25))
        .await
        .unwrap();
    assert_eq!((later.abandoned, later.deleted), (1, 1));
    // O anexado fica, no banco e no bucket.
    assert_eq!(stored(&db).await, [object_key_of(&db, &kept).await]);
    assert_eq!(c.s3.keys().len(), 1);
    // A contagem de uploads das horas passadas sai.
    let counts: i64 = sqlx::query_scalar("select count(*) from screenshot_upload_counts")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(counts, 0);
}

async fn object_key_of(db: &PgPool, screenshot: &Value) -> String {
    object_key(db, &screenshot["id"]).await
}

#[tokio::test]
async fn the_scheduler_runs_it_once_a_day() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let (_, screenshot) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    let (status, _) = delete(
        &c.app,
        &format!("{}/{}", screenshots_uri(FINAL, 1), screenshot["id"]),
        &c.p1,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(queued(&db).await.len(), 1);

    let now = after_uploads();
    realtime::tick(&c.state, now).await.unwrap();
    assert!(queued(&db).await.is_empty());
    assert!(c.s3.keys().is_empty());
    assert_eq!(c.s3.delete_requests(), 1);

    // No mesmo dia, nem olha a fila.
    let (_, other) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    let (status, _) = delete(
        &c.app,
        &format!("{}/{}", screenshots_uri(FINAL, 1), other["id"]),
        &c.p1,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let again = screenshots::sweep_due(&c.state, now + Duration::hours(23))
        .await
        .unwrap();
    assert_eq!(again, None);
    assert_eq!(queued(&db).await.len(), 1);
    let next_day = screenshots::sweep_due(&c.state, now + Duration::hours(25))
        .await
        .unwrap();
    assert_eq!(next_day.map(|s| s.deleted), Some(1));
    assert_eq!(c.s3.delete_requests(), 2);
}

#[tokio::test]
async fn what_the_bucket_does_not_delete_waits_in_the_queue() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let mut keys = Vec::new();
    for _ in 0..2 {
        let (_, screenshot) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
        keys.push(object_key(&db, &screenshot["id"]).await);
        let uri = format!("{}/{}", screenshots_uri(FINAL, 1), screenshot["id"]);
        assert_eq!(delete(&c.app, &uri, &c.p1).await.0, StatusCode::NO_CONTENT);
    }
    c.s3.refuse_to_delete(&keys[0]);
    let storage = c.state.storage.as_ref().unwrap();
    let done = screenshots::sweep(&db, storage, after_uploads())
        .await
        .unwrap();
    assert_eq!((done.deleted, done.drained), (1, false));
    assert_eq!(queued(&db).await, [keys[0].clone()]);
    assert_eq!(
        c.s3.keys().into_iter().collect::<Vec<_>>(),
        [keys[0].clone()]
    );
    // A passada fica marcada: o bucket não leva outra tentativa a cada 10 s.
    let again = screenshots::sweep_due(&c.state, after_uploads())
        .await
        .unwrap();
    assert_eq!(again, None);
}

#[tokio::test]
async fn an_object_leaves_the_bucket_only_after_its_upload_url_expires() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let (_, screenshot) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    let key = object_key(&db, &screenshot["id"]).await;
    let uri = format!("{}/{}", screenshots_uri(FINAL, 1), screenshot["id"]);
    assert_eq!(delete(&c.app, &uri, &c.p1).await.0, StatusCode::NO_CONTENT);
    // Apagado com a URL valendo, o mesmo PUT gravaria o objeto de novo, sem
    // linha nem fila: ele espera.
    let storage = c.state.storage.as_ref().unwrap();
    let early = screenshots::sweep(&db, storage, Utc::now()).await.unwrap();
    assert_eq!((early.deleted, early.drained), (0, true));
    assert_eq!(queued(&db).await, std::slice::from_ref(&key));
    assert!(c.s3.keys().contains(&key));
    let later = screenshots::sweep(&db, storage, after_uploads())
        .await
        .unwrap();
    assert_eq!((later.deleted, later.drained), (1, true));
    assert!(c.s3.keys().is_empty());
}

#[tokio::test]
async fn a_deleted_community_takes_its_screenshots_from_the_bucket() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 1, json!({})).await;
    let (status, body) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = confirm(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(status_of(&db).await, "finished");
    let deletion = json!({ "confirm_slug": "quake-br" });
    let (status, body) = post(&c.app, "/tenants/quake-br/deletion", &c.ana, deletion).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(c.s3.keys().len(), 1);

    // Na data, a comunidade sai com os screenshots, e a rotina do mesmo dia os
    // tira do bucket.
    let later = Utc::now() + Duration::days(7) + Duration::minutes(1);
    let tick = realtime::tick(&c.state, later).await.unwrap().unwrap();
    assert_eq!((tick.deleted_tenants, tick.swept), (1, true));
    assert!(stored(&db).await.is_empty());
    assert!(queued(&db).await.is_empty());
    assert!(c.s3.keys().is_empty());
}
