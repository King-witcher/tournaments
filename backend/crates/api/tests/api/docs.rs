use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

use crate::common::app;

async fn fetch(db: PgPool, uri: &str) -> (StatusCode, Vec<u8>) {
    let response = app(db)
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, body.to_vec())
}

/// O `gen:api` do front lê daqui.
#[sqlx::test(migrations = "../../migrations")]
async fn openapi_json_is_served(db: PgPool) {
    let (status, body) = fetch(db, "/openapi.json").await;

    assert_eq!(status, StatusCode::OK);
    let spec: Value = serde_json::from_slice(&body).unwrap();
    assert!(spec["openapi"].is_string());
    assert!(spec["paths"]["/health"].is_object());
}

#[sqlx::test(migrations = "../../migrations")]
async fn docs_serve_scalar(db: PgPool) {
    let (status, body) = fetch(db, "/docs").await;

    assert_eq!(status, StatusCode::OK);
    let html = String::from_utf8(body).unwrap();
    assert!(html.contains("@scalar/api-reference"));
}
