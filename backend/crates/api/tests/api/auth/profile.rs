//! `PATCH /me`: o próprio usuário troca o nome exibido.

use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

use crate::common::*;

#[sqlx::test(migrations = "../../migrations")]
async fn changes_the_display_name(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;

    let (status, me) = patch(&app, "/me", &ana, json!({ "display_name": "  Ana QC  " })).await;
    assert_eq!(status, StatusCode::OK, "{me}");
    assert_eq!(me["display_name"], "Ana QC");
    assert_eq!(get(&app, "/me", &ana).await.1["display_name"], "Ana QC");
    // O apelido na comunidade continua o de antes.
    let (_, tenant) = get(&app, "/tenants/quake-br", &ana).await;
    assert_eq!(tenant["nickname"], "Ana");

    let invalid = (
        StatusCode::UNPROCESSABLE_ENTITY,
        err("auth.display_name_invalid"),
    );
    for bad in ["   ", "a\u{7}b", &"é".repeat(33)] {
        let body = json!({ "display_name": bad });
        assert_eq!(patch(&app, "/me", &ana, body).await, invalid, "{bad:?}");
    }
    assert_eq!(
        patch(&app, "/me", &ana, json!({})).await.1,
        err("request.invalid_body")
    );
    assert_eq!(get(&app, "/me", &ana).await.1["display_name"], "Ana QC");

    let (status, _) = call(
        &app,
        Method::PATCH,
        "/me",
        None,
        Some(json!({ "display_name": "X" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
