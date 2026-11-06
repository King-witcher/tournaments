//! O rating de cada time congelado no início para o desempate pelo menor
//! rating: gravado ao iniciar (só com ele entre os desempates), apagado ao
//! desfazer, gravado de novo ao reiniciar, e o que a tabela usa mesmo depois
//! de o rating da comunidade mudar.

use serde_json::{Value, json};

use super::*;
use crate::matches::{bracket, copa, play, players, sides, start, table, table_row};
use crate::ratings::RATINGS;
use crate::schema::{CHECK, exec, rejected, rule};

/// `delta` no rating de `who`, pela organização.
async fn adjust(app: &Router, ana: &User, who: &User, delta: i32) {
    let uri = format!("{RATINGS}/{}/adjustments", who.id);
    let (status, body) = post(app, &uri, ana, json!({ "delta": delta, "reason": "Liga" })).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

/// O rating congelado do time de cada um de `who`, na ordem.
async fn frozen(db: &PgPool, who: &[&User]) -> Vec<Option<i32>> {
    let mut out = Vec::new();
    for u in who {
        let r = sqlx::query_scalar!(
            r#"select t.rating_at_start from teams t
               join participants p on p.tournament_id = t.tournament_id and p.team_id = t.id
               where p.user_id = $1"#,
            u.id
        )
        .fetch_one(db)
        .await
        .unwrap();
        out.push(r);
    }
    out
}

fn round_robin(tiebreakers: Value) -> Value {
    copa(table(
        "round_robin",
        json!({ "best_of": 1, "tiebreakers": tiebreakers }),
    ))
}

#[tokio::test]
async fn the_start_freezes_the_rating_and_undoing_clears_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    adjust(&app, &ana, &p[0], 100).await;
    adjust(&app, &ana, &p[1], -50).await;
    let all = [&p[0], &p[1], &p[2]];
    // P3 sem rating: o inicial da comunidade.
    start(&app, &ana, round_robin(json!(["lower_rating"])), &all).await;
    assert_eq!(
        frozen(&db, &all).await,
        [Some(160_000), Some(145_000), Some(150_000)]
    );
    let b = bracket(&app, &ana).await;
    assert_eq!(table_row(&b, &p[1])["rating"], json!(1450.0));
    assert_eq!(
        rejected(exec(&db, "update teams set rating_at_start = 1").await),
        rule(CHECK, "teams_started_frozen")
    );

    // Desfazer apaga; o rating muda e reiniciar grava o de então.
    let (status, body) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(frozen(&db, &all).await, [None, None, None]);
    // Só time da chave tem o rating do início.
    assert_eq!(
        rejected(exec(&db, "update teams set seed = null, rating_at_start = 1").await),
        rule(CHECK, "teams_rating_at_start_seeded")
    );
    adjust(&app, &ana, &p[2], 7).await;
    assert_eq!(ok(&app, &ana, "start").await, "in_progress");
    assert_eq!(
        frozen(&db, &all).await,
        [Some(160_000), Some(145_000), Some(150_700)]
    );
}

#[tokio::test]
async fn without_the_lower_rating_nothing_is_frozen() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    let both = [&p[0], &p[1]];
    start(&app, &ana, round_robin(json!(["head_to_head"])), &both).await;
    assert_eq!(frozen(&db, &both).await, [None, None]);
    let b = bracket(&app, &ana).await;
    assert_eq!(table_row(&b, &p[0])["rating"], Value::Null);
}

#[tokio::test]
async fn the_table_puts_the_lower_frozen_rating_ahead() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    adjust(&app, &ana, &p[0], 100).await;
    adjust(&app, &ana, &p[1], -50).await;
    start(
        &app,
        &ana,
        round_robin(json!(["lower_rating"])),
        &[&p[0], &p[1], &p[2]],
    )
    .await;
    // Ciclo P1 > P2 > P3 > P1: os três com saldo 0. Os mapas mudam o rating
    // da comunidade, e a tabela segue com o do início: P2 (1450), P3 (1500),
    // P1 (1600).
    let b = bracket(&app, &ana).await;
    for (winner, loser) in [(0, 1), (1, 2), (2, 0)] {
        let (w, l) = (json!(p[winner].id), json!(p[loser].id));
        let m = b["matches"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| {
                let s = sides(&b, m);
                s == (w.clone(), l.clone()) || s == (l.clone(), w.clone())
            })
            .unwrap();
        let key = m["key"].as_str().unwrap();
        let (a, _) = sides(&b, m);
        let (maps_a, maps_b) = if a == w { (1, 0) } else { (0, 1) };
        play(&app, &p[winner], &p[loser], key, maps_a, maps_b).await;
    }
    let b = bracket(&app, &ana).await;
    assert_eq!(b["status"], "finished");
    let order: Vec<Value> = (0..3)
        .map(|i| crate::matches::player(&b, &b["table"][i]["team_id"]))
        .collect();
    assert_eq!(order, [json!(p[1].id), json!(p[2].id), json!(p[0].id)]);
    let ahead: Vec<&Value> = (0..2).map(|i| &b["table"][i]["ahead_by"]).collect();
    assert_eq!(ahead, [&json!("lower_rating"), &json!("lower_rating")]);
    assert_eq!(b["table"][2]["rating"], json!(1600.0));
}
