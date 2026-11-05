//! Banimento de scanners. O que é armadilha e quem é scanner, caso a caso, está
//! nos testes de `src/scanners/traps.rs`; aqui, o que acontece com as rotas.

use api::{
    AppState,
    scanners::{Bans, SECRET_HEADER},
};
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::common::{INTERNAL_SECRET, app_from, redis, state, user};

const SCANNER: &str = "203.0.113.7";
const SOMEONE_ELSE: &str = "198.51.100.9";

struct Answer {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Answer {
    fn is_empty_404(&self) -> bool {
        self.status == StatusCode::NOT_FOUND && self.body.is_empty()
    }
}

async fn send(
    app: &Router,
    method: Method,
    uri: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> Answer {
    let mut req = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        req = req.header(*name, *value);
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap();
    let (parts, body) = res.into_parts();
    Answer {
        status: parts.status,
        headers: parts.headers,
        body: body.collect().await.unwrap().to_bytes().to_vec(),
    }
}

async fn get_from(app: &Router, ip: &str, uri: &str) -> Answer {
    send(app, Method::GET, uri, &[("cf-connecting-ip", ip)], "").await
}

async fn internal(app: &Router, method: Method, body: &str) -> Answer {
    let secret = [(SECRET_HEADER, INTERNAL_SECRET)];
    send(app, method, "/internal/bans", &secret, body).await
}

async fn listed(app: &Router) -> Value {
    let answer = internal(app, Method::GET, "").await;
    assert_eq!(answer.status, StatusCode::OK);
    serde_json::from_slice(&answer.body).unwrap()
}

#[tokio::test]
async fn a_scanner_gets_the_404_of_a_route_that_does_not_exist_from_then_on() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));

    assert_eq!(
        get_from(&app, SCANNER, "/health").await.status,
        StatusCode::OK
    );
    assert!(get_from(&app, SCANNER, "/.env").await.is_empty_404());

    let banned = get_from(&app, SCANNER, "/health").await;
    assert!(banned.is_empty_404());
    // Quem compara com uma rota que não existe não vê diferença.
    let unknown = get_from(&app, SOMEONE_ELSE, "/nada-aqui").await;
    assert!(unknown.is_empty_404());
    assert_eq!(banned.headers, unknown.headers);

    for method in [Method::POST, Method::OPTIONS, Method::DELETE] {
        let answer = send(
            &app,
            method,
            "/health",
            &[("cf-connecting-ip", SCANNER)],
            "",
        )
        .await;
        assert!(answer.is_empty_404());
    }
    // Os outros seguem normais.
    assert_eq!(
        get_from(&app, SOMEONE_ELSE, "/health").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn an_ipv6_ban_takes_the_whole_block() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));

    get_from(&app, "2001:db8:1:2::aaaa", "/wp-login.php").await;
    assert!(
        get_from(&app, "2001:db8:1:2::bbbb", "/health")
            .await
            .is_empty_404()
    );
    assert_eq!(
        get_from(&app, "2001:db8:1:3::aaaa", "/health").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_link_opened_in_a_browser_does_not_ban() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));
    let browser = [
        ("cf-connecting-ip", SCANNER),
        ("sec-fetch-site", "cross-site"),
    ];

    assert!(
        send(&app, Method::GET, "/wp-admin", &browser, "")
            .await
            .is_empty_404()
    );
    assert_eq!(
        get_from(&app, SCANNER, "/health").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_real_route_is_never_a_trap() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));

    let answer = get_from(&app, SCANNER, "/tenants/foo.php").await;
    let error: Value = serde_json::from_slice(&answer.body).unwrap();
    assert!(error["error"].is_string());
    assert_eq!(
        get_from(&app, SCANNER, "/health").await.status,
        StatusCode::OK
    );
}

/// A rede interna do Render: sem o `CF-Connecting-IP`, todo mundo teria este IP.
#[tokio::test]
async fn an_internal_address_is_never_banned() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));

    get_from(&app, "10.0.0.1", "/.env").await;
    assert_eq!(
        get_from(&app, "10.0.0.1", "/health").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_next_server_neither_bans_itself_nor_is_refused() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));
    let next = [
        ("cf-connecting-ip", SCANNER),
        (SECRET_HEADER, INTERNAL_SECRET),
    ];

    send(&app, Method::GET, "/.env", &next, "").await;
    assert_eq!(
        get_from(&app, SCANNER, "/health").await.status,
        StatusCode::OK
    );

    // O IP de saída da Vercel, banido pelo scanner de outro cliente dela.
    get_from(&app, SCANNER, "/.env").await;
    assert!(get_from(&app, SCANNER, "/health").await.is_empty_404());
    let answer = send(&app, Method::GET, "/health", &next, "").await;
    assert_eq!(answer.status, StatusCode::OK);
}

#[tokio::test]
async fn a_live_session_passes_the_ban() {
    let (db, _lease) = crate::db::test_db().await;
    let player = user(&db, "Quem divide o IP").await;
    let app = app_from(state(db));
    get_from(&app, SCANNER, "/.env").await;

    let live = [("cf-connecting-ip", SCANNER), ("cookie", &player.cookie)];
    let answer = send(&app, Method::GET, "/me", &live, "").await;
    assert_eq!(answer.status, StatusCode::OK);

    let invented = [
        ("cf-connecting-ip", SCANNER),
        ("cookie", "session=inventada"),
    ];
    assert!(
        send(&app, Method::GET, "/me", &invented, "")
            .await
            .is_empty_404()
    );
}

#[tokio::test]
async fn the_internal_route_does_not_exist_without_the_secret() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));

    for method in [Method::GET, Method::POST, Method::DELETE, Method::PUT] {
        let answer = send(&app, method, "/internal/bans", &[], SCANNER).await;
        assert!(answer.is_empty_404());
    }
    let wrong = [(SECRET_HEADER, "outro")];
    let answer = send(&app, Method::GET, "/internal/bans", &wrong, "").await;
    assert!(answer.is_empty_404());
    assert_eq!(listed(&app).await, json!([]));
}

#[tokio::test]
async fn the_web_reports_scanners_and_a_ban_can_be_undone() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));

    let reported = internal(&app, Method::POST, SCANNER).await;
    assert_eq!(reported.status, StatusCode::NO_CONTENT);
    assert_eq!(listed(&app).await, json!([SCANNER]));
    assert!(get_from(&app, SCANNER, "/health").await.is_empty_404());

    // O que chega do front também passa pelas regras.
    internal(&app, Method::POST, "10.0.0.1").await;
    assert_eq!(listed(&app).await, json!([SCANNER]));
    let junk = internal(&app, Method::POST, "não é ip").await;
    assert_eq!(junk.status, StatusCode::UNPROCESSABLE_ENTITY);

    let undone = internal(&app, Method::DELETE, SCANNER).await;
    assert_eq!(undone.status, StatusCode::NO_CONTENT);
    assert_eq!(listed(&app).await, json!([]));
    assert_eq!(
        get_from(&app, SCANNER, "/health").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn bans_survive_a_boot() {
    let (db, _lease) = crate::db::test_db().await;
    let redis = redis(&db);
    let first = state(db.clone());
    let bans = first.bans.clone();
    let app = app_from(first);
    get_from(&app, SCANNER, "/.env").await;
    get_from(&app, SOMEONE_ELSE, "/.env").await;
    bans.save().await;
    internal(&app, Method::DELETE, SOMEONE_ELSE).await;

    let reloaded = Bans::load(redis, Some(INTERNAL_SECRET)).await;
    let app = app_from(AppState {
        bans: reloaded,
        ..state(db)
    });
    assert!(get_from(&app, SCANNER, "/health").await.is_empty_404());
    assert_eq!(
        get_from(&app, SOMEONE_ELSE, "/health").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn without_the_secret_nothing_is_banned() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(AppState {
        bans: Bans::new(redis(&db), None),
        ..state(db)
    });

    assert!(get_from(&app, SCANNER, "/.env").await.is_empty_404());
    assert_eq!(
        get_from(&app, SCANNER, "/health").await.status,
        StatusCode::OK
    );
    let answer = send(
        &app,
        Method::GET,
        "/internal/bans",
        &[(SECRET_HEADER, "")],
        "",
    )
    .await;
    assert!(answer.is_empty_404());
}
