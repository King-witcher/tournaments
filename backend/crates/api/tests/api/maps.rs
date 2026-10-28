use axum::http::{Method, StatusCode};
use sqlx::PgPool;

use crate::common::{app, call};

#[sqlx::test(migrations = "../../migrations")]
async fn maps_list_the_duel_pool_first(db: PgPool) {
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
