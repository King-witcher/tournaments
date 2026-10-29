//! Suíço: o início grava a primeira rodada, a última série de uma rodada
//! gera a seguinte, a correção não refaz pareamento, e quem desiste mantém os
//! pontos e não é mais pareado.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::realtime::{clear, notices, outbox};

/// Os pares gravados de cada rodada: `[(rodada, a, b)]`, `b` nulo no bye.
async fn pairings(db: &PgPool) -> Vec<(i16, i64, Option<i64>)> {
    sqlx::query_as("select round, side_a, side_b from stage_pairings order by round, number")
        .fetch_all(db)
        .await
        .unwrap()
}

/// Os lados (`user_id`) das partidas da rodada `round` da chave `b`.
fn round_sides(b: &Value, round: u16) -> Vec<(Value, Value)> {
    b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["round"] == round && m["state"] != "bye")
        .map(|m| sides(b, m))
        .collect()
}

/// Joga, com vitória do seed melhor, as partidas prontas da rodada `round`.
async fn play_round(app: &axum::Router, p: &[User], round: u16) {
    let b = bracket(app, &p[0]).await;
    for key in keys_in(&b, "ready") {
        if key.starts_with(&format!("sw-{round}-")) {
            favorite_wins(app, p, &key, 1).await;
        }
    }
}

#[tokio::test]
async fn each_round_is_paired_when_the_previous_one_ends() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 6).await;
    let all: Vec<&User> = p.iter().collect();
    let body = copa(table("swiss", json!({ "best_of": 1, "rounds": null })));
    start(&app, &ana, body, &all).await;

    // ⌈log2 6⌉ = 3 rodadas; só a primeira existe: 1 x 4, 2 x 5, 3 x 6.
    let b = bracket(&app, &p[0]).await;
    assert_eq!(
        b["format"],
        json!({ "format": "swiss", "best_of": 1, "rounds": null })
    );
    assert_eq!(b["total_rounds"], 3);
    assert_eq!(keys_in(&b, "ready"), ["sw-1-1", "sw-1-2", "sw-1-3"]);
    assert_eq!(find(&b, "sw-1-1")["part"], "swiss");
    let id = |i: usize| json!(p[i].id);
    assert_eq!(
        round_sides(&b, 1),
        [(id(0), id(3)), (id(1), id(4)), (id(2), id(5))]
    );
    assert_eq!(pairings(&db).await.len(), 3);
    assert_eq!(b["table"][0]["buchholz"], 0);
    let ready = notices(&outbox(&db).await);
    assert_eq!(ready.iter().filter(|n| n.0 == "match.ready").count(), 6);

    // Duas séries decididas: a rodada ainda não acabou.
    favorite_wins(&app, &p, "sw-1-1", 1).await;
    favorite_wins(&app, &p, "sw-1-2", 1).await;
    assert_eq!(
        bracket(&app, &p[0]).await["matches"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    // A última série da rodada gera a seguinte, sem revanche, e avisa todos.
    clear(&db).await;
    favorite_wins(&app, &p, "sw-1-3", 1).await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(keys_in(&b, "ready"), ["sw-2-1", "sw-2-2", "sw-2-3"]);
    let first = round_sides(&b, 1);
    let second = round_sides(&b, 2);
    let mut seen: Vec<Value> = second
        .iter()
        .flat_map(|(a, b)| [a.clone(), b.clone()])
        .collect();
    seen.sort_by_key(|v| v.as_i64());
    let mut everyone: Vec<Value> = (0..6).map(id).collect();
    everyone.sort_by_key(|v| v.as_i64());
    assert_eq!(seen, everyone);
    for (a, b) in &second {
        assert!(
            !first.contains(&(a.clone(), b.clone())) && !first.contains(&(b.clone(), a.clone())),
            "revanche {a} x {b}"
        );
    }
    let ready: Vec<(String, Vec<i64>, Value)> = notices(&outbox(&db).await)
        .into_iter()
        .filter(|n| n.0 == "match.ready")
        .collect();
    assert_eq!(ready.len(), 6);
    assert!(
        ready
            .iter()
            .all(|n| n.2["match"].as_str().unwrap().starts_with("sw-2-"))
    );
    // Os vencedores da rodada 1 têm 1 ponto; quem perdeu para eles soma o
    // ponto deles no Buchholz.
    assert_eq!(table_row(&b, &p[3])["buchholz"], 1);

    // A correção de uma série da rodada 1 muda a tabela, não o pareamento.
    let (status, body) = set_game(&app, &ana, "sw-1-1", 1, game("b", MAPS[0]), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &p[0]).await;
    assert_eq!(round_sides(&b, 2), second);
    assert_eq!(pairings(&db).await.len(), 6);
    assert_eq!(table_row(&b, &p[3])["points"], 1);
    assert_eq!(table_row(&b, &p[0])["points"], 0);
}

#[tokio::test]
async fn a_withdrawn_team_keeps_its_points_and_is_not_paired_again() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 6).await;
    let all: Vec<&User> = p.iter().collect();
    let body = copa(table("swiss", json!({ "best_of": 1, "rounds": 3 })));
    start(&app, &ana, body, &all).await;
    play_round(&app, &p, 1).await;

    // P1 (venceu a rodada 1) desiste: a partida dele na rodada 2 é W.O.
    let team = team_ids(&app, &ana, "copa", &[&p[0]]).await[0];
    let uri = format!("{COPA}/teams/{team}/withdrawal");
    let (status, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let b = bracket(&app, &p[1]).await;
    let theirs = b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| {
            let (a, other) = sides(&b, m);
            m["round"] == 2 && (a == p[0].id || other == p[0].id)
        })
        .unwrap();
    assert_eq!(theirs["result"]["confirmed_via"], "withdrawal");

    // A rodada 3 sai sem ele: 5 ativos, dois pares e um bye.
    play_round(&app, &p, 2).await;
    let b = bracket(&app, &p[1]).await;
    let third: Vec<(i16, i64, Option<i64>)> = pairings(&db)
        .await
        .into_iter()
        .filter(|r| r.0 == 3)
        .collect();
    assert_eq!(third.len(), 3);
    assert!(
        third.iter().all(|r| r.1 != team && r.2 != Some(team)),
        "{third:?}"
    );
    assert_eq!(third[2].2, None);
    let bye = b["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == "sw-3-3")
        .unwrap();
    assert_eq!(
        (&bye["state"], &bye["b"]["bye"]),
        (&json!("bye"), &json!(true))
    );
    // Os pontos ficam: venceu a rodada 1, perdeu a 2 por W.O.
    let row = table_row(&b, &p[0]);
    assert_eq!(
        (&row["points"], &row["wins"], &row["losses"]),
        (&json!(1), &json!(1), &json!(1))
    );

    play_round(&app, &p, 3).await;
    let b = bracket(&app, &p[1]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(b["standings"].as_array().unwrap().len(), 6);
    // Quem teve o bye ganhou o ponto dele.
    let bye_team = third[2].1;
    let bye_row = b["table"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["team_id"] == bye_team)
        .unwrap();
    assert_eq!(bye_row["byes"], 1);
}

#[tokio::test]
async fn table_formats_are_checked_on_create_and_edit() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let invalid = |code: &str, params: Value| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": code, "params": params }),
        )
    };
    let create = |body: Value| post(&app, "/tenants/quake-br/tournaments", &ana, body);
    // O round robin vai até 32 participantes (as vagas contam).
    let rr = with(
        duel("copa"),
        with(
            table("round_robin", json!({ "best_of": 3 })),
            json!({ "max_participants": 33 }),
        ),
    );
    assert_eq!(
        create(rr).await,
        invalid(
            "format.too_many_participants",
            json!({ "max": 32, "got": 33 })
        )
    );
    let swiss = |rounds: Value| {
        with(
            duel("copa"),
            table("swiss", json!({ "best_of": 3, "rounds": rounds })),
        )
    };
    assert_eq!(
        create(swiss(json!(0))).await,
        invalid("format.invalid_rounds", json!({ "max": 127, "got": 0 }))
    );
    assert_eq!(
        create(swiss(json!(128))).await,
        invalid("format.invalid_rounds", json!({ "max": 127, "got": 128 }))
    );
    // Rodadas acima de N − 1 passam: valem N − 1 na fase.
    let created = create_tournament(&app, &ana, swiss(json!(20))).await;
    assert_eq!(created["stages"][0]["rounds"], 20);

    // A edição troca o formato, com as mesmas regras.
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        json!({ "stages": [{ "format": "round_robin", "best_of": 4 }] }),
    )
    .await;
    assert_eq!(
        (status, body),
        invalid("format.invalid_best_of", json!({ "best_of": 4 }))
    );
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        table("round_robin", json!({ "best_of": 5 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body["stages"][0],
        json!({ "format": "round_robin", "best_of": 5 })
    );
}
