//! Quem contesta e até quando: qualquer jogador da série menos quem enviou
//! (confirmar é só do outro time) e, depois de confirmada pelo adversário ou
//! pelo prazo, até a próxima série de um dos lados começar, com a série
//! reaberta e a chave refeita.

use axum::http::StatusCode;
use serde_json::{Value, json};

use super::*;
use crate::common::*;
use crate::teams::{started, tdm};

#[tokio::test]
async fn a_teammate_contests_but_only_the_other_team_confirms() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // A = P1 e P2; B = P3 e P4.
    started(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let k = "wb-1-1";
    let (status, body) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let can = |d: &Value| {
        (
            d["games"][0]["task"].clone(),
            d["games"][0]["can_contest"].clone(),
        )
    };
    assert_eq!(can(&body), (json!("await_opponent"), json!(false)));
    let teammate = detail(&app, &p[1], k).await;
    assert_eq!(can(&teammate), (json!("await_opponent"), json!(true)));
    let opponent = detail(&app, &p[2], k).await;
    assert_eq!(can(&opponent), (json!("respond"), json!(true)));

    let own = (StatusCode::FORBIDDEN, err("match.own_report"));
    assert_eq!(confirm(&app, &p[1], k, 1).await, own);
    assert_eq!(contest(&app, &p[0], k, 1, "errei").await, own);
    let (status, body) = contest(&app, &p[1], k, 1, "Quem venceu foi o B").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let g = &body["games"][0];
    assert_eq!(
        (&g["status"], &g["contested_by"], &g["can_contest"]),
        (&json!("contested"), &json!("P2"), &json!(false))
    );
}

#[tokio::test]
async fn a_confirmed_game_is_contested_until_either_side_starts_another_series() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // wb-1-1: P1 x P4; wb-1-2: P2 x P3.
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let k = "wb-1-1";
    // P1 envia a 1 e P4 não responde: confirma pelo prazo. A 2, P4 confirma.
    let (status, body) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    age(&db, 26).await;
    let (status, body) = report(&app, &p[0], k, 2, game("a", "exile")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[3], k, 2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    play(&app, &p[1], &p[2], "wb-1-2", 2, 0).await;
    let b = bracket(&app, &ana).await;
    assert_eq!(find(&b, "wb-2-1")["state"], "ready");

    let can = |d: &Value| -> Vec<Value> {
        let games = d["games"].as_array().unwrap();
        games.iter().map(|g| g["can_contest"].clone()).collect()
    };
    let d = detail(&app, &p[3], k).await;
    assert_eq!(d["games"][0]["confirmed_via"], "deadline");
    assert_eq!(can(&d), [true, true]);
    assert_eq!(can(&detail(&app, &p[0], k).await), [false, false]);

    // P4 contesta a 2, que decidia a série: ela volta a contestada, a série
    // reabre e a final perde o lado de P1.
    let (status, body) = contest(&app, &p[3], k, 2, "Eu venci a 2").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        (
            &body["summary"]["state"],
            &body["summary"]["result"],
            &body["games"][1]["status"],
            &body["games"][1]["confirmed_via"]
        ),
        (
            &json!("ready"),
            &Value::Null,
            &json!("contested"),
            &Value::Null
        )
    );
    let history = kinds(&body);
    assert_eq!(
        history[history.len() - 3..],
        ["decided", "contested:2", "reopened"]
    );
    let b = bracket(&app, &ana).await;
    assert_eq!(
        (side_player(&b, "wb-2-1", "a"), &find(&b, "wb-2-1")["state"]),
        (Value::Null, &json!("pending"))
    );
    let (status, pending) = get(&app, "/me/organizing", &ana).await;
    assert_eq!(status, StatusCode::OK, "{pending}");
    assert_eq!(
        (
            &pending["contested"][0]["key"],
            &pending["contested"][0]["game"]
        ),
        (&json!(k), &json!(2))
    );

    // A organização aceita a 2 como foi enviada: a série fecha de novo e a
    // chave volta a ter P1 na final. Resolvida por ela, não se contesta mais.
    let (status, body) = confirm(&app, &ana, k, 2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &ana).await;
    assert_eq!(side_player(&b, "wb-2-1", "a"), p[0].id);
    let closed = (StatusCode::CONFLICT, err("match.contest_closed"));
    assert_eq!(contest(&app, &p[3], k, 2, "de novo").await, closed);
    assert_eq!(can(&detail(&app, &p[3], k).await), [true, false]);

    // A final começa: as séries dos dois finalistas não se contestam mais.
    let (status, body) = report(&app, &p[0], "wb-2-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(contest(&app, &p[3], k, 1, "tarde").await, closed);
    assert_eq!(contest(&app, &p[2], "wb-1-2", 1, "tarde").await, closed);
    assert_eq!(can(&detail(&app, &p[3], k).await), [false, false]);
}
