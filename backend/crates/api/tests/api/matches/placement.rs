//! Disputa de posições: os pares pela ordem da chave, todos prontos (e
//! avisados) no início, quem vence fica com a melhor posição, o último seed
//! sem par com N ímpar, a desistência que vira W.O. e a série par recusada.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};

use super::*;
use crate::common::*;
use crate::realtime::{notices, outbox};

async fn mine(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, "/me/tournaments", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["registered"][0].clone()
}

/// Os `match.ready` do outbox: `(destinatário, partida)`, em ordem.
async fn ready_notices(db: &sqlx::PgPool) -> Vec<(i64, String)> {
    let mut all: Vec<(i64, String)> = notices(&outbox(db).await)
        .into_iter()
        .filter(|(code, ..)| code == "match.ready")
        .map(|(_, to, params)| (to[0], params["match"].as_str().unwrap().to_owned()))
        .collect();
    all.sort();
    all
}

#[tokio::test]
async fn four_players_pair_by_seed_and_the_winner_takes_the_better_place() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    start(
        &app,
        &ana,
        copa(table("placement", json!({ "best_of": 3 }))),
        &all,
    )
    .await;

    // 1 x 2 e 3 x 4, numa rodada só, os dois prontos; o lado a é o seed melhor.
    let b = bracket(&app, &p[0]).await;
    assert_eq!(
        b["format"],
        json!({ "format": "placement", "best_of": 3, "advance": null, "check_in": false })
    );
    assert_eq!(
        (&b["table"], &b["total_rounds"]),
        (&Value::Null, &Value::Null)
    );
    assert_eq!(keys_in(&b, "ready"), ["pl-1-1", "pl-1-2"]);
    assert_eq!(find(&b, "pl-1-1")["part"], "placement");
    let id = |i: usize| json!(p[i].id);
    assert_eq!(sides(&b, find(&b, "pl-1-1")), (id(0), id(1)));
    assert_eq!(sides(&b, find(&b, "pl-1-2")), (id(2), id(3)));
    assert_eq!(standings(&b), json!([]));
    // Os dois lados de cada série recebem `match.ready` no início.
    let mut expected: Vec<(i64, String)> = [(0, 1), (1, 1), (2, 2), (3, 2)]
        .iter()
        .map(|&(i, pair)| (p[i].id, format!("pl-1-{pair}")))
        .collect();
    expected.sort();
    assert_eq!(ready_notices(&db).await, expected);

    // O 4º vence o 3º e fica em 3º, enquanto a outra série segue.
    play(&app, &p[3], &p[2], "pl-1-2", 0, 2).await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "in_progress");
    assert_eq!(
        standings(&b),
        json!([
            { "user_id": p[3].id, "rank": 3, "tied": 1 },
            { "user_id": p[2].id, "rank": 4, "tied": 1 },
        ])
    );
    let third = mine(&app, &p[3]).await;
    assert_eq!(
        (&third["situation"], &third["placement"]),
        (&json!("eliminated"), &json!({ "rank": 3, "tied": 1 }))
    );

    favorite_wins(&app, &p, "pl-1-1", 2).await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(
        standings(&b),
        json!([
            { "user_id": p[0].id, "rank": 1, "tied": 1 },
            { "user_id": p[1].id, "rank": 2, "tied": 1 },
            { "user_id": p[3].id, "rank": 3, "tied": 1 },
            { "user_id": p[2].id, "rank": 4, "tied": 1 },
        ])
    );
    assert_eq!(mine(&app, &p[0]).await["situation"], "champion");
}

#[tokio::test]
async fn five_players_leave_the_last_in_place_and_a_withdrawal_is_a_walkover() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 5).await;
    let all: Vec<&User> = p.iter().collect();
    start(
        &app,
        &ana,
        copa(table("placement", json!({ "best_of": 1 }))),
        &all,
    )
    .await;

    // O 5º não tem par: fica em 5º desde o início, sem jogar nem ser avisado.
    let b = bracket(&app, &p[0]).await;
    assert_eq!(keys_in(&b, "ready"), ["pl-1-1", "pl-1-2"]);
    let alone = find(&b, "pl-1-3");
    assert_eq!(
        (&alone["state"], &alone["b"]["bye"]),
        (&json!("bye"), &json!(true))
    );
    assert_eq!(side_player(&b, "pl-1-3", "a"), p[4].id);
    assert_eq!(
        standings(&b),
        json!([{ "user_id": p[4].id, "rank": 5, "tied": 1 }])
    );
    let told = ready_notices(&db).await;
    assert_eq!(told.len(), 4);
    assert!(told.iter().all(|(to, _)| *to != p[4].id), "{told:?}");

    // O 2º desiste: perde a série dele por W.O. e fica em 2º.
    let team = team_ids(&app, &ana, "copa", &[&p[1]]).await[0];
    let uri = format!("{COPA}/teams/{team}/withdrawal");
    let (status, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let b = bracket(&app, &p[0]).await;
    let first = find(&b, "pl-1-1");
    assert_eq!(
        (&first["state"], &first["result"]["confirmed_via"]),
        (&json!("completed"), &json!("withdrawal"))
    );

    favorite_wins(&app, &p, "pl-1-2", 1).await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    let ranks: Vec<Value> = (0..5)
        .map(|i| json!({ "user_id": p[i].id, "rank": i + 1, "tied": 1 }))
        .collect();
    assert_eq!(standings(&b), json!(ranks));
    let gone = mine(&app, &p[1]).await;
    assert_eq!(
        (&gone["situation"], &gone["placement"]),
        (&json!("withdrawn"), &json!({ "rank": 2, "tied": 1 }))
    );
}

#[tokio::test]
async fn placement_takes_only_odd_series_and_up_to_128() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let create = |body: Value| post(&app, "/tenants/quake-br/tournaments", &ana, body);
    let placement = |fields: Value| with(duel("copa"), table("placement", fields));
    assert_eq!(
        create(placement(json!({ "best_of": 2 }))).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "format.invalid_best_of", "params": { "best_of": 2, "stage": 1 } })
        )
    );
    let crowded = with(
        placement(json!({ "best_of": 3 })),
        json!({ "max_participants": 129 }),
    );
    assert_eq!(
        create(crowded).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "error": "format.too_many_participants",
                "params": { "max": 128, "got": 129, "stage": 1 }
            })
        )
    );
    let created = create_tournament(&app, &ana, placement(json!({ "best_of": 5 }))).await;
    assert_eq!(
        created["stages"][0],
        json!({
            "format": "placement", "best_of": 5, "advance": null, "check_in": false,
            "number": 1, "status": "pending", "started_at": null, "check_in_opened_at": null
        })
    );
    // A edição troca para outro formato e volta, com a mesma regra.
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        table("placement", json!({ "best_of": 4 })),
    )
    .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "format.invalid_best_of", "params": { "best_of": 4, "stage": 1 } })
        )
    );
    let (status, body) = patch(&app, COPA, &ana, single(3, 5)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        table("placement", json!({ "best_of": 7 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["stages"][0],
        json!({
            "format": "placement", "best_of": 7, "advance": null, "check_in": false,
            "number": 1, "status": "pending", "started_at": null, "check_in_opened_at": null
        })
    );
}
