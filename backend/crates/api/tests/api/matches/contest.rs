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
async fn accepting_a_late_contest_decides_the_series_again() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // wb-1-1: P1 x P4.
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let k = "wb-1-1";
    let (status, body) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[3], k, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = contest(&app, &p[3], k, 1, "Eu venci").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "ready");

    // No duelo, o capitão é o adversário: quem enviou reconhece o erro.
    assert_eq!(body["games"][0]["task"], "await_organizer");
    let right = json!({ "map": "awoken", "a_frags": 5, "b_frags": 10 });
    let (status, body) = accept(&app, &p[0], k, 1, right).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        (
            &body["summary"]["state"],
            &body["summary"]["result"]["winner"]
        ),
        (&json!("completed"), &json!("b"))
    );
    let b = bracket(&app, &ana).await;
    assert_eq!(side_player(&b, "wb-2-1", "a"), p[3].id);
}

#[tokio::test]
async fn accepting_keeps_the_games_after_the_contested_one() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let k = "wb-1-1";
    let (status, body) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = contest(&app, &p[1], k, 1, "Eu venci").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Contestar não trava a próxima.
    let (status, body) = report(&app, &p[0], k, 2, game("a", "vestibule-of-exile")).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let right = json!({ "map": "awoken", "a_frags": 5, "b_frags": 10 });
    let (status, body) = accept(&app, &p[0], k, 1, right).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let games = body["games"].as_array().unwrap();
    assert_eq!(
        (
            &games[0]["status"],
            &games[0]["winner"],
            &games[1]["status"]
        ),
        (&json!("confirmed"), &json!("b"), &json!("pending"))
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
    let (status, body) = report(&app, &p[0], k, 2, game("a", "vestibule-of-exile")).await;
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

/// Contesta `n` propondo `proposal` no lugar do resultado enviado.
async fn contest_with(
    app: &axum::Router,
    who: &User,
    key: &str,
    n: u8,
    proposal: Value,
) -> (StatusCode, Value) {
    let uri = game_uri(key, n, "/contest");
    post(
        app,
        &uri,
        who,
        json!({ "comment": "Foi 5 a 10", "proposal": proposal }),
    )
    .await
}

fn ok((status, body): (StatusCode, Value)) -> Value {
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[tokio::test]
async fn a_contest_may_carry_the_right_result() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let k = "wb-1-1";
    ok(report(&app, &p[0], k, 1, game("a", "awoken")).await);
    ok(report(&app, &p[0], k, 2, game("a", "vestibule-of-exile")).await);

    for (proposal, code) in [
        (game("a", "awoken"), "match.contest_unchanged"),
        (game("b", "q3dm17"), "match.map_unknown"),
        (json!({ "a_frags": 5, "b_frags": 10 }), "match.map_required"),
        (
            json!({ "map": "awoken", "a_frags": 7, "b_frags": 7 }),
            "match.game_tied",
        ),
        (
            json!({ "map": "awoken", "a_frags": 1000, "b_frags": 0 }),
            "match.frags_invalid",
        ),
        // O mapa da outra partida da série não serve.
        (game("b", "vestibule-of-exile"), "match.map_repeated"),
    ] {
        let (status, body) = contest_with(&app, &p[1], k, 1, proposal).await;
        assert_eq!(
            (status, &body["error"]),
            (StatusCode::UNPROCESSABLE_ENTITY, &json!(code))
        );
    }
    // Nada ficou gravado: a partida segue pendente.
    assert_eq!(
        detail(&app, &p[1], k).await["games"][0]["status"],
        "pending"
    );

    // Vem com o que é preciso para mostrar e aplicar.
    let body = ok(contest_with(&app, &p[1], k, 1, game("b", "insomnia")).await);
    let g = &body["games"][0];
    let proposal = &g["contest_proposal"];
    assert_eq!(g["status"], "contested");
    assert_eq!(
        (
            &proposal["map"]["slug"],
            &proposal["a_frags"],
            &proposal["b_frags"]
        ),
        (&json!("insomnia"), &json!(5), &json!(10))
    );
    assert!(proposal["map"]["name"].is_string());
    // O histórico guarda o comentário, não a proposta.
    let event = body["history"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(event["kind"], "contested");
    assert_eq!(event["outcome"], Value::Null);

    // Sem proposta, continua sem.
    let body = ok(contest(&app, &p[1], k, 2, "Foi o B").await);
    assert_eq!(body["games"][1]["contest_proposal"], Value::Null);
}

#[tokio::test]
async fn resolving_the_contest_drops_the_proposal() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(5, 5)), &[&p[0], &p[1]]).await;
    let k = "wb-1-1";
    let right = |map: &str| game("b", map);
    let settled = |g: &Value, winner: &str| {
        assert_eq!(
            (&g["status"], &g["winner"], &g["contest_proposal"]),
            (&json!("confirmed"), &json!(winner), &Value::Null)
        );
    };

    // A organização aplica a proposta pelo PUT.
    ok(report(&app, &p[0], k, 1, game("a", "awoken")).await);
    ok(contest_with(&app, &p[1], k, 1, right("awoken")).await);
    let body = ok(set_game(&app, &ana, k, 1, right("awoken"), None).await);
    settled(&body["games"][0], "b");

    // O capitão adversário, pelo aceite.
    ok(report(&app, &p[0], k, 2, game("a", "vestibule-of-exile")).await);
    ok(contest_with(&app, &p[1], k, 2, right("vestibule-of-exile")).await);
    let body = ok(accept(&app, &p[0], k, 2, right("vestibule-of-exile")).await);
    settled(&body["games"][1], "b");

    // Manter o enviado também a apaga.
    ok(report(&app, &p[0], k, 3, game("a", "insomnia")).await);
    ok(contest_with(&app, &p[1], k, 3, right("insomnia")).await);
    let body = ok(confirm(&app, &ana, k, 3).await);
    settled(&body["games"][2], "a");

    // Anular leva a linha inteira.
    ok(report(&app, &p[0], k, 4, game("a", "corrupted-keep")).await);
    ok(contest_with(&app, &p[1], k, 4, right("corrupted-keep")).await);
    let uri = game_uri(k, 4, "/replay");
    let body = ok(post(&app, &uri, &ana, json!({})).await);
    assert_eq!(body["games"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn a_late_contest_may_carry_the_result_on_the_vetoed_map() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let veto = json!({ "veto": { "map_mode": "veto", "champion_phase": false } });
    let everyone = [&p[0], &p[1], &p[2], &p[3]];
    start(&app, &ana, copa(with(single(1, 1), veto)), &everyone).await;
    // wb-1-1: P1 x P4.
    let k = "wb-1-1";
    let (first, second) = (&p[0], &p[3]);
    let veto_uri = |action: &str| format!("{COPA}/matches/{k}/veto/{action}");
    for who in [first, second] {
        let (status, body) = post_empty(&app, &veto_uri("ready"), who).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let map = loop {
        let seen = detail(&app, first, k).await;
        if seen["veto"]["status"] == "done" {
            break seen["veto"]["games"][0]["map"]["slug"]
                .as_str()
                .unwrap()
                .to_owned();
        }
        let who = if seen["veto"]["turn"]["sides"] == json!(["a"]) {
            first
        } else {
            second
        };
        let option = detail(&app, who, k).await["veto"]["options"][0].clone();
        ok(post(&app, &veto_uri("steps"), who, json!({ "map": option })).await);
    };
    ok(report(&app, first, k, 1, game("a", &map)).await);
    ok(confirm(&app, second, k, 1).await);

    let other = if map == "awoken" {
        "insomnia"
    } else {
        "awoken"
    };
    for (proposal, code) in [
        (game("a", &map), "match.contest_unchanged"),
        // Decidida, a série não deixa de estar presa ao mapa do veto.
        (game("b", other), "match.map_mismatch"),
    ] {
        let (status, body) = contest_with(&app, second, k, 1, proposal).await;
        assert_eq!(
            (status, &body["error"]),
            (StatusCode::UNPROCESSABLE_ENTITY, &json!(code))
        );
    }
    let body = ok(contest_with(&app, second, k, 1, game("b", &map)).await);
    assert_eq!(body["summary"]["state"], "ready");
    assert_eq!(body["games"][0]["contest_proposal"]["b_frags"], 10);
    let body = ok(accept(&app, first, k, 1, game("b", &map)).await);
    assert_eq!(
        (
            &body["summary"]["state"],
            &body["summary"]["result"]["winner"]
        ),
        (&json!("completed"), &json!("b"))
    );
}
