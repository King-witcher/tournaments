//! Veto: o histórico só cresce, com a forma de cada tipo e os mesmos lados;
//! pronto antes do começo e o resto depois dele, um começo por partida; o
//! pool de mapas com mapas para a série mais longa e, como a configuração do
//! veto, congelado depois do início; tudo sai com o torneio.

use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{copa, single, start};

/// Final única em md3 com veto, iniciada: `(torneio, tenant, p1, p2)` com os
/// ids dos participantes.
async fn started(db: &PgPool) -> (i64, i64, i64, i64) {
    let (app, ana, _) = quake_br(db).await;
    let p1 = member(&app, db, &ana, "P1").await;
    let p2 = member(&app, db, &ana, "P2").await;
    let body = copa(with(
        single(3, 3),
        json!({ "veto": { "map_mode": "veto", "champion_phase": true } }),
    ));
    start(&app, &ana, body, &[&p1, &p2]).await;
    let row: (i64, i64) =
        sqlx::query_as("select id, tenant_id from tournaments where slug = 'copa'")
            .fetch_one(db)
            .await
            .unwrap();
    let ids: Vec<i64> =
        sqlx::query_scalar("select id from teams where tournament_id = $1 order by seed")
            .bind(row.0)
            .fetch_all(db)
            .await
            .unwrap();
    (row.0, row.1, ids[0], ids[1])
}

/// Um evento do veto de `wb-1-1` entre `a` e `b`, com `columns`/`values` a
/// mais.
fn event(t: (i64, i64, i64, i64), kind: &str, columns: &str, values: &str) -> String {
    let (tournament, tenant, a, b) = t;
    format!(
        "insert into match_veto_events
             (tenant_id, tournament_id, stage, match_key, side_a, side_b, kind{columns})
         values ({tenant}, {tournament}, 1, 'wb-1-1', {a}, {b}, '{kind}'{values})"
    )
}

#[tokio::test]
async fn veto_events_keep_their_shape_and_order() {
    let (db, _lease) = crate::db::test_db().await;
    let t = started(&db).await;
    let awoken = "(select id from maps where slug = 'awoken')";
    let ban = event(
        t,
        "ban",
        ", game, side, map_id",
        &format!(", 1, 'a', {awoken}"),
    );
    // Nada antes do começo, só o pronto.
    assert_eq!(
        rejected(exec(&db, &ban).await),
        rule(CHECK, "match_veto_events_order")
    );
    exec(&db, &event(t, "ready", ", side", ", 'a'"))
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, &event(t, "ready", ", side", ", 'a'")).await),
        rule(UNIQUE, "match_veto_events_ready_once")
    );
    // O começo tem a seed e é o jogo 1.
    assert_eq!(
        rejected(exec(&db, &event(t, "begin", ", game, side", ", 1, 'a'")).await),
        rule(CHECK, "match_veto_events_seed")
    );
    assert_eq!(
        rejected(exec(&db, &event(t, "begin", ", game, side, seed", ", 2, 'a', 7")).await),
        rule(CHECK, "match_veto_events_shape")
    );
    exec(
        &db,
        &event(t, "begin", ", game, side, seed", ", 1, 'a', -7"),
    )
    .await
    .unwrap();
    assert_eq!(
        rejected(exec(&db, &event(t, "begin", ", game, side, seed", ", 1, 'a', 8")).await),
        rule(UNIQUE, "match_veto_events_begin_once")
    );
    assert_eq!(
        rejected(exec(&db, &event(t, "ready", ", side", ", 'b'")).await),
        rule(CHECK, "match_veto_events_order")
    );
    exec(&db, &ban).await.unwrap();
    // Mapa e campeão juntos, mapa fora do pool, passo sorteado com autor.
    let both = event(
        t,
        "ban",
        ", game, side, map_id, champion_id",
        &format!(", 1, 'b', {awoken}, (select min(id) from champions)"),
    );
    assert_eq!(
        rejected(exec(&db, &both).await),
        rule(CHECK, "match_veto_events_shape")
    );
    let outside = event(
        t,
        "ban",
        ", game, side, map_id",
        ", 1, 'b', (select id from maps where slug = 'tower-of-koth')",
    );
    assert_eq!(rejected(exec(&db, &outside).await).0, FOREIGN_KEY);
    let timeout_with_actor = event(
        t,
        "ban",
        ", game, side, map_id, timeout, actor",
        ", 1, 'b', (select id from maps where slug = 'exile'), true, (select min(id) from users)",
    );
    assert_eq!(
        rejected(exec(&db, &timeout_with_actor).await),
        rule(CHECK, "match_veto_events_timeout")
    );
    // Os mesmos lados em toda a partida.
    let (tournament, tenant, a, b) = t;
    let swapped = event((tournament, tenant, b, a), "undo", ", game", ", 1");
    assert_eq!(
        rejected(exec(&db, &swapped).await),
        rule(CHECK, "match_veto_events_same_sides")
    );
    // O histórico não muda; apagar a partida inteira, sim.
    assert_eq!(
        rejected(exec(&db, "update match_veto_events set timeout = false").await),
        rule(CHECK, "match_veto_events_immutable")
    );
    exec(&db, "delete from match_veto_events").await.unwrap();
}

#[tokio::test]
async fn the_pool_fits_the_series_and_freezes_with_the_start() {
    let (db, _lease) = crate::db::test_db().await;
    let (tournament, _, _, _) = started(&db).await;
    let remove_one =
        format!("delete from tournament_maps where tournament_id = {tournament} and position = 7");
    assert_eq!(
        rejected(exec(&db, &remove_one).await),
        rule(CHECK, "tournament_maps_started_frozen")
    );
    let veto_off = format!("update tournaments set map_mode = 'off' where id = {tournament}");
    assert_eq!(
        rejected(exec(&db, &veto_off).await),
        rule(CHECK, "tournaments_veto_started_frozen")
    );

    // Num torneio que não começou: md3 precisa de 3 mapas no pool, fora do
    // modo desligado (conferido no commit).
    let (app, ana) = {
        let ana = user(&db, "Outra").await;
        let app = app(db.clone());
        create(&app, &ana, "outra").await;
        (app, ana)
    };
    let (status, created) = post(
        &app,
        "/tenants/outra/tournaments",
        &ana,
        with(duel("liga"), json!({ "veto": { "map_mode": "veto" } })),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED, "{created}");
    let liga = "(select id from tournaments where slug = 'liga')";
    let shrink =
        format!("delete from tournament_maps where tournament_id = {liga} and position > 4");
    assert_eq!(
        rejected(exec(&db, &shrink).await),
        rule(CHECK, "tournament_maps_pool_size")
    );
    exec(
        &db,
        &format!("update tournaments set map_mode = 'off' where id = {liga}"),
    )
    .await
    .unwrap();
    exec(&db, &shrink).await.unwrap();
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!("update tournaments set map_mode = 'vote' where id = {liga}")
            )
            .await
        ),
        rule(CHECK, "tournament_maps_pool_size")
    );
}

#[tokio::test]
async fn the_veto_goes_with_the_tournament() {
    let (db, _lease) = crate::db::test_db().await;
    let t = started(&db).await;
    exec(&db, &event(t, "ready", ", side", ", 'a'"))
        .await
        .unwrap();
    exec(&db, &format!("delete from tournaments where id = {}", t.0))
        .await
        .unwrap();
    assert_eq!(
        count(&db, "select count(*) from match_veto_events").await,
        0
    );
    assert_eq!(count(&db, "select count(*) from tournament_maps").await, 0);
}
