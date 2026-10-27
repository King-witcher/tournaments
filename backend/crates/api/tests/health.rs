use std::sync::Arc;

use api::{AppState, auth::AuthSettings, discord::HttpDiscord};
use axum::{
    body::Body,
    http::{HeaderValue, Request, StatusCode},
};
use http_body_util::BodyExt;
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test(migrations = "../../migrations")]
async fn health_reports_database_ok(db: PgPool) {
    let app = api::app(
        AppState {
            db,
            discord: Arc::new(HttpDiscord::new(String::new(), String::new())),
            auth: Arc::new(AuthSettings {
                discord_client_id: None,
                redirect_uri: String::new(),
                web_origin: "http://localhost:3000".into(),
                cookie_domain: None,
                cookie_secure: false,
            }),
        },
        HeaderValue::from_static("http://localhost:3000"),
    );

    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"database":"ok"}"#);
}
