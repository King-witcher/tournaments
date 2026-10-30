use axum::http::{Method, StatusCode};

use crate::common::{app, call};

#[tokio::test]
async fn maps_list_the_duel_pool_first() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let (status, maps) = call(&app, Method::GET, "/maps", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let maps = maps.as_array().unwrap();
    assert_eq!(maps.len(), 22);
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
            "blood-run",
            "corrupted-keep",
            "molten-falls",
            "ruins-of-sarnath",
            "vale-of-pnath",
            "crucible",
            "deep-embrace",
            "exile",
            "insomnia"
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
            "blood-run",
            "corrupted-keep",
            "molten-falls",
            "ruins-of-sarnath",
            "vale-of-pnath"
        ]
    );
}
