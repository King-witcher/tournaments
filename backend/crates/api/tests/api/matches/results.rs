//! Um jogador envia, o adversário confirma (ou a janela acaba) e a chave
//! avança; o que dá código no caminho.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;

#[sqlx::test(migrations = "../../migrations")]
async fn the_bracket_advances_only_when_the_opponent_confirms(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;

    // P4 (lado b) envia a derrota por 2 x 0.
    let (status, sent) = report(&app, &p[3], "wb-1-1", score(2, 0)).await;
    assert_eq!(status, StatusCode::OK, "{sent}");
    let summary = &sent["summary"];
    assert_eq!(
        (&summary["state"], &summary["task"]),
        (&json!("ready"), &json!("await_opponent"))
    );
    let result = &summary["result"];
    assert_eq!(
        (
            &result["status"],
            &result["winner"],
            &result["reported_side"]
        ),
        (&json!("pending"), &json!("a"), &json!("b"))
    );
    assert!(result["confirm_by"].is_string());

    // Pendente não avança: a final ainda espera o vencedor.
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["results"], 1);
    assert_eq!(find(&b, "wb-1-1")["task"], "respond");
    assert_eq!(find(&b, "wb-2-1")["a"]["user_id"], Value::Null);

    let (status, confirmed) = confirm(&app, &p[0], "wb-1-1").await;
    assert_eq!(status, StatusCode::OK, "{confirmed}");
    assert_eq!(confirmed["summary"]["state"], "completed");
    assert_eq!(confirmed["summary"]["result"]["confirmed_via"], "opponent");
    assert_eq!(kinds(&confirmed), ["reported", "confirmed"]);
    let actors: Vec<&Value> = confirmed["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| &e["actor"]["user_id"])
        .collect();
    assert_eq!(actors, [&json!(p[3].id), &json!(p[0].id)]);
    assert_eq!(
        confirmed["history"][0]["outcome"],
        json!({ "winner": p[0].id, "winner_nickname": "P1", "walkover": false, "winner_maps": 2, "loser_maps": 0 })
    );

    let b = bracket(&app, &p[0]).await;
    let final_ = find(&b, "wb-2-1");
    assert_eq!(
        (&final_["a"]["user_id"], &final_["state"]),
        (&json!(p[0].id), &json!("pending"))
    );
    // P4 já tem a colocação: 3º–4º.
    assert_eq!(
        b["standings"],
        json!([{ "user_id": p[3].id, "rank": 3, "tied": 2 }])
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn reporting_rules(db: PgPool) {
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

    assert_eq!(report(&app, &p[1], "wb-1-1", score(2, 0)).await, not_player);
    // O organizador que não joga também não envia (ele lança pelo resultado).
    assert_eq!(report(&app, &ana, "wb-1-1", score(2, 0)).await, not_player);
    assert_eq!(
        report(&app, &p[0], "wb-2-1", score(2, 0)).await,
        (StatusCode::CONFLICT, err("match.not_ready"))
    );
    assert_eq!(
        report(&app, &p[0], "wb-1-1", score(1, 0)).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "match.invalid_score", "params": { "best_of": 3 } })
        )
    );
    assert_eq!(
        report(&app, &p[0], "wb-1-1", score(2, 2)).await.1["error"],
        "match.invalid_score"
    );
    assert_eq!(
        confirm(&app, &p[1], "wb-1-2").await,
        (StatusCode::CONFLICT, err("match.no_report"))
    );

    let (status, _) = report(&app, &p[0], "wb-1-1", score(2, 1)).await;
    assert_eq!(status, StatusCode::OK);
    let reported = (StatusCode::CONFLICT, err("match.already_reported"));
    assert_eq!(report(&app, &p[0], "wb-1-1", score(2, 1)).await, reported);
    assert_eq!(report(&app, &p[3], "wb-1-1", score(0, 2)).await, reported);
    assert_eq!(
        confirm(&app, &p[0], "wb-1-1").await,
        (StatusCode::FORBIDDEN, err("match.own_report"))
    );
    assert_eq!(confirm(&app, &p[1], "wb-1-1").await, not_player);
    assert_eq!(
        contest(&app, &p[0], "wb-1-1", "errado").await,
        (StatusCode::FORBIDDEN, err("match.own_report"))
    );

    let (status, _) = confirm(&app, &p[3], "wb-1-1").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(report(&app, &p[0], "wb-1-1", score(2, 1)).await, reported);
    assert_eq!(
        confirm(&app, &p[3], "wb-1-1").await,
        (StatusCode::CONFLICT, err("match.already_confirmed"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_map_detail_must_agree_with_the_score(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let with_games = |games: Value| json!({ "a_wins": 2, "b_wins": 1, "games": games });
    let game = |map: &str, a: u16, b: u16| json!({ "map": map, "a_frags": a, "b_frags": b });
    let invalid = |code: &str| (StatusCode::UNPROCESSABLE_ENTITY, err(code));

    let cases = [
        (
            json!([game("awoken", 20, 10), game("exile", 5, 10)]),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": "match.games_count_mismatch", "params": { "expected": 3, "got": 2 } }),
            ),
        ),
        (
            json!([
                game("awoken", 20, 10),
                game("exile", 10, 10),
                game("insomnia", 30, 2)
            ]),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": "match.game_tied", "params": { "game": 2 } }),
            ),
        ),
        (
            json!([
                game("awoken", 20, 10),
                game("awoken", 5, 10),
                game("insomnia", 30, 2)
            ]),
            invalid("match.map_repeated"),
        ),
        (
            json!([
                game("awoken", 20, 10),
                game("q3dm17", 5, 10),
                game("insomnia", 30, 2)
            ]),
            invalid("match.map_unknown"),
        ),
        // A ganhou o terceiro mapa depois de já ter 2: a série acabaria antes.
        (
            json!([
                game("awoken", 20, 10),
                game("exile", 25, 10),
                game("insomnia", 3, 30)
            ]),
            invalid("match.games_disagree"),
        ),
        (
            json!([
                game("awoken", 1000, 10),
                game("exile", 5, 10),
                game("insomnia", 30, 2)
            ]),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": "match.frags_invalid", "params": { "max": 999 } }),
            ),
        ),
    ];
    for (games, expected) in cases {
        assert_eq!(
            report(&app, &p[1], "wb-1-1", with_games(games.clone())).await,
            expected,
            "{games}"
        );
    }

    let games = json!([
        game("awoken", 20, 10),
        game("exile", 5, 10),
        game("insomnia", 30, 2)
    ]);
    let (status, sent) = report(&app, &p[1], "wb-1-1", with_games(games)).await;
    assert_eq!(status, StatusCode::OK, "{sent}");
    assert_eq!(
        sent["games"][1],
        json!({ "number": 2, "map": { "slug": "exile", "name": "Exile" }, "a_frags": 5, "b_frags": 10 })
    );
    assert_eq!(sent["games"].as_array().unwrap().len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn unanswered_reports_confirm_when_the_window_ends(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (status, _) = report(&app, &p[0], "wb-1-1", score(2, 0)).await;
    assert_eq!(status, StatusCode::OK);
    let window = sqlx::query_scalar!(
        r#"select extract(epoch from confirm_by - reported_at)::integer as "seconds!"
           from match_results"#
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(window, 5 * 60);

    // Ainda na janela: segue pendente.
    let b = bracket(&app, &p[1]).await;
    assert_eq!(find(&b, "wb-1-1")["result"]["status"], "pending");

    // Cinco minutos depois (o relógio volta no banco em vez de esperar).
    sqlx::query!("update match_events set created_at = created_at - interval '6 minutes'")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query!(
        "update match_results set reported_at = reported_at - interval '6 minutes',
                                  confirm_by = confirm_by - interval '6 minutes'"
    )
    .execute(&db)
    .await
    .unwrap();
    let b = bracket(&app, &p[1]).await;
    let m = find(&b, "wb-1-1");
    assert_eq!(
        (&m["state"], &m["result"]["confirmed_via"]),
        (&json!("completed"), &json!("deadline"))
    );
    // Era a final: o torneio encerrou sozinho.
    assert_eq!(b["status"], "finished");
    let (_, seen) = get(&app, COPA, &p[1]).await;
    assert_eq!(seen["status"], "finished");
    assert_eq!(
        contest(&app, &p[1], "wb-1-1", "tarde demais").await,
        (StatusCode::CONFLICT, err("tournament.not_in_progress"))
    );

    let (_, detail) = get(&app, &format!("{COPA}/matches/wb-1-1"), &p[1]).await;
    assert_eq!(kinds(&detail), ["reported", "auto_confirmed"]);
    assert_eq!(detail["history"][1]["actor"], Value::Null);
    let confirmed_at = &detail["summary"]["result"]["confirmed_at"];
    assert_eq!(&detail["history"][1]["at"], confirmed_at);
}
