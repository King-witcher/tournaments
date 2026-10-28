//! Os avisos e quem recebe cada um: check-in aberto (inscritos), partida
//! pronta (os dois lados, também ao iniciar, nunca na que vira W.O.), mapa
//! enviado (o adversário, com o prazo) e contestado (organizers e owner). A
//! hora de iniciar sai do agendador (`scheduler`).

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{COPA, confirm, contest, copa, game, players, report, single, start};

#[sqlx::test(migrations = "../../migrations")]
async fn check_in_opened_goes_to_the_registered(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let cris = member(&app, &db, &ana, "Cris").await;
    member(&app, &db, &ana, "Fora").await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia, &cris]).await;
    clear(&db).await;
    let (status, body) = post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let events = outbox(&db).await;
    assert_eq!(
        notices(&events),
        [(
            "check_in.opened".to_owned(),
            ids(&[&bia, &cris]),
            json!({ "tournament": "Copa de Outubro" })
        )]
    );
    assert_eq!(events.last().unwrap()["notice"]["path"], "/quake-br/copa");
}

#[sqlx::test(migrations = "../../migrations")]
async fn starting_tells_both_sides_of_each_ready_match(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let ready = |to: &User, key: &str, opponent: &str| {
        (
            "match.ready".to_owned(),
            vec![to.id],
            json!({ "tournament": "Copa de Outubro", "match": key, "opponent": opponent }),
        )
    };
    let events = outbox(&db).await;
    assert_eq!(
        notices(&events),
        [
            ready(&p[0], "wb-1-1", "P4"),
            ready(&p[3], "wb-1-1", "P1"),
            ready(&p[1], "wb-1-2", "P3"),
            ready(&p[2], "wb-1-2", "P2"),
        ]
    );
    let path = events.iter().find(|e| e["kind"] == "notice").unwrap()["notice"]["path"].clone();
    assert_eq!(path, "/quake-br/copa/matches/wb-1-1");
}

#[sqlx::test(migrations = "../../migrations")]
async fn reported_and_contested_maps(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    clear(&db).await;

    let (status, body) = report(&app, &p[1], "wb-1-1", 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let found = notices(&outbox(&db).await);
    assert_eq!(found.len(), 1);
    let (code, to, params) = &found[0];
    assert_eq!((code.as_str(), to), ("match.game_reported", &vec![p[0].id]));
    assert_eq!(params["opponent"], "P2");
    assert_eq!(
        (params["match"].clone(), params["game"].clone()),
        (json!("wb-1-1"), json!(1))
    );
    // O prazo é o do mapa, que a partida também mostra.
    let detail = get(&app, &format!("{COPA}/matches/wb-1-1"), &p[0]).await.1;
    assert_eq!(params["confirm_by"], detail["games"][0]["confirm_by"]);

    clear(&db).await;
    let (status, body) = contest(&app, &p[0], "wb-1-1", 1, "foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        notices(&outbox(&db).await),
        [(
            "match.game_contested".to_owned(),
            ids(&[&ana, &olga]),
            json!({ "tournament": "Copa de Outubro", "match": "wb-1-1", "game": 1, "player": "P1" })
        )]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_next_match_is_ready_when_both_sides_arrive(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let ok = |(status, body): (StatusCode, serde_json::Value)| {
        assert_eq!(status, StatusCode::OK, "{body}");
    };

    ok(report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await);
    clear(&db).await;
    ok(confirm(&app, &p[3], "wb-1-1", 1).await);
    // A final ainda espera o outro lado.
    assert_eq!(notices(&outbox(&db).await), []);

    ok(report(&app, &p[1], "wb-1-2", 1, game("a", "awoken")).await);
    clear(&db).await;
    ok(confirm(&app, &p[2], "wb-1-2", 1).await);
    let events = outbox(&db).await;
    let found: Vec<_> = notices(&events)
        .into_iter()
        .map(|(code, to, params)| (code, to, params["opponent"].clone()))
        .collect();
    assert_eq!(
        found,
        [
            ("match.ready".to_owned(), vec![p[0].id], json!("P2")),
            ("match.ready".to_owned(), vec![p[1].id], json!("P1")),
        ]
    );
    // A semifinal e a final mudaram; "Seus torneios" de quem jogou nelas.
    assert!(events.contains(&tournament("copa", Some("wb-1-2"))));
    assert!(events.contains(&tournament("copa", Some("wb-2-1"))));
    assert!(events.contains(&me(&ids(&[&p[0], &p[1], &p[2]]))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn no_ready_notice_for_a_walkover(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let ok = |(status, body): (StatusCode, serde_json::Value)| {
        assert_eq!(status, StatusCode::OK, "{body}");
    };
    ok(report(&app, &p[1], "wb-1-2", 1, game("a", "awoken")).await);
    ok(confirm(&app, &p[2], "wb-1-2", 1).await);
    // P2 venceu a semifinal e desistiu: a final vira W.O. assim que fica pronta.
    let uri = format!("{COPA}/participants/{}/withdrawal", p[1].id);
    let (status, _) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    ok(report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await);
    clear(&db).await;
    ok(confirm(&app, &p[3], "wb-1-1", 1).await);
    let events = outbox(&db).await;
    assert_eq!(notices(&events), []);
    assert!(events.contains(&tournament("copa", Some("wb-2-1"))));
    // E o torneio encerrou: muda inteiro, para todos os inscritos.
    assert!(events.contains(&tournament("copa", None)));
    assert!(events.contains(&me(&ids(&[&p[0], &p[1], &p[2], &p[3]]))));
}
