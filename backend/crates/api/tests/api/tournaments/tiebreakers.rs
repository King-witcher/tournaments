//! Os desempates escolhidos de uma fase de tabela: gravados na criação e na
//! edição, nulos na eliminação, travados depois do início. A recusa de lista
//! inválida está em `matches::swiss` (`table_formats_are_checked_on_create_and_edit`).

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::copa;
use crate::schema::{CHECK, exec, rejected, rule};

const COPA: &str = "/tenants/quake-br/tournaments/copa";

/// A coluna da fase 1 do torneio.
async fn stored(db: &PgPool) -> Option<Vec<String>> {
    sqlx::query_scalar("select tiebreakers from tournament_stages where position = 1")
        .fetch_one(db)
        .await
        .unwrap()
}

fn stage(format: &str, fields: Value) -> Value {
    json!({ "stages": [with(json!({ "format": format }), fields)] })
}

#[tokio::test]
async fn the_order_is_saved_on_create_and_edit_and_frozen_after_the_start() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    // Sem check-in, para iniciar direto.
    let body = copa(stage(
        "round_robin",
        json!({ "best_of": 3, "tiebreakers": ["frag_diff", "head_to_head"] }),
    ));
    let created = create_tournament(&app, &ana, body).await;
    assert_eq!(
        created["stages"][0]["tiebreakers"],
        json!(["frag_diff", "head_to_head"])
    );
    assert_eq!(
        stored(&db).await,
        Some(vec!["frag_diff".to_owned(), "head_to_head".to_owned()])
    );

    // Trocar para o suíço sem nenhum: decide a ordem da chave.
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        stage("swiss", json!({ "best_of": 3, "tiebreakers": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["stages"][0]["tiebreakers"], json!([]));
    assert_eq!(stored(&db).await, Some(vec![]));

    // Na eliminação, nenhum.
    let elimination = json!({ "bracket": "single", "best_of": 3, "finals_best_of": 5 });
    let (status, body) = patch(&app, COPA, &ana, stage("elimination", elimination)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["stages"][0].get("tiebreakers"), None);
    assert_eq!(stored(&db).await, None);

    // Começou: a ordem não muda, nem pela API nem direto no banco.
    let swiss = || {
        stage(
            "swiss",
            json!({ "best_of": 1, "tiebreakers": ["map_diff"] }),
        )
    };
    let (status, body) = patch(&app, COPA, &ana, swiss()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    let (status, body) = seed(&app, &ana, "copa", &[&ana, &bia]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // A mesma fase passa; outra ordem, não.
    let (status, body) = patch(&app, COPA, &ana, swiss()).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let other = stage(
        "swiss",
        json!({ "best_of": 1, "tiebreakers": ["head_to_head"] }),
    );
    assert_eq!(
        patch(&app, COPA, &ana, other).await,
        (StatusCode::CONFLICT, err("tournament.setup_locked"))
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                "update tournament_stages set tiebreakers = '{head_to_head}'"
            )
            .await
        ),
        rule(CHECK, "tournament_stages_started_frozen")
    );
    assert_eq!(stored(&db).await, Some(vec!["map_diff".to_owned()]));
}

#[tokio::test]
async fn the_lower_rating_is_accepted_anywhere_in_the_list() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let body = copa(stage(
        "round_robin",
        json!({ "best_of": 3, "tiebreakers": ["lower_rating", "head_to_head"] }),
    ));
    let created = create_tournament(&app, &ana, body).await;
    assert_eq!(
        created["stages"][0]["tiebreakers"],
        json!(["lower_rating", "head_to_head"])
    );
    // Os cinco no suíço, ele por último.
    let all = json!([
        "buchholz",
        "head_to_head",
        "map_diff",
        "frag_diff",
        "lower_rating"
    ]);
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        stage("swiss", json!({ "best_of": 3, "tiebreakers": all })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["stages"][0]["tiebreakers"], all);
    assert_eq!(stored(&db).await.unwrap().last().unwrap(), "lower_rating");
}
