//! Seeding antes de iniciar: ver os elegíveis na ordem, sortear e ordenar à mão.

use axum::{Router, http::StatusCode};
use engine::{core::ParticipantId, seeding::random_order};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::COPA;
use crate::common::*;

/// `(apelido, seed)` na ordem do seeding: no duelo, cada time é de um
/// jogador, sem nome.
fn seeds(list: &Value) -> Vec<(&str, Option<i64>)> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|t| {
            assert!(t["name"].is_null(), "{t}");
            assert_eq!(t["members"].as_array().unwrap().len(), 1, "{t}");
            (
                t["members"][0]["nickname"].as_str().unwrap(),
                t["seed"].as_i64(),
            )
        })
        .collect()
}

async fn seeding(app: &Router, who: &User) -> Value {
    let (status, order) = get(app, &format!("{COPA}/seeding"), who).await;
    assert_eq!(status, StatusCode::OK, "{order}");
    order
}

/// Copa com Ana, Bia, Caio e Duda inscritos, em check-in, e check-in de
/// Ana, Bia e Duda.
async fn copa_in_check_in(db: &PgPool) -> (Router, User, User, User, User) {
    let (app, ana, bia) = quake_br(db).await;
    let caio = member(&app, db, &ana, "Caio").await;
    let duda = member(&app, db, &ana, "Duda").await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia, &caio, &duda]).await;
    post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    for who in [&ana, &bia, &duda] {
        let (status, _) = post_empty(&app, &format!("{COPA}/check-in"), who).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    (app, ana, bia, caio, duda)
}

#[tokio::test]
async fn manual_order_is_kept() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia, caio, duda) = copa_in_check_in(&db).await;
    // Sem seed ainda: os elegíveis na ordem de inscrição, sem Caio.
    assert_eq!(
        seeds(&seeding(&app, &ana).await),
        [("Ana", None), ("Bia", None), ("Duda", None)]
    );

    let uri = format!("{COPA}/seeding");
    let (status, order) = seed(&app, &ana, "copa", &[&duda, &ana, &bia]).await;
    assert_eq!(status, StatusCode::OK, "{order}");
    let expected = [("Duda", Some(1)), ("Ana", Some(2)), ("Bia", Some(3))];
    assert_eq!(seeds(&order), expected);
    assert_eq!(seeds(&seeding(&app, &ana).await), expected);

    // Precisa ser exatamente os times elegíveis, cada um uma vez.
    let t = team_ids(&app, &ana, "copa", &[&duda, &ana, &bia, &caio]).await;
    for wrong in [
        json!([t[0], t[1]]),
        json!([t[0], t[1], t[2], t[3]]),
        json!([t[0], t[1], t[1]]),
        json!([t[0], t[1], t[3]]),
    ] {
        assert_eq!(
            put(&app, &uri, &ana, json!({ "team_ids": wrong })).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("seeding.order_invalid")
            )
        );
    }

    // Quem faz check-in depois entra no fim, sem seed.
    post_empty(&app, &format!("{COPA}/check-in"), &caio).await;
    assert_eq!(
        seeds(&seeding(&app, &ana).await),
        [
            ("Duda", Some(1)),
            ("Ana", Some(2)),
            ("Bia", Some(3)),
            ("Caio", None)
        ]
    );
}

#[tokio::test]
async fn draw_is_saved_and_reproducible() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, ..) = copa_in_check_in(&db).await;
    let (status, drawn) = post_empty(&app, &format!("{COPA}/seeding/shuffle"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{drawn}");
    let order: Vec<(&str, Option<i64>)> = seeds(&drawn);
    assert_eq!(
        order.iter().map(|(_, seed)| *seed).collect::<Vec<_>>(),
        [Some(1), Some(2), Some(3)]
    );
    assert_eq!(seeds(&seeding(&app, &ana).await), order);

    // A seed gravada refaz a mesma ordem no engine.
    let seed = sqlx::query_scalar!("select seeding_seed from tournaments")
        .fetch_one(&db)
        .await
        .unwrap()
        .unwrap();
    let eligible = sqlx::query_scalar!(
        "select team_id as \"team_id!\" from participants where checked_in_at is not null order by team_id"
    )
    .fetch_all(&db)
    .await
    .unwrap();
    let ids: Vec<ParticipantId> = eligible.into_iter().map(ParticipantId).collect();
    let by_seed = sqlx::query_scalar!("select id from teams where seed is not null order by seed")
        .fetch_all(&db)
        .await
        .unwrap();
    let replayed: Vec<i64> = random_order(&ids, seed as u64)
        .into_iter()
        .map(|p| p.0)
        .collect();
    assert_eq!(replayed, by_seed);
    assert_eq!(drawn.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn players_do_not_see_or_change_the_seeding() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, bia, ..) = copa_in_check_in(&db).await;
    let forbidden = (StatusCode::FORBIDDEN, err("tenant.forbidden"));
    assert_eq!(get(&app, &format!("{COPA}/seeding"), &bia).await, forbidden);
    assert_eq!(
        put(
            &app,
            &format!("{COPA}/seeding"),
            &bia,
            json!({ "team_ids": [] })
        )
        .await,
        forbidden
    );
    assert_eq!(
        post_empty(&app, &format!("{COPA}/seeding/shuffle"), &bia).await,
        forbidden
    );
}
