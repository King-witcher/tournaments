//! Ajuste manual: só a organização, com motivo, autor e horário; entra no
//! reprocessamento na ordem do horário e não conta como jogo.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::play;

fn adjustments_uri(user: i64) -> String {
    format!("{RATINGS}/{user}/adjustments")
}

#[sqlx::test(migrations = "../../migrations")]
async fn organizer_adjusts_with_a_reason(db: PgPool) {
    let f = final_md3(&db, json!({})).await;
    let olga = organizer(&f.app, &db, &f.ana, "Olga").await;
    let (status, created) = post(
        &f.app,
        &adjustments_uri(f.p1.id),
        &olga,
        json!({ "delta": 50, "reason": "  Rating da liga antiga " }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(created["delta"], 50);
    assert_eq!(created["reason"], "Rating da liga antiga");
    assert_eq!(created["created_by"]["user_id"], olga.id);
    assert_eq!(created["created_by"]["nickname"], "Olga");
    // Sem jogo: provisório, sem último jogo.
    let r = ranking(&f.app, &f.bia).await;
    assert_eq!(table(&r), [(f.p1.id, 1550, 0, 0, 0, true)]);
    assert_eq!(r["players"][0]["last_game_at"], Value::Null);

    // Depois do ajuste, o primeiro mapa ainda tem K 80: 1550 contra 1500.
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    let p1 = member_rating(&f.app, &f.bia, f.p1.id).await;
    assert_eq!(p1["games"][1]["delta"], 34);
    assert_eq!(p1["adjusted"], 50);
    // O player vê a soma, não a lista; a organização vê a lista.
    assert_eq!(p1["adjustments"], Value::Null);
    let staff = member_rating(&f.app, &f.ana, f.p1.id).await;
    let list = staff["adjustments"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["reason"], "Rating da liga antiga");
}

#[sqlx::test(migrations = "../../migrations")]
async fn adjustment_goes_in_by_time(db: PgPool) {
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    let (status, body) = post(
        &f.app,
        &adjustments_uri(f.p2.id),
        &f.ana,
        json!({ "delta": -10, "reason": "Atraso" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    // Depois dos dois mapas: só tira 10 do P2, sem mexer no P1.
    assert_eq!(
        table(&ranking(&f.app, &f.bia).await),
        [
            (f.p1.id, 1569, 2, 2, 0, true),
            (f.p2.id, 1421, 2, 0, 2, true)
        ]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn who_can_adjust_and_what(db: PgPool) {
    let f = final_md3(&db, json!({})).await;
    let ok = json!({ "delta": 10, "reason": "x" });
    let (status, body) = post(&f.app, &adjustments_uri(f.p1.id), &f.bia, ok.clone()).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let (status, body) = post(&f.app, &adjustments_uri(999_999), &f.ana, ok).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
    for delta in [0, 1001, -1001] {
        let (status, body) = post(
            &f.app,
            &adjustments_uri(f.p1.id),
            &f.ana,
            json!({ "delta": delta, "reason": "x" }),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{delta}");
        assert_eq!(
            body,
            json!({ "error": "rating.delta_invalid", "params": { "max": 1000 } })
        );
    }
    for reason in ["   ".to_owned(), "é".repeat(281)] {
        let (status, body) = post(
            &f.app,
            &adjustments_uri(f.p1.id),
            &f.ana,
            json!({ "delta": 5, "reason": reason }),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "error": "rating.reason_invalid", "params": { "max": 280 } })
        );
    }
    assert_eq!(ranking(&f.app, &f.bia).await["players"], json!([]));
}
