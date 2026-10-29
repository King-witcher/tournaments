//! Round robin: todas as partidas prontas no início, a tabela com os
//! desempates, o aviso da próxima partida de cada time, o fim com a
//! classificação e o pódio, e a desistência que vira W.O. em tudo que faltava.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};

use super::*;
use crate::common::*;
use crate::realtime::{clear, notices, outbox};

/// Os `match.ready` do outbox: `(destinatário, partida)`, em ordem.
async fn ready_notices(db: &sqlx::PgPool) -> Vec<(i64, String)> {
    notices(&outbox(db).await)
        .into_iter()
        .filter(|(code, ..)| code == "match.ready")
        .map(|(_, to, params)| (to[0], params["match"].as_str().unwrap().to_owned()))
        .collect()
}

async fn mine(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, "/me/tournaments", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body[0].clone()
}

#[tokio::test]
async fn a_round_robin_plays_every_pair_and_ends_with_the_table() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = copa(table("round_robin", json!({ "best_of": 1 })));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2], &p[3]]).await;

    let b = bracket(&app, &p[0]).await;
    assert_eq!(
        b["format"],
        json!({ "format": "round_robin", "best_of": 1 })
    );
    assert_eq!(b["total_rounds"], 3);
    // Seis partidas, todas prontas, rodada 1 com 1 x 4 e 2 x 3.
    let ready = keys_in(&b, "ready");
    assert_eq!(ready.len(), 6);
    assert!(ready.iter().all(|k| k.starts_with("rr-")), "{ready:?}");
    assert_eq!(find(&b, "rr-1-1")["part"], "round_robin");
    assert_eq!(
        sides(&b, find(&b, "rr-1-1")),
        (json!(p[0].id), json!(p[3].id))
    );
    assert_eq!(
        sides(&b, find(&b, "rr-1-2")),
        (json!(p[1].id), json!(p[2].id))
    );
    assert_eq!(b["standings"], json!([]));
    assert_eq!(table_points(&b).iter().map(|r| r.1).sum::<i64>(), 0);
    assert_eq!(b["table"][0]["buchholz"], Value::Null);

    // Cada time recebe só a primeira partida dele, não as seis.
    assert_eq!(
        ready_notices(&db).await,
        [
            (p[0].id, "rr-1-1".to_owned()),
            (p[3].id, "rr-1-1".to_owned()),
            (p[1].id, "rr-1-2".to_owned()),
            (p[2].id, "rr-1-2".to_owned()),
        ]
    );
    // "Seus torneios": jogando, com a primeira partida.
    let first = mine(&app, &p[3]).await;
    assert_eq!(first["situation"], "playing");
    assert_eq!(first["next_match"]["key"], "rr-1-1");

    // Terminou uma série: os dois recebem a próxima deles (na rodada 2).
    clear(&db).await;
    favorite_wins(&app, &p, "rr-1-1", 1).await;
    let mut next = ready_notices(&db).await;
    next.sort();
    let to: Vec<i64> = next.iter().map(|n| n.0).collect();
    assert_eq!(to, [p[0].id, p[3].id]);
    assert!(next.iter().all(|n| n.1.starts_with("rr-2-")), "{next:?}");
    assert_eq!(mine(&app, &p[3]).await["next_match"]["key"], next[1].1);

    for key in keys_in(&bracket(&app, &p[0]).await, "ready") {
        favorite_wins(&app, &p, &key, 1).await;
    }
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(
        table_points(&b),
        [
            (json!(p[0].id), 3),
            (json!(p[1].id), 2),
            (json!(p[2].id), 1),
            (json!(p[3].id), 0)
        ]
    );
    let ahead: Vec<&Value> = (0..4).map(|i| &b["table"][i]["ahead_by"]).collect();
    assert_eq!(
        ahead,
        [
            &json!("points"),
            &json!("points"),
            &json!("points"),
            &Value::Null
        ]
    );
    let first = table_row(&b, &p[0]);
    assert_eq!(
        (&first["wins"], &first["losses"], &first["map_diff"]),
        (&json!(3), &json!(0), &json!(3))
    );
    assert_eq!(
        standings(&b),
        json!([
            { "user_id": p[0].id, "rank": 1, "tied": 1 },
            { "user_id": p[1].id, "rank": 2, "tied": 1 },
            { "user_id": p[2].id, "rank": 3, "tied": 1 },
            { "user_id": p[3].id, "rank": 4, "tied": 1 },
        ])
    );
    let second = mine(&app, &p[1]).await;
    assert_eq!(
        (&second["situation"], &second["placement"]),
        (&json!("eliminated"), &json!({ "rank": 2, "tied": 1 }))
    );
    assert_eq!(mine(&app, &p[0]).await["situation"], "champion");
    // O pódio do aviso (Discord) sai da tabela.
    let events = outbox(&db).await;
    let finished = events
        .iter()
        .find(|e| e["announce"]["code"] == "tournament.finished")
        .unwrap();
    assert_eq!(
        finished["announce"]["params"]["podium"],
        json!([
            { "rank": 1, "player": "P1" },
            { "rank": 2, "player": "P2" },
            { "rank": 3, "player": "P3" },
        ])
    );
}

#[tokio::test]
async fn a_withdrawal_is_a_walkover_in_everything_left() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = copa(table("round_robin", json!({ "best_of": 3 })));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2], &p[3]]).await;
    favorite_wins(&app, &p, "rr-1-2", 2).await;

    let team = team_ids(&app, &ana, "copa", &[&p[3]]).await[0];
    let uri = format!("{COPA}/teams/{team}/withdrawal");
    let (status, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    // As três partidas de P4 viram W.O., confirmadas pela desistência.
    let b = bracket(&app, &p[0]).await;
    let theirs: Vec<&Value> = b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| {
            [
                player(&b, &m["a"]["team_id"]),
                player(&b, &m["b"]["team_id"]),
            ]
            .contains(&json!(p[3].id))
        })
        .collect();
    assert_eq!(theirs.len(), 3);
    for m in theirs {
        assert_eq!(
            (
                &m["state"],
                &m["result"]["walkover"],
                &m["result"]["confirmed_via"]
            ),
            (&json!("completed"), &json!(true), &json!("withdrawal")),
            "{m}"
        );
    }
    // O W.O. vale vitória sem perder mapa (2 x 0 na md3), sem frags.
    let row = table_row(&b, &p[0]);
    assert_eq!(
        (
            &row["points"],
            &row["maps_won"],
            &row["maps_lost"],
            &row["frags_for"]
        ),
        (&json!(1), &json!(2), &json!(0), &json!(0))
    );
    let out = table_row(&b, &p[3]);
    assert_eq!(
        (
            &out["points"],
            &out["played"],
            &out["losses"],
            &out["position"]
        ),
        (&json!(0), &json!(3), &json!(3), &json!(4))
    );

    for key in keys_in(&b, "ready") {
        favorite_wins(&app, &p, &key, 2).await;
    }
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(
        standings(&b)[3],
        json!({ "user_id": p[3].id, "rank": 4, "tied": 1 })
    );
    let gone = mine(&app, &p[3]).await;
    assert_eq!(
        (&gone["situation"], &gone["placement"]),
        (&json!("withdrawn"), &json!({ "rank": 4, "tied": 1 }))
    );
}
