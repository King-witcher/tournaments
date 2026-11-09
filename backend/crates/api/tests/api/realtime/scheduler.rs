//! O agendador: confirma os mapas vencidos, numa instância só, e avisa a
//! organização da contestação que ninguém resolveu.

use api::realtime::{self, Tick};
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::json;

use super::*;
use crate::common::*;
use crate::matches::{accept, age, contest, copa, game, players, report, single, start};

#[tokio::test]
async fn the_organization_hears_of_a_contest_only_if_nobody_resolves_it() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let k = "wb-1-1";
    let (status, body) = report(&app, &p[1], k, 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = contest(&app, &p[0], k, 1, "foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    clear(&db).await;

    // A organização já vê, mas ainda não é avisada.
    let (_, pending) = get(&app, "/me/organizing", &ana).await;
    assert_eq!(pending["contested"][0]["key"], k);
    let early = Utc::now() + Duration::seconds(30);
    let tick = realtime::tick(&state, early).await.unwrap().unwrap();
    assert_eq!(tick.contest_notices, 0);
    assert_eq!(notices(&outbox(&db).await), []);

    // Passado o tempo, o aviso sai uma vez só.
    let later = Utc::now() + Duration::seconds(61);
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.contest_notices, 1);
    assert_eq!(
        notices(&outbox(&db).await),
        [(
            "match.game_contested".to_owned(),
            ids(&[&ana, &olga]),
            json!({ "tournament": "Copa de Outubro", "match": k, "game": 1, "player": "P1" })
        )]
    );
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.contest_notices, 0);

    // Resolvida antes de vencer, não incomoda ninguém.
    let (status, body) = report(&app, &p[1], k, 2, game("b", "vestibule-of-exile")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = contest(&app, &p[0], k, 2, "foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let right = json!({ "map": "vestibule-of-exile", "a_frags": 10, "b_frags": 5 });
    let (status, body) = accept(&app, &p[1], k, 2, right).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let later = Utc::now() + Duration::seconds(61);
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.contest_notices, 0);
}

#[tokio::test]
async fn the_captain_who_organizes_is_not_told_twice() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let p = players(&app, &db, &ana, 1).await;
    start(&app, &ana, copa(single(3, 3)), &[&olga, &p[0]]).await;
    let (status, body) = report(&app, &olga, "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    clear(&db).await;
    let (status, body) = contest(&app, &p[0], "wb-1-1", 1, "foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let found = notices(&outbox(&db).await);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].0.as_str(), &found[0].1),
        ("match.contest_received", &vec![olga.id])
    );

    clear(&db).await;
    let later = Utc::now() + Duration::seconds(61);
    realtime::tick(&state, later).await.unwrap().unwrap();
    let found = notices(&outbox(&db).await);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].0.as_str(), &found[0].1),
        ("match.game_contested", &vec![ana.id])
    );
}

#[tokio::test]
async fn confirms_due_maps_and_moves_the_bracket() {
    let (db, _lease) = crate::db::test_db().await;
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

    // Enviadas logo no início: o prazo é a duração da partida (15 minutos)
    // mais a janela (10).
    age(&db, 26).await;
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

#[tokio::test]
async fn one_pass_at_a_time() {
    let (db, _lease) = crate::db::test_db().await;
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
