//! O organizador anula o último mapa ainda sem confirmar para os jogadores o
//! jogarem de novo.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};

use super::*;
use crate::common::*;

async fn replay(
    app: &Router,
    who: &User,
    key: &str,
    n: u8,
    comment: Option<&str>,
) -> (StatusCode, Value) {
    post(
        app,
        &game_uri(key, n, "/replay"),
        who,
        json!({ "comment": comment }),
    )
    .await
}

#[tokio::test]
async fn a_contested_map_is_voided_and_played_again() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // Quatro jogadores: o torneio segue depois da wb-1-1 (seed 1 × seed 4).
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let k = "wb-1-1";

    // Md1: com o mapa 1 contestado, não sobra mapa para a revanche.
    let (status, _) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, contested) = contest(&app, &p[3], k, 1, "Caiu; combinamos de refazer").await;
    assert_eq!(status, StatusCode::OK, "{contested}");
    assert_eq!(contested["next_game"], Value::Null);

    // Só a organização anula.
    assert_eq!(
        replay(&app, &p[3], k, 1, None).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        replay(&app, &ana, k, 1, Some("  ")).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("match.comment_invalid")
        )
    );
    let (status, voided) = replay(&app, &ana, k, 1, Some("Queda no meio")).await;
    assert_eq!(status, StatusCode::OK, "{voided}");
    assert_eq!(voided["games"], json!([]));
    assert_eq!(voided["summary"]["contested_maps"], 0);
    assert_eq!(kinds(&voided), ["reported:1", "contested:1", "replay:1"]);
    let event = &voided["history"][2];
    assert_eq!(
        (
            &event["actor"]["user_id"],
            &event["comment"],
            &event["outcome"]
        ),
        (&json!(ana.id), &json!("Queda no meio"), &Value::Null)
    );
    assert_eq!(
        replay(&app, &ana, k, 1, None).await,
        (StatusCode::NOT_FOUND, err("match.game_not_found"))
    );

    // A revanche entra como mapa 1, no mesmo mapa, e decide a série.
    let d = detail(&app, &p[3], k).await;
    assert_eq!(d["next_game"], 1);
    let (status, _) = report(&app, &p[3], k, 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, decided) = confirm(&app, &p[0], k, 1).await;
    assert_eq!(status, StatusCode::OK, "{decided}");
    assert_eq!(decided["summary"]["result"]["winner"], "b");
    assert_eq!(
        replay(&app, &ana, k, 1, None).await,
        (StatusCode::CONFLICT, err("match.series_decided"))
    );
}

#[tokio::test]
async fn only_the_last_map_without_confirmation_is_voided() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let k = "wb-1-1";

    // Mapa 1 contestado e o 2 enviado depois: o 1 não é o último.
    let (status, _) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = contest(&app, &p[1], k, 1, "Não foi assim").await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = report(&app, &p[0], k, 2, game("b", "exile")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        replay(&app, &ana, k, 1, None).await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "match.game_not_last", "params": { "last": 2 } })
        )
    );

    // Anulando do fim para o começo, um de cada vez; vale para o pendente.
    let (status, body) = replay(&app, &ana, k, 2, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = replay(&app, &ana, k, 1, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        kinds(&body),
        [
            "reported:1",
            "contested:1",
            "reported:2",
            "replay:2",
            "replay:1"
        ]
    );

    // Confirmado não se anula: a organização corrige.
    let (status, _) = report(&app, &p[0], k, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = confirm(&app, &p[1], k, 1).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        replay(&app, &ana, k, 1, None).await,
        (StatusCode::CONFLICT, err("match.already_confirmed"))
    );
}
