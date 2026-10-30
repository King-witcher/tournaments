//! Mapa a mapa: um jogador envia, o adversário confirma (ou a janela acaba),
//! a série fecha na maioria e só então a chave avança; o que dá código no
//! caminho.

use axum::http::StatusCode;
use serde_json::{Value, json};

use super::*;
use crate::common::*;

#[tokio::test]
async fn maps_are_confirmed_one_by_one_and_the_series_closes_on_the_majority() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;

    // P4 (lado b) envia o mapa 1, vencido por P1, com frags; os de P4,
    // negativos (suicídio e queda tiram frag).
    let first = json!({ "winner": "a", "map": "awoken", "a_frags": 20, "b_frags": -1 });
    let (status, sent) = report(&app, &p[3], "wb-1-1", 1, first).await;
    assert_eq!(status, StatusCode::OK, "{sent}");
    let g = &sent["games"][0];
    assert_eq!((&g["a_frags"], &g["b_frags"]), (&json!(20), &json!(-1)));
    assert_eq!(
        (
            &g["status"],
            &g["winner"],
            &g["reported_side"],
            &g["map"]["name"]
        ),
        (
            &json!("pending"),
            &json!("a"),
            &json!("b"),
            &json!("Awoken")
        )
    );
    assert!(g["confirm_by"].is_string());
    assert_eq!(g["task"], "await_opponent");
    // Quem enviou já pode mandar o próximo, sem esperar a confirmação.
    assert_eq!(
        (&sent["next_game"], &sent["summary"]["task"]),
        (&json!(2), &json!("report"))
    );

    let b = bracket(&app, &p[0]).await;
    let m = find(&b, "wb-1-1");
    assert_eq!(
        (&m["task"], &m["pending_maps"], &m["a_maps"]),
        (&json!("respond"), &json!(1), &json!(0))
    );

    let (status, confirmed) = confirm(&app, &p[0], "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{confirmed}");
    assert_eq!(confirmed["games"][0]["confirmed_via"], "opponent");
    // 1 x 0 ainda não decide a md3: a final continua esperando.
    let b = bracket(&app, &p[0]).await;
    let m = find(&b, "wb-1-1");
    assert_eq!(
        (&m["state"], &m["a_maps"], &m["result"]),
        (&json!("ready"), &json!(1), &Value::Null)
    );
    assert_eq!(side_player(&b, "wb-2-1", "a"), Value::Null);

    // O mapa 2, enviado por P1 e confirmado por P4, fecha a série.
    let (status, _) = report(&app, &p[0], "wb-1-1", 2, game("a", "exile")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        side_player(&bracket(&app, &p[0]).await, "wb-2-1", "a"),
        Value::Null
    );
    let (status, closed) = confirm(&app, &p[3], "wb-1-1", 2).await;
    assert_eq!(status, StatusCode::OK, "{closed}");
    let result = &closed["summary"]["result"];
    assert_eq!(
        (
            &closed["summary"]["state"],
            &result["winner"],
            &result["a_wins"],
            &result["b_wins"],
            &result["confirmed_via"]
        ),
        (
            &json!("completed"),
            &json!("a"),
            &json!(2),
            &json!(0),
            &json!("maps")
        )
    );
    assert_eq!(closed["next_game"], Value::Null);
    assert_eq!(
        kinds(&closed),
        [
            "reported:1",
            "confirmed:1",
            "reported:2",
            "confirmed:2",
            "decided"
        ]
    );
    let history = closed["history"].as_array().unwrap();
    assert_eq!(history[0]["actor"]["user_id"], p[3].id);
    assert_eq!(outcome(&history[0])["winner"], p[0].id);
    assert_eq!(
        outcome(&history[4]),
        json!({ "winner": p[0].id, "winner_nickname": "P1", "walkover": false, "winner_maps": 2, "loser_maps": 0 })
    );

    let b = bracket(&app, &p[0]).await;
    let final_ = find(&b, "wb-2-1");
    assert_eq!(
        (&player(&b, &final_["a"]["team_id"]), &final_["state"]),
        (&json!(p[0].id), &json!("pending"))
    );
    assert_eq!(
        standings(&b),
        json!([{ "user_id": p[3].id, "rank": 3, "tied": 2 }])
    );
}

#[tokio::test]
async fn reporting_rules() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let not_player = (StatusCode::FORBIDDEN, err("match.not_player"));
    let conflict = |code: &str| (StatusCode::CONFLICT, err(code));
    let invalid = |code: &str| (StatusCode::UNPROCESSABLE_ENTITY, err(code));
    let k = "wb-1-1";

    assert_eq!(
        report(&app, &p[1], k, 1, game("a", "awoken")).await,
        not_player
    );
    // O organizador que não joga também não envia (ele lança).
    assert_eq!(
        report(&app, &ana, k, 1, game("a", "awoken")).await,
        not_player
    );
    assert_eq!(
        report(&app, &p[0], "wb-2-1", 1, game("a", "awoken")).await,
        conflict("match.not_ready")
    );
    // Fora de ordem: o mapa 2 antes do 1.
    assert_eq!(
        report(&app, &p[0], k, 2, game("a", "awoken")).await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "match.game_out_of_order", "params": { "next": 1 } })
        )
    );
    assert_eq!(
        report(&app, &p[0], k, 1, json!({ "winner": "a" })).await,
        invalid("match.map_required")
    );
    assert_eq!(
        report(&app, &p[0], k, 1, game("a", "q3dm17")).await,
        invalid("match.map_unknown")
    );
    assert_eq!(
        report(
            &app,
            &p[0],
            k,
            1,
            json!({ "winner": "a", "map": "awoken", "a_frags": 5, "b_frags": 9 })
        )
        .await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "match.frags_disagree", "params": { "game": 1 } })
        )
    );
    assert_eq!(
        confirm(&app, &p[3], k, 1).await,
        (StatusCode::NOT_FOUND, err("match.game_not_found"))
    );

    let (status, _) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        report(&app, &p[3], k, 1, game("b", "exile")).await,
        conflict("match.game_already_reported")
    );
    // Mapa repetido na série.
    assert_eq!(
        report(&app, &p[3], k, 2, game("a", "awoken")).await,
        invalid("match.map_repeated")
    );
    assert_eq!(
        confirm(&app, &p[0], k, 1).await,
        (StatusCode::FORBIDDEN, err("match.own_report"))
    );
    assert_eq!(
        contest(&app, &p[0], k, 1, "errado").await,
        (StatusCode::FORBIDDEN, err("match.own_report"))
    );
    assert_eq!(confirm(&app, &p[1], k, 1).await, not_player);

    // O segundo mapa de A, ainda pendente, já decide pela soma dos enviados:
    // não há terceiro.
    let (status, _) = report(&app, &p[3], k, 2, game("a", "exile")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        report(&app, &p[3], k, 3, game("b", "insomnia")).await,
        conflict("match.series_decided")
    );
    let (status, _) = confirm(&app, &p[3], k, 1).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        confirm(&app, &p[3], k, 1).await,
        conflict("match.already_confirmed")
    );
    let (status, _) = confirm(&app, &p[0], k, 2).await;
    assert_eq!(status, StatusCode::OK);
    // Série decidida: nada mais entra.
    assert_eq!(
        report(&app, &p[0], k, 3, game("a", "insomnia")).await,
        conflict("match.series_decided")
    );
}

#[tokio::test]
async fn unanswered_maps_confirm_when_the_deadline_ends() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    for (n, map) in [(1, "awoken"), (2, "exile")] {
        let (status, body) = report(&app, &p[0], "wb-1-1", n, game("a", map)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    // Enviadas antes da duração prevista (15 minutos): os 10 da janela
    // contam do fim dela, desde a partida liberada (a 1 no início do
    // torneio, a 2 no envio da 1).
    assert_eq!(deadlines(&db).await, [25 * 60, 25 * 60]);

    // 24 minutos depois, ainda no prazo: seguem pendentes.
    age(&db, 24).await;
    let b = bracket(&app, &p[1]).await;
    assert_eq!(find(&b, "wb-1-1")["pending_maps"], 2);

    // Mais dois minutos: confirmam pelo tempo e a série fecha.
    age(&db, 2).await;
    let b = bracket(&app, &p[1]).await;
    let m = find(&b, "wb-1-1");
    assert_eq!(
        (
            &m["state"],
            &m["result"]["confirmed_via"],
            &m["pending_maps"]
        ),
        (&json!("completed"), &json!("maps"), &json!(0))
    );
    // Era a final: o torneio encerrou sozinho.
    assert_eq!(b["status"], "finished");
    assert_eq!(
        contest(&app, &p[1], "wb-1-1", 1, "tarde demais").await,
        (StatusCode::CONFLICT, err("tournament.not_in_progress"))
    );

    let d = detail(&app, &p[1], "wb-1-1").await;
    assert_eq!(
        kinds(&d),
        [
            "reported:1",
            "reported:2",
            "auto_confirmed:1",
            "auto_confirmed:2",
            "decided"
        ]
    );
    assert_eq!(d["history"][2]["actor"], Value::Null);
    assert_eq!(d["games"][1]["confirmed_via"], "deadline");
    // A série fecha quando o último mapa confirmou.
    assert_eq!(&d["history"][4]["at"], &d["games"][1]["confirmed_at"]);
    assert_eq!(
        &d["summary"]["result"]["confirmed_at"],
        &d["games"][1]["confirmed_at"]
    );
}
