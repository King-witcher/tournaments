use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use tower::ServiceExt;

use crate::common::{app, call};

/// A lista é a mesma para todos e só muda num deploy: o navegador a guarda.
#[tokio::test]
async fn the_list_is_cached_by_the_browser() {
    let (db, _lease) = crate::db::test_db().await;
    let req = Request::get("/maps").body(Body::empty()).unwrap();
    let res = app(db).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CACHE_CONTROL], "public, max-age=300");
}

#[tokio::test]
async fn maps_list_the_duel_pool_first() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let (status, maps) = call(&app, Method::GET, "/maps", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let maps = maps.as_array().unwrap();
    assert_eq!(maps.len(), 20);
    let pool: Vec<&str> = maps
        .iter()
        .filter(|m| m["duel_pool"] == true)
        .map(|m| m["slug"].as_str().unwrap())
        .collect();
    // O padrão primeiro, na ordem dele; os outros do pool de duelo, por nome.
    assert_eq!(
        pool,
        [
            "awoken",
            "blood-covenant",
            "corrupted-keep",
            "molten-falls",
            "ruins-of-sarnath",
            "vale-of-pnath",
            "deep-embrace",
            "blood-run",
            "crucible",
            "insomnia",
            "vestibule-of-exile"
        ]
    );
    assert!(maps[..11].iter().all(|m| m["duel_pool"] == true));
    assert_eq!(maps[11]["slug"], "burial-chamber");
}

#[tokio::test]
async fn the_default_pool_is_marked_in_order() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let (_, maps) = call(&app, Method::GET, "/maps", None, None).await;
    let default: Vec<&str> = maps
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["default_pool"] == true)
        .map(|m| m["slug"].as_str().unwrap())
        .collect();
    assert_eq!(
        default,
        [
            "awoken",
            "blood-covenant",
            "corrupted-keep",
            "molten-falls",
            "ruins-of-sarnath",
            "vale-of-pnath",
            "deep-embrace"
        ]
    );
}
