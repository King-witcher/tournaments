//! Correção pelo organizador (replay com o histórico corrigido), encerramento
//! automático e desfazer o início com resultados.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;

#[sqlx::test(migrations = "../../migrations")]
async fn corrections_that_contradict_a_later_match_are_refused(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    play(&app, &p[0], &p[3], "wb-1-1", 2, 0).await;
    play(&app, &p[1], &p[2], "wb-1-2", 2, 1).await;
    play(&app, &p[0], &p[1], "wb-2-1", 2, 1).await;
    let b = bracket(&app, &ana).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(
        b["standings"][0],
        json!({ "user_id": p[0].id, "rank": 1, "tied": 1 })
    );

    // P4 venceria a primeira, mas a final (confirmada) foi P1 x P2.
    assert_eq!(
        set_result(&app, &ana, "wb-1-1", played(1, 2), None).await,
        (StatusCode::CONFLICT, err("match.correction_conflicts"))
    );
    // O mesmo vencedor com outro placar passa, mesmo encerrado.
    let (status, fixed) = set_result(&app, &ana, "wb-1-1", played(2, 1), Some("era 2 x 1")).await;
    assert_eq!(status, StatusCode::OK, "{fixed}");
    assert_eq!(kinds(&fixed), ["reported", "confirmed", "corrected"]);
    assert_eq!(fixed["summary"]["result"]["b_wins"], 1);
    assert_eq!(fixed["status"], "finished");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_valid_correction_rebuilds_the_bracket(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    play(&app, &p[0], &p[3], "wb-1-1", 2, 0).await;
    play(&app, &p[1], &p[2], "wb-1-2", 2, 1).await;
    // A final tem um envio pendente de P1 x P2.
    let (status, _) = report(&app, &p[1], "wb-2-1", score(0, 2)).await;
    assert_eq!(status, StatusCode::OK);

    let (status, fixed) = set_result(&app, &ana, "wb-1-1", played(0, 2), None).await;
    assert_eq!(status, StatusCode::OK, "{fixed}");
    let b = bracket(&app, &ana).await;
    let final_ = find(&b, "wb-2-1");
    assert_eq!(sides(final_), (json!(p[3].id), json!(p[1].id)));
    // O envio da final era de outra partida: foi anulado.
    assert_eq!(final_["result"], Value::Null);
    let (_, detail) = get(&app, &format!("{COPA}/matches/wb-2-1"), &p[3]).await;
    assert_eq!(kinds(&detail), ["reported", "voided"]);
    assert_eq!(detail["summary"]["task"], "report");
    assert_eq!(
        b["standings"],
        json!([
            { "user_id": p[0].id, "rank": 3, "tied": 2 },
            { "user_id": p[2].id, "rank": 3, "tied": 2 }
        ])
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_tournament_finishes_and_reopens_with_the_bracket(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    // Dupla com reset: wb-1-1, a grande final e o reset.
    start(&app, &ana, copa(json!({})), &[&p[0], &p[1]]).await;
    play(&app, &p[0], &p[1], "wb-1-1", 2, 0).await;
    play(&app, &p[0], &p[1], "gf-1-1", 3, 1).await;
    // Quem vem de cima venceu: o reset não precisou e acabou.
    let b = bracket(&app, &p[1]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(find(&b, "gf-2-1")["state"], "not_needed");
    assert_eq!(
        b["standings"],
        json!([
            { "user_id": p[0].id, "rank": 1, "tied": 1 },
            { "user_id": p[1].id, "rank": 2, "tied": 1 }
        ])
    );

    // Corrigida, a grande final passa a pedir o reset: volta a andar.
    let (status, fixed) = set_result(&app, &ana, "gf-1-1", played(2, 3), None).await;
    assert_eq!(status, StatusCode::OK, "{fixed}");
    assert_eq!(fixed["status"], "in_progress");
    let b = bracket(&app, &p[1]).await;
    assert_eq!(b["status"], "in_progress");
    let reset = find(&b, "gf-2-1");
    assert_eq!(
        (&reset["state"], &reset["task"]),
        (&json!("ready"), &json!("report"))
    );
    assert_eq!(b["standings"], json!([]));

    // O reset decide: P1, que veio de cima, vence e é o campeão.
    play(&app, &p[1], &p[0], "gf-2-1", 3, 0).await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(b["standings"][0]["user_id"], p[0].id);

    // Agora a grande final não pode mais voltar atrás: o reset foi jogado.
    assert_eq!(
        set_result(&app, &ana, "gf-1-1", played(3, 0), None).await,
        (StatusCode::CONFLICT, err("match.correction_conflicts"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn undo_start_discards_results_and_history(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    play(&app, &p[0], &p[3], "wb-1-1", 2, 0).await;
    let (status, _) = report(&app, &p[1], "wb-1-2", score(2, 0)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bracket(&app, &ana).await["results"], 2);

    let (status, undone) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{undone}");
    let left = sqlx::query_scalar!(
        r#"select (select count(*) from match_results) + (select count(*) from match_events)
           as "n!""#
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(left, 0);

    // Começar de novo dá uma chave limpa.
    let (status, _) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    let b = bracket(&app, &ana).await;
    assert_eq!(b["results"], 0);
    assert_eq!(find(&b, "wb-1-1")["state"], "ready");
}
