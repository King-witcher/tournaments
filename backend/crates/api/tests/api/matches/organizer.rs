//! Contestação, resolução pelo organizador (inclusive em partida que ele
//! joga), lançamento direto e W.O.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;

#[sqlx::test(migrations = "../../migrations")]
async fn contested_results_wait_for_the_organizer(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;

    // P4 diz que venceu; P1 contesta.
    let (status, _) = report(&app, &p[3], "wb-1-1", score(0, 2)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        contest(&app, &p[0], "wb-1-1", "  ").await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("match.comment_invalid")
        )
    );
    assert_eq!(
        contest(&app, &p[1], "wb-1-1", "não joguei").await,
        (StatusCode::FORBIDDEN, err("match.not_player"))
    );
    let (status, contested) = contest(&app, &p[0], "wb-1-1", "Eu venci por 2 x 1").await;
    assert_eq!(status, StatusCode::OK, "{contested}");
    let result = &contested["summary"]["result"];
    assert_eq!(
        (
            &result["status"],
            &result["contest_comment"],
            &result["confirm_by"]
        ),
        (
            &json!("contested"),
            &json!("Eu venci por 2 x 1"),
            &Value::Null
        )
    );
    assert_eq!(contested["summary"]["task"], "await_organizer");
    assert_eq!(
        contest(&app, &p[0], "wb-1-1", "de novo").await,
        (StatusCode::CONFLICT, err("match.already_contested"))
    );
    let waits = (StatusCode::CONFLICT, err("match.contested"));
    assert_eq!(confirm(&app, &p[0], "wb-1-1").await, waits);
    assert_eq!(confirm(&app, &p[3], "wb-1-1").await, waits);
    assert_eq!(
        find(&bracket(&app, &ana).await, "wb-1-1")["task"],
        "resolve"
    );
    // Contestada não confirma pelo tempo.
    sqlx::query!(
        "update match_results set reported_at = reported_at - interval '1 hour',
                                  confirm_by = confirm_by - interval '1 hour'"
    )
    .execute(&db)
    .await
    .unwrap();
    assert_eq!(
        find(&bracket(&app, &ana).await, "wb-1-1")["result"]["status"],
        "contested"
    );

    // Jogador não define resultado.
    assert_eq!(
        set_result(&app, &p[0], "wb-1-1", played(2, 1), None).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let (status, resolved) =
        set_result(&app, &ana, "wb-1-1", played(2, 1), Some("Vi o print")).await;
    assert_eq!(status, StatusCode::OK, "{resolved}");
    let result = &resolved["summary"]["result"];
    assert_eq!(
        (
            &result["status"],
            &result["winner"],
            &result["a_wins"],
            &result["reported_side"]
        ),
        (&json!("confirmed"), &json!("a"), &json!(2), &Value::Null)
    );
    assert_eq!(kinds(&resolved), ["reported", "contested", "resolved"]);
    let history = &resolved["history"];
    assert_eq!(history[1]["comment"], "Eu venci por 2 x 1");
    assert_eq!(history[2]["actor"]["user_id"], ana.id);
    assert_eq!(history[2]["comment"], "Vi o print");
    assert_eq!(
        history[2]["outcome"],
        json!({ "winner": p[0].id, "winner_nickname": "P1", "walkover": false, "winner_maps": 2, "loser_maps": 1 })
    );
    assert_eq!(resolved["can_manage"], true);
    assert_eq!(
        find(&bracket(&app, &p[0]).await, "wb-2-1")["a"]["user_id"],
        p[0].id
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_organizer_who_plays_can_resolve_their_own_match(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    start(&app, &ana, copa(single(3, 3)), &[&ana, &bia]).await;

    let (status, sent) = report(&app, &ana, "wb-1-1", score(2, 0)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sent["summary"]["task"], "await_opponent");
    let (status, _) = contest(&app, &bia, "wb-1-1", "Não foi assim").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        find(&bracket(&app, &bia).await, "wb-1-1")["task"],
        "await_organizer"
    );

    // Ana resolve aceitando o próprio placar; fica registrado quem foi.
    let (status, resolved) = confirm(&app, &ana, "wb-1-1").await;
    assert_eq!(status, StatusCode::OK, "{resolved}");
    assert_eq!(resolved["summary"]["result"]["confirmed_via"], "organizer");
    assert_eq!(kinds(&resolved), ["reported", "contested", "resolved"]);
    assert_eq!(resolved["history"][2]["actor"]["user_id"], ana.id);
    assert_eq!(resolved["history"][2]["outcome"]["winner"], ana.id);
    assert_eq!(resolved["status"], "finished");
}

#[sqlx::test(migrations = "../../migrations")]
async fn organizers_enter_results_and_walkovers(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;

    // Lançado direto, sem ninguém ter enviado.
    let (status, entered) = set_result(&app, &ana, "wb-1-2", played(1, 2), None).await;
    assert_eq!(status, StatusCode::OK, "{entered}");
    assert_eq!(kinds(&entered), ["entered"]);
    assert_eq!(entered["summary"]["result"]["winner"], "b");

    // W.O. sobre um pendente: o placar enviado sai.
    let (status, _) = report(&app, &p[0], "wb-1-1", score(2, 0)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, walkover) = set_result(
        &app,
        &ana,
        "wb-1-1",
        json!({ "kind": "walkover", "winner": "b" }),
        Some("P1 não apareceu"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{walkover}");
    let result = &walkover["summary"]["result"];
    assert_eq!(
        (&result["walkover"], &result["winner"], &result["a_wins"]),
        (&json!(true), &json!("b"), &Value::Null)
    );
    assert_eq!(kinds(&walkover), ["reported", "walkover"]);
    assert_eq!(
        walkover["history"][1]["outcome"],
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
        set_result(&app, &ana, "wb-2-1", played(2, 0), None).await,
        (StatusCode::CONFLICT, err("match.not_ready"))
    );
}
