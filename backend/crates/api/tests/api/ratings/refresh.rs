//! O recálculo: correção e desfazer o início mudam o passado; o agendador e
//! a leitura recalculam o tenant marcado e emitem a mudança da comunidade só
//! quando o ranking mudou.

use api::realtime;
use axum::http::StatusCode;
use chrono::Utc;
use serde_json::json;

use super::*;
use crate::common::*;
use crate::matches::{COPA, confirm, game, play, report, set_game};
use crate::realtime::{clear, outbox, tenant};

#[tokio::test]
async fn a_correction_changes_the_past() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    assert_eq!(table(&ranking(&f.app, &f.bia).await)[0].1, 1569);
    // O mapa 2 passa a ser do P2: a série reabre em 1 x 1, e o segundo mapa
    // vira a zebra (+46 para quem tinha 1460).
    let (status, body) = set_game(&f.app, &f.ana, "wb-1-1", 2, game("b", "exile"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        table(&ranking(&f.app, &f.bia).await),
        [
            (f.p2.id, 1506, 2, 1, 1, true),
            (f.p1.id, 1494, 2, 1, 1, true)
        ]
    );
}

#[tokio::test]
async fn undoing_the_start_takes_the_maps_out() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    let (status, body) = report(&f.app, &f.p1, "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&f.app, &f.p2, "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(table(&ranking(&f.app, &f.bia).await).len(), 2);
    let (status, body) = delete(&f.app, &format!("{COPA}/start"), &f.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ranking(&f.app, &f.bia).await["players"], json!([]));
}

#[tokio::test]
async fn the_scheduler_recomputes_and_emits_once() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    clear(&db).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.rated, 1);
    assert_eq!(outbox(&db).await, [tenant("quake-br")]);

    // Nada mudou: nem recalcula de novo, nem emite.
    clear(&db).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.rated, 0);
    assert_eq!(table(&ranking(&f.app, &f.bia).await)[0].1, 1569);
    assert!(outbox(&db).await.is_empty());
}

#[tokio::test]
async fn reading_recomputes_what_the_scheduler_did_not_see_yet() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    clear(&db).await;
    assert_eq!(table(&ranking(&f.app, &f.bia).await)[0].1, 1569);
    assert_eq!(outbox(&db).await, [tenant("quake-br")]);
    clear(&db).await;
    member_rating(&f.app, &f.bia, f.p1.id).await;
    ranking(&f.app, &f.bia).await;
    assert!(outbox(&db).await.is_empty());
}

#[tokio::test]
async fn a_change_that_keeps_the_ranking_emits_nothing() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    ranking(&f.app, &f.bia).await;
    clear(&db).await;
    // Mudar a configuração para o mesmo valor marca e recalcula, mas o
    // cache sai igual.
    let (status, body) = put(
        &f.app,
        "/tenants/quake-br/ratings/settings",
        &f.ana,
        json!({ "initial": 1500, "k_min": 32, "k_max": 80, "k_decay": 10 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    ranking(&f.app, &f.bia).await;
    assert!(outbox(&db).await.is_empty());
}
