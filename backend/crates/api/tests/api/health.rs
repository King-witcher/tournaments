use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::common::app;

#[tokio::test]
async fn health_reports_database_ok() {
    let (db, _lease) = crate::db::test_db().await;
    let response = app(db)
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"database":"ok"}"#);
}
