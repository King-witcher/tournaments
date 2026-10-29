//! Janela de contestação de resultado: padrão, PATCH e limites.

use axum::http::StatusCode;
use serde_json::json;

use crate::common::*;

const COPA: &str = "/tenants/quake-br/tournaments/copa";

#[tokio::test]
async fn contest_window_defaults_to_ten_and_changes_until_finished() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let created = create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(created["contest_minutes"], 10);

    let invalid = (
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": "tournament.contest_minutes_invalid", "params": { "min": 1, "max": 1440 } }),
    );
    assert_eq!(
        patch(&app, COPA, &ana, json!({ "contest_minutes": 0 })).await,
        invalid
    );
    let other = with(duel("outra"), json!({ "contest_minutes": 1441 }));
    assert_eq!(
        post(&app, "/tenants/quake-br/tournaments", &ana, other).await,
        invalid
    );

    // Muda com o torneio em andamento (vale para os próximos envios).
    set_status(&db, "copa", "in_progress").await;
    let (status, updated) = patch(&app, COPA, &ana, json!({ "contest_minutes": 15 })).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["contest_minutes"], 15);

    set_status(&db, "copa", "finished").await;
    assert_eq!(
        patch(&app, COPA, &ana, json!({ "contest_minutes": 30 })).await,
        (StatusCode::CONFLICT, err("tournament.finished"))
    );
    // Reenviar o valor atual não é mudança.
    let (status, _) = patch(&app, COPA, &ana, json!({ "contest_minutes": 15 })).await;
    assert_eq!(status, StatusCode::OK);
}
