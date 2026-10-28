//! O agendador: confirma os mapas vencidos, abre o check-in na hora e avisa
//! a hora de iniciar uma vez, numa instância só.

use api::realtime::{self, Tick};
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{age, copa, game, players, report, single, start};

async fn status_of(db: &PgPool, slug: &str) -> String {
    sqlx::query_scalar("select status from tournaments where slug = $1")
        .bind(slug)
        .fetch_one(db)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn confirms_due_maps_and_moves_the_bracket(db: PgPool) {
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    for (who, key) in [(&p[0], "wb-1-1"), (&p[1], "wb-1-2")] {
        let (status, body) = report(&app, who, key, 1, game("a", "awoken")).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let tick = realtime::tick(&state, Utc::now()).await.unwrap();
    assert_eq!(tick, Some(Tick::default()));

    age(&db, 11).await;
    clear(&db).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.confirmed, 2);
    // As semifinais fecharam pelo prazo e a final ficou pronta.
    let events = outbox(&db).await;
    assert!(events.contains(&tournament("copa", Some("wb-1-1"))));
    assert!(events.contains(&tournament("copa", Some("wb-2-1"))));
    let ready: Vec<_> = notices(&events)
        .into_iter()
        .map(|(code, to, _)| (code, to))
        .collect();
    assert_eq!(
        ready,
        [
            ("match.ready".to_owned(), vec![p[0].id]),
            ("match.ready".to_owned(), vec![p[1].id]),
        ]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn opens_the_check_in_on_time(db: PgPool) {
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    // Janela de 30 minutos, início em 40: abre daqui a 10.
    starts_in(&db, "copa", 40).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.opened, 0);
    assert_eq!(status_of(&db, "copa").await, "registration");

    clear(&db).await;
    let later = Utc::now() + Duration::minutes(11);
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.opened, 1);
    assert_eq!(status_of(&db, "copa").await, "check_in");
    let found = notices(&outbox(&db).await);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].0.as_str(), &found[0].1),
        ("check_in.opened", &vec![bia.id])
    );

    // Reaberta a inscrição depois de a janela abrir, a abertura automática
    // fica suspensa, como ao ler.
    let uri = "/tenants/quake-br/tournaments/copa/registration/reopen";
    let (status, _) = post_empty(&app, uri, &ana).await;
    assert_eq!(status, StatusCode::OK);
    starts_in(&db, "copa", 20).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.opened, 0);
    assert_eq!(status_of(&db, "copa").await, "registration");
}

#[sqlx::test(migrations = "../../migrations")]
async fn reminds_the_start_once(db: PgPool) {
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    member(&app, &db, &ana, "Bia").await;
    let without_check_in = with(duel("copa"), json!({ "check_in_minutes": null }));
    create_tournament(&app, &ana, without_check_in.clone()).await;
    publish(&app, &ana, "copa").await;
    // Rascunho com a data passada não é lembrado.
    create_tournament(
        &app,
        &ana,
        with(without_check_in, json!({ "slug": "rascunho" })),
    )
    .await;
    for slug in ["copa", "rascunho"] {
        starts_in(&db, slug, -1).await;
    }

    clear(&db).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.reminded, 1);
    let found = notices(&outbox(&db).await);
    assert_eq!(
        found,
        [(
            "tournament.start_due".to_owned(),
            ids(&[&ana, &olga]),
            json!({ "tournament": "Copa de Outubro" })
        )]
    );
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.reminded, 0);

    // Adiado para depois do aviso, avisa de novo quando a nova hora chega.
    starts_in(&db, "copa", 30).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.reminded, 0);
    let later = Utc::now() + Duration::minutes(31);
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.reminded, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn one_pass_at_a_time(db: PgPool) {
    let state = state(db.clone());
    // Outra instância numa passada (o mesmo advisory lock).
    let mut other = db.begin().await.unwrap();
    let locked: bool = sqlx::query_scalar("select pg_try_advisory_xact_lock($1)")
        .bind(0x746f_7572_6e65_7973_i64)
        .fetch_one(&mut *other)
        .await
        .unwrap();
    assert!(locked);
    assert_eq!(realtime::tick(&state, Utc::now()).await.unwrap(), None);
    other.rollback().await.unwrap();
    assert!(realtime::tick(&state, Utc::now()).await.unwrap().is_some());
}
