use axum::http::{Method, StatusCode};

use crate::common::{app, call};

#[tokio::test]
async fn maps_list_the_duel_pool_first() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let (status, maps) = call(&app, Method::GET, "/maps", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let maps = maps.as_array().unwrap();
    assert_eq!(maps.len(), 16);
    let pool: Vec<&str> = maps
        .iter()
        .filter(|m| m["duel_pool"] == true)
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(pool.len(), 10);
    assert_eq!(pool[0], "Awoken");
    assert!(maps[..10].iter().all(|m| m["duel_pool"] == true));
    assert_eq!(maps[10]["slug"], "burial-chamber");
}

#[tokio::test]
async fn the_default_pool_is_the_first_seven_of_the_duel_pool() {
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
            "deep-embrace",
            "exile",
            "insomnia"
        ]
    );
}
