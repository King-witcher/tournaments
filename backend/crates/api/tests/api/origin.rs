//! Escrita vinda de uma página de outra origem (`shared/origin.rs`).

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use tower::ServiceExt;

use crate::common::{WEB, app};

async fn status(
    app: &Router,
    method: Method,
    uri: &str,
    headers: &[(header::HeaderName, &str)],
) -> StatusCode {
    let mut req = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        req = req.header(name, *value);
    }
    let req = req.body(Body::empty()).unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

/// O logout responde 204 também sem sessão.
async fn logout(app: &Router, headers: &[(header::HeaderName, &str)]) -> StatusCode {
    status(app, Method::POST, "/auth/logout", headers).await
}

#[tokio::test]
async fn writes_only_from_the_site_or_without_origin() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    assert_eq!(
        logout(&app, &[]).await,
        StatusCode::NO_CONTENT,
        "sem Origin"
    );
    assert_eq!(
        logout(&app, &[(header::ORIGIN, WEB)]).await,
        StatusCode::NO_CONTENT
    );
    // O Scalar em /docs, na origem da própria API.
    let api = [
        (header::ORIGIN, "http://localhost:4000"),
        (header::HOST, "localhost:4000"),
    ];
    assert_eq!(logout(&app, &api).await, StatusCode::NO_CONTENT);
    for origin in ["https://outro.exemplo", "http://localhost:4002", "null"] {
        let headers = [(header::ORIGIN, origin), (header::HOST, "localhost:4000")];
        assert_eq!(
            logout(&app, &headers).await,
            StatusCode::FORBIDDEN,
            "{origin}"
        );
    }

    // Leitura passa: o CORS já não deixa a página de fora ler a resposta.
    let health = status(
        &app,
        Method::GET,
        "/health",
        &[(header::ORIGIN, "https://outro.exemplo")],
    );
    assert_eq!(health.await, StatusCode::OK);
}
