//! Rating: a forma da configuração, dos ajustes e do cache; as marcas de
//! "desatualizado" que os triggers gravam; e o que sai com o tenant.

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{confirm, game, report};
use crate::ratings::{RATINGS, final_md3};

async fn dirty(db: &PgPool) -> i64 {
    count(db, "select count(*) from rating_dirty").await
}

async fn clean(db: &PgPool) {
    exec(db, "delete from rating_dirty").await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn shape_of_settings_adjustments_and_cache(db: PgPool) {
    let cup = finished_cup(&db).await;
    let (t, p1, p2) = (cup.tenant, cup.p1.id, cup.p2.id);
    let adjustment = |modality: &str, user: i64, delta: i32, reason: &str| {
        format!(
            "insert into rating_adjustments (tenant_id, modality, user_id, delta, reason)
             values ({t}, '{modality}', {user}, {delta}, '{reason}')"
        )
    };
    for (sql, name) in [
        (
            adjustment("duel", p1, 0, "x"),
            "rating_adjustments_delta_check",
        ),
        (
            adjustment("duel", p1, 1001, "x"),
            "rating_adjustments_delta_check",
        ),
        (
            adjustment("duel", p1, 5, ""),
            "rating_adjustments_reason_check",
        ),
        (
            adjustment("ffa", p1, 5, "x"),
            "rating_adjustments_modality_check",
        ),
        (
            format!(
                "insert into rating_settings (tenant_id, initial, k_min, k_max, k_decay)
                 values ({t}, 1500, 50, 40, 10)"
            ),
            "rating_settings_k_order",
        ),
        (
            format!(
                "insert into rating_settings (tenant_id, initial, k_min, k_max, k_decay)
                 values ({t}, 99, 32, 80, 10)"
            ),
            "rating_settings_initial_check",
        ),
        (
            format!(
                "insert into ratings
                     (tenant_id, modality, user_id, rating, games, wins, losses, last_game_at)
                 values ({t}, 'duel', {p1}, 1500, 2, 1, 0, now())"
            ),
            "ratings_record",
        ),
        (
            format!(
                "insert into ratings
                     (tenant_id, modality, user_id, rating, games, wins, losses, last_game_at)
                 values ({t}, 'duel', {p1}, 1500, 0, 0, 0, now())"
            ),
            "ratings_last_game",
        ),
        (
            format!(
                "insert into rating_history
                     (tenant_id, modality, user_id, seq, delta, rating_after, at)
                 values ({t}, 'duel', {p1}, 1, 5, 1505, now())"
            ),
            "rating_history_event",
        ),
        (
            format!(
                "insert into rating_history (tenant_id, modality, user_id, seq, game_id,
                                             opponent_id, won, delta, rating_after, at)
                 values ({t}, 'duel', {p1}, 1, 1, {p1}, true, 5, 1505, now())"
            ),
            "rating_history_opponent",
        ),
        (
            format!(
                "insert into rating_history (tenant_id, modality, user_id, seq, game_id,
                                             delta, rating_after, at)
                 values ({t}, 'duel', {p1}, 1, 1, 5, 1505, now())"
            ),
            "rating_history_game",
        ),
    ] {
        assert_eq!(rejected(exec(&db, &sql).await), rule(CHECK, name), "{sql}");
    }
    // Ajuste e cache só de membro do tenant.
    let outsider = user(&db, "Zé de fora").await;
    assert_eq!(
        rejected(exec(&db, &adjustment("duel", outsider.id, 5, "x")).await),
        rule(FOREIGN_KEY, "rating_adjustments_tenant_id_user_id_fkey")
    );
    exec(&db, &adjustment("duel", p2, -1000, "ok"))
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_what_changes_the_rating_marks_the_tenant(db: PgPool) {
    let f = final_md3(&db, json!({})).await;
    // Mapa enviado (pendente): ainda não conta.
    let (status, body) = report(&f.app, &f.p1, "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(dirty(&db).await, 0);
    let (status, body) = confirm(&f.app, &f.p2, "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(dirty(&db).await, 1);

    // Corrigir um confirmado marca; o "conta para o rating", só se muda.
    clean(&db).await;
    exec(&db, "update match_games set a_frags = 20, b_frags = 3")
        .await
        .unwrap();
    assert_eq!(dirty(&db).await, 1);
    clean(&db).await;
    exec(&db, "update tournaments set counts_for_rating = true")
        .await
        .unwrap();
    assert_eq!(dirty(&db).await, 0);
    exec(&db, "update tournaments set counts_for_rating = false")
        .await
        .unwrap();
    assert_eq!(dirty(&db).await, 1);

    // A leitura recalcula e consome as marcas.
    let (status, body) = get(&f.app, RATINGS, &f.bia).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["players"], json!([]));
    assert_eq!(dirty(&db).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_tenant_takes_everything_with_it(db: PgPool) {
    let cup = finished_cup(&db).await;
    let (status, body) = post(
        &cup.app,
        &format!("{RATINGS}/{}/adjustments", cup.p2.id),
        &cup.olga,
        json!({ "delta": 25, "reason": "volta" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (status, body) = put(
        &cup.app,
        "/tenants/quake-br/ratings/settings",
        &cup.ana,
        json!({ "initial": 1400, "k_min": 32, "k_max": 80, "k_decay": 10 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let tables = [
        "rating_settings",
        "rating_adjustments",
        "ratings",
        "rating_history",
        "rating_dirty",
    ];
    let rows = |table: &str| format!("select count(*) from {table}");
    assert!(count(&db, &rows("ratings")).await > 0);
    assert!(count(&db, &rows("rating_history")).await > 0);

    // Excluir o torneio com mapas marca o tenant (o rating muda).
    clean(&db).await;
    exec(
        &db,
        &format!("delete from tournaments where id = {}", cup.tournament),
    )
    .await
    .unwrap();
    assert!(dirty(&db).await > 0);

    exec(&db, "delete from tenants where slug = 'quake-br'")
        .await
        .unwrap();
    for table in tables {
        assert_eq!(count(&db, &rows(table)).await, 0, "{table}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_the_tenant_with_maps_and_marks(db: PgPool) {
    // A cascata apaga os mapas confirmados depois do tenant: o trigger não
    // tenta marcar um tenant que não existe mais.
    finished_cup(&db).await;
    assert!(dirty(&db).await > 0);
    exec(&db, "delete from tenants where slug = 'quake-br'")
        .await
        .unwrap();
    assert_eq!(dirty(&db).await, 0);
}
