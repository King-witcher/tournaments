//! Correção de mapas pelo organizador (replay com a série corrigida),
//! encerramento automático e desfazer o início com resultados.

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

    // Dar o mapa 1 para P4 reabre a série (1 x 1), mas a final, decidida,
    // foi P1 x P2.
    let conflicts = (StatusCode::CONFLICT, err("match.correction_conflicts"));
    assert_eq!(
        set_game(&app, &ana, "wb-1-1", 1, game("b", "awoken"), None).await,
        conflicts
    );
    // O mesmo vencedor, com frags, passa, mesmo encerrado.
    let with_frags = json!({ "winner": "a", "map": "awoken", "a_frags": 30, "b_frags": 28 });
    let (status, fixed) = set_game(&app, &ana, "wb-1-1", 1, with_frags, Some("frags")).await;
    assert_eq!(status, StatusCode::OK, "{fixed}");
    assert_eq!(kinds(&fixed).last().unwrap(), "corrected:1");
    assert_eq!(fixed["games"][0]["a_frags"], 30);
    assert_eq!(fixed["status"], "finished");
    // W.O. para P4 também contradiz a final.
    assert_eq!(
        set_result(&app, &ana, "wb-1-1", walkover("b"), None).await,
        conflicts
    );
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
    // A final tem um mapa de P1 x P2 enviado.
    let (status, _) = report(&app, &p[1], "wb-2-1", 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK);

    // O mapa 1 de wb-1-1 era de P4: a série reabre em 1 x 1 e a final perde
    // o lado (o mapa enviado nela é anulado).
    let (status, reopened) = set_game(&app, &ana, "wb-1-1", 1, game("b", "awoken"), None).await;
    assert_eq!(status, StatusCode::OK, "{reopened}");
    assert_eq!(
        (&reopened["summary"]["state"], &reopened["next_game"]),
        (&json!("ready"), &json!(3))
    );
    assert_eq!(
        kinds(&reopened)[5..],
        ["corrected:1".to_owned(), "reopened".to_owned()]
    );
    let final_ = detail(&app, &p[1], "wb-2-1").await;
    assert_eq!(kinds(&final_), ["reported:1", "voided:1"]);
    assert_eq!(final_["games"], json!([]));

    // O 2 também: P4 vence 2 x 0 e vai à final.
    let (status, fixed) = set_game(&app, &ana, "wb-1-1", 2, game("b", "exile"), None).await;
    assert_eq!(status, StatusCode::OK, "{fixed}");
    assert_eq!(fixed["summary"]["result"]["b_wins"], 2);
    let b = bracket(&app, &ana).await;
    let final_ = find(&b, "wb-2-1");
    assert_eq!(sides(final_), (json!(p[3].id), json!(p[1].id)));
    assert_eq!(final_["task"], Value::Null);
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
    // Dupla com reset: wb-1-1, a grande final (md5) e o reset.
    start(&app, &ana, copa(json!({})), &[&p[0], &p[1]]).await;
    play(&app, &p[0], &p[1], "wb-1-1", 2, 0).await;
    // Mapas da grande final: P2, P1, P1, P1.
    play(&app, &p[0], &p[1], "gf-1-1", 3, 1).await;
    let b = bracket(&app, &p[1]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(find(&b, "gf-2-1")["state"], "not_needed");

    // O mapa 2 era de P2: 2 x 2, a grande final reabre e o torneio volta.
    let (status, fixed) = set_game(&app, &ana, "gf-1-1", 2, game("b", MAPS[1]), None).await;
    assert_eq!(status, StatusCode::OK, "{fixed}");
    assert_eq!(fixed["status"], "in_progress");
    assert_eq!(fixed["next_game"], 5);
    // O quinto, lançado para P2, decide a grande final para quem veio de
    // baixo: o reset passa a valer.
    let (status, _) = set_game(&app, &ana, "gf-1-1", 5, game("b", MAPS[4]), None).await;
    assert_eq!(status, StatusCode::OK);
    let b = bracket(&app, &p[1]).await;
    assert_eq!(find(&b, "gf-1-1")["result"]["b_wins"], 3);
    let reset = find(&b, "gf-2-1");
    assert_eq!(
        (&reset["state"], &reset["task"]),
        (&json!("ready"), &json!("report"))
    );
    assert_eq!(b["standings"], json!([]));

    // O reset decide: P1 é o campeão.
    play(&app, &p[1], &p[0], "gf-2-1", 3, 0).await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(b["standings"][0]["user_id"], p[0].id);

    // Agora a grande final não pode mais voltar atrás: o reset foi jogado.
    assert_eq!(
        set_game(&app, &ana, "gf-1-1", 5, game("a", MAPS[4]), None).await,
        (StatusCode::CONFLICT, err("match.correction_conflicts"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn undo_start_discards_maps_results_and_history(db: PgPool) {
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
    let (status, _) = report(&app, &p[1], "wb-1-2", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bracket(&app, &ana).await["results"], 2);

    let (status, undone) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{undone}");
    let left = sqlx::query_scalar!(
        r#"select (select count(*) from match_results) + (select count(*) from match_games)
                  + (select count(*) from match_events) as "n!""#
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
