//! Contestação de um mapa e resolução pelo organizador (inclusive em partida
//! que ele joga), confirmar na hora, lançar mapas e W.O. na série.

use axum::http::StatusCode;
use serde_json::{Value, json};

use super::*;
use crate::common::*;

#[tokio::test]
async fn a_contested_map_waits_for_the_organizer_without_holding_the_next() {
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
    let k = "wb-1-1";

    // P4 diz que venceu o mapa 1; P1 contesta.
    let (status, _) = report(&app, &p[3], k, 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        contest(&app, &p[0], k, 1, "  ").await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("match.comment_invalid")
        )
    );
    assert_eq!(
        contest(&app, &p[1], k, 1, "não joguei").await,
        (StatusCode::FORBIDDEN, err("match.not_player"))
    );
    let (status, contested) = contest(&app, &p[0], k, 1, "Eu venci o mapa 1").await;
    assert_eq!(status, StatusCode::OK, "{contested}");
    let g = &contested["games"][0];
    assert_eq!(
        (
            &g["status"],
            &g["contest_comment"],
            &g["confirm_by"],
            &g["task"]
        ),
        (
            &json!("contested"),
            &json!("Eu venci o mapa 1"),
            &Value::Null,
            &json!("await_organizer")
        )
    );
    assert_eq!(
        contest(&app, &p[0], k, 1, "de novo").await,
        (StatusCode::CONFLICT, err("match.already_contested"))
    );
    assert_eq!(
        confirm(&app, &p[0], k, 1).await,
        (StatusCode::CONFLICT, err("match.contested"))
    );
    let b = bracket(&app, &ana).await;
    assert_eq!(
        (&find(&b, k)["task"], &find(&b, k)["contested_maps"]),
        (&json!("resolve"), &json!(1))
    );

    // A contestação não trava o resto: os mapas 2 e 3 seguem.
    for (n, side, map) in [(2, "b", "exile"), (3, "a", "insomnia")] {
        let (status, body) = report(&app, &p[0], k, n, game(side, map)).await;
        assert_eq!(status, StatusCode::OK, "{n}: {body}");
        let (status, body) = confirm(&app, &p[3], k, n).await;
        assert_eq!(status, StatusCode::OK, "{n}: {body}");
    }
    // Contestado não confirma pelo tempo, e a série espera por ele.
    age(&db, 60).await;
    let b = bracket(&app, &ana).await;
    assert_eq!(
        (&find(&b, k)["state"], &find(&b, k)["contested_maps"]),
        (&json!("ready"), &json!(1))
    );

    // Jogador não lança mapa.
    assert_eq!(
        set_game(&app, &p[0], k, 1, game("a", "awoken"), None).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    // Ana resolve com o mapa para P1: 2 x 1 nos três mapas.
    let (status, resolved) = set_game(
        &app,
        &ana,
        k,
        1,
        game("a", "awoken"),
        Some("Vi o screenshot"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{resolved}");
    let result = &resolved["summary"]["result"];
    assert_eq!(
        (&result["winner"], &result["a_wins"], &result["b_wins"]),
        (&json!("a"), &json!(2), &json!(1))
    );
    let history = resolved["history"].as_array().unwrap();
    let resolution = history.iter().find(|e| e["kind"] == "resolved").unwrap();
    assert_eq!(
        (
            &resolution["game"],
            &resolution["actor"]["user_id"],
            &resolution["comment"]
        ),
        (&json!(1), &json!(ana.id), &json!("Vi o screenshot"))
    );
    assert_eq!(resolved["games"][0]["confirmed_via"], "organizer");
    assert_eq!(kinds(&resolved).last().unwrap(), "decided");
    assert_eq!(
        find(&bracket(&app, &p[0]).await, "wb-2-1")["a"]["user_id"],
        p[0].id
    );
}

#[tokio::test]
async fn maps_after_the_decision_are_voided_and_organizers_confirm_right_away() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let k = "wb-1-1";

    // Mapa 1 de P1, contestado; o 2 de P1 e o 3 de P2 entram mesmo assim
    // (o contestado não conta para decidir a série).
    let (status, _) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = contest(&app, &p[1], k, 1, "Não foi assim").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = report(&app, &p[0], k, 2, game("a", "exile")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = report(&app, &p[1], k, 3, game("b", "insomnia")).await;
    assert_eq!(status, StatusCode::OK);

    // Ana aceita o mapa 1 como foi enviado: com 2 x 0 o terceiro não
    // existiria e é anulado. O 2 ainda espera P2.
    let (status, resolved) = confirm(&app, &ana, k, 1).await;
    assert_eq!(status, StatusCode::OK, "{resolved}");
    assert_eq!(resolved["games"].as_array().unwrap().len(), 2);
    assert_eq!(resolved["summary"]["result"], Value::Null);
    let d = &resolved;
    assert!(kinds(d).contains(&"resolved:1".to_owned()));
    assert!(kinds(d).contains(&"voided:3".to_owned()));

    // Confirmar agora: ela não espera P2 nem a janela.
    let (status, now) = confirm(&app, &ana, k, 2).await;
    assert_eq!(status, StatusCode::OK, "{now}");
    assert_eq!(now["games"][1]["confirmed_via"], "organizer");
    let history = now["history"].as_array().unwrap();
    let confirmed = history
        .iter()
        .rev()
        .find(|e| e["kind"] == "confirmed")
        .unwrap();
    assert_eq!(
        (&confirmed["game"], &confirmed["actor"]["user_id"]),
        (&json!(2), &json!(ana.id))
    );
    assert_eq!(
        (&now["summary"]["result"]["a_wins"], &now["status"]),
        (&json!(2), &json!("finished"))
    );
}

#[tokio::test]
async fn an_organizer_who_plays_can_resolve_their_own_map() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    start(&app, &ana, copa(single(1, 1)), &[&ana, &bia]).await;

    let (status, sent) = report(&app, &ana, "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sent["summary"]["task"], "await_opponent");
    let (status, _) = contest(&app, &bia, "wb-1-1", 1, "Não foi assim").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        find(&bracket(&app, &bia).await, "wb-1-1")["task"],
        "await_organizer"
    );

    // Ana resolve aceitando o próprio mapa; fica registrado quem foi.
    let (status, resolved) = confirm(&app, &ana, "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{resolved}");
    assert_eq!(
        kinds(&resolved),
        ["reported:1", "contested:1", "resolved:1", "decided"]
    );
    assert_eq!(resolved["history"][2]["actor"]["user_id"], ana.id);
    assert_eq!(resolved["history"][2]["outcome"]["winner"], ana.id);
    assert_eq!(resolved["status"], "finished");
}

#[tokio::test]
async fn organizers_enter_maps_and_walkovers() {
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

    // Lançados direto, sem ninguém ter enviado: já confirmados.
    for (n, map) in [(1, "awoken"), (2, "exile")] {
        let (status, body) = set_game(&app, &ana, "wb-1-2", n, game("b", map), None).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let d = detail(&app, &ana, "wb-1-2").await;
    assert_eq!(kinds(&d), ["entered:1", "entered:2", "decided"]);
    assert_eq!(d["summary"]["result"]["winner"], "b");
    assert_eq!(
        set_game(&app, &ana, "wb-1-2", 3, game("a", "insomnia"), None).await,
        (StatusCode::CONFLICT, err("match.series_decided"))
    );

    // W.O. com um mapa pendente: o mapa sai, a série vira W.O.
    let (status, _) = report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, wo) =
        set_result(&app, &ana, "wb-1-1", walkover("b"), Some("P1 não apareceu")).await;
    assert_eq!(status, StatusCode::OK, "{wo}");
    let result = &wo["summary"]["result"];
    assert_eq!(
        (&result["walkover"], &result["winner"], &result["a_wins"]),
        (&json!(true), &json!("b"), &Value::Null)
    );
    assert_eq!(wo["games"], json!([]));
    assert_eq!(kinds(&wo), ["reported:1", "voided:1", "walkover"]);
    assert_eq!(
        wo["history"][2]["outcome"],
        json!({ "winner": p[3].id, "winner_nickname": "P4", "walkover": true, "winner_maps": null, "loser_maps": null })
    );
    let b = bracket(&app, &p[3]).await;
    assert_eq!(sides(find(&b, "wb-2-1")), (json!(p[3].id), json!(p[2].id)));

    // Não dá para lançar o que ainda não tem lados.
    let (status, _) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        set_game(&app, &ana, "wb-2-1", 1, game("a", "awoken"), None).await,
        (StatusCode::CONFLICT, err("match.not_ready"))
    );
    assert_eq!(
        set_result(&app, &ana, "wb-2-1", walkover("a"), None).await,
        (StatusCode::CONFLICT, err("match.not_ready"))
    );
}
