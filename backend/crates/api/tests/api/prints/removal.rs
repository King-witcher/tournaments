//! Quem remove um print, e o que leva os prints junto: o mapa anulado e o
//! desfazer o início. Tudo o que sai do banco vai para a fila de exclusão do
//! bucket.

use axum::http::StatusCode;
use serde_json::json;

use super::*;
use crate::common::*;
use crate::matches::{COPA, confirm, game as map, report};
use crate::realtime::{clear, outbox, tournament};

async fn remove(c: &Cup, who: &User, n: u8, id: &Value) -> (StatusCode, Value) {
    delete(&c.app, &format!("{}/{id}", prints_uri(FINAL, n)), who).await
}

#[tokio::test]
async fn the_uploader_removes_until_the_map_confirms_and_staff_always() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let (_, mine) = attach_print(&c.app, &c.p1, FINAL, 1).await;
    let (_, theirs) = attach_print(&c.app, &c.p2, FINAL, 1).await;
    let (_, kept) = attach_print(&c.app, &c.p1, FINAL, 1).await;
    let key = object_key(&db, &mine["id"]).await;

    let (status, body) = remove(&c, &c.p1, 1, &theirs["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("prints.forbidden"))
    );
    // Pelo número certo do mapa.
    let (status, _) = remove(&c, &c.p1, 2, &mine["id"]).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    clear(&db).await;
    let (status, _) = remove(&c, &c.p1, 1, &mine["id"]).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(outbox(&db).await.contains(&tournament("copa", Some(FINAL))));
    assert_eq!(queued(&db).await, [key]);

    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = confirm(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = remove(&c, &c.p1, 1, &kept["id"]).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("prints.game_confirmed"))
    );
    let list = listing(&c.app, &c.p1, FINAL).await;
    assert_eq!(game(&list, 1)["prints"][1]["can_remove"], false);
    let list = listing(&c.app, &c.ana, FINAL).await;
    assert_eq!(game(&list, 1)["prints"][1]["can_remove"], true);
    for print in [&theirs, &kept] {
        let (status, _) = remove(&c, &c.ana, 1, &print["id"]).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    assert_eq!(stored(&db).await, Vec::<String>::new());
    assert_eq!(queued(&db).await.len(), 3);
}

#[tokio::test]
async fn a_voided_map_takes_its_prints() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let (_, print) = attach_print(&c.app, &c.p1, FINAL, 1).await;
    let key = object_key(&db, &print["id"]).await;
    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    // A organização anula o mapa para ser jogado de novo.
    let uri = format!("{COPA}/matches/{FINAL}/games/1/replay");
    let (status, body) = post(&c.app, &uri, &c.ana, json!({ "comment": null })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let list = listing(&c.app, &c.p1, FINAL).await;
    assert_eq!(numbers(&list), [1]);
    assert_eq!(game(&list, 1)["prints"], json!([]));
    assert_eq!(queued(&db).await, [key]);
}

#[tokio::test]
async fn undoing_the_start_discards_every_print() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(&db, 3, json!({})).await;
    let (_, sent) = attach_print(&c.app, &c.p1, FINAL, 1).await;
    let (status, _) = report(&c.app, &c.p1, FINAL, 1, map("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    // O do próximo mapa, ainda não enviado, também sai.
    let (_, next) = attach_print(&c.app, &c.p2, FINAL, 2).await;
    let (_, pending) = request_upload(&c.app, &c.p2, FINAL, 2, "image/webp", 10).await;
    let mut keys = vec![
        object_key(&db, &sent["id"]).await,
        object_key(&db, &next["id"]).await,
        object_key(&db, &pending["id"]).await,
    ];
    keys.sort();
    let (status, body) = delete(&c.app, &format!("{COPA}/start"), &c.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(stored(&db).await, Vec::<String>::new());
    assert_eq!(queued(&db).await, keys);
}
