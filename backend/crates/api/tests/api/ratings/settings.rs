//! A configuração do Elo: todos leem, só o owner muda, e mudar recalcula o
//! histórico inteiro.

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::play;

const SETTINGS: &str = "/tenants/quake-br/ratings/settings";

fn elo(initial: i32, k_min: i32, k_max: i32, k_decay: i32) -> serde_json::Value {
    json!({ "initial": initial, "k_min": k_min, "k_max": k_max, "k_decay": k_decay })
}

#[sqlx::test(migrations = "../../migrations")]
async fn defaults_and_who_changes(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let (status, body) = get(&app, SETTINGS, &bia).await;
    assert_eq!((status, body), (StatusCode::OK, elo(1500, 32, 80, 10)));
    let olga = organizer(&app, &db, &ana, "Olga").await;
    for who in [&bia, &olga] {
        let (status, body) = put(&app, SETTINGS, who, elo(1200, 20, 40, 5)).await;
        assert_eq!(
            (status, body),
            (StatusCode::FORBIDDEN, err("tenant.forbidden"))
        );
    }
    let (status, body) = put(&app, SETTINGS, &ana, elo(1200, 20, 40, 5)).await;
    assert_eq!((status, body), (StatusCode::OK, elo(1200, 20, 40, 5)));
    let (_, body) = get(&app, SETTINGS, &bia).await;
    assert_eq!(body, elo(1200, 20, 40, 5));
    assert_eq!(ranking(&app, &bia).await["initial"], 1200);
}

#[sqlx::test(migrations = "../../migrations")]
async fn invalid_settings(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let cases = [
        (
            elo(99, 32, 80, 10),
            "rating.initial_invalid",
            Some((100, 3000)),
        ),
        (elo(1500, 0, 80, 10), "rating.k_min_invalid", Some((1, 100))),
        (
            elo(1500, 32, 201, 10),
            "rating.k_max_invalid",
            Some((1, 200)),
        ),
        (
            elo(1500, 32, 80, 0),
            "rating.k_decay_invalid",
            Some((1, 100)),
        ),
        (elo(1500, 50, 40, 10), "rating.k_order_invalid", None),
    ];
    for (body, code, range) in cases {
        let (status, got) = put(&app, SETTINGS, &ana, body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{code}");
        let want = match range {
            Some((min, max)) => json!({ "error": code, "params": { "min": min, "max": max } }),
            None => err(code),
        };
        assert_eq!(got, want);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn changing_settings_recomputes_the_history(db: PgPool) {
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    assert_eq!(table(&ranking(&f.app, &f.bia).await)[0].1, 1569);
    let (status, body) = put(
        &f.app,
        "/tenants/quake-br/ratings/settings",
        &f.ana,
        elo(1200, 20, 40, 5),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // +20 no primeiro mapa (K 40), +16 no segundo (K 36,4 contra 1180).
    assert_eq!(
        table(&ranking(&f.app, &f.bia).await),
        [
            (f.p1.id, 1236, 2, 2, 0, true),
            (f.p2.id, 1164, 2, 0, 2, true)
        ]
    );
}
