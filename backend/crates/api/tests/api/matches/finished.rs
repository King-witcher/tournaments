//! O aviso do fim do torneio para quem jogou: quem recebe, o que diz e a faixa
//! de empate. As fases ligadas e a edição que encerra ficam em `stages`, o
//! canal do Discord em `discord::delivery`.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::realtime::{clear, notices, outbox};
use crate::teams::{started, tdm};

/// Os avisos do fim no outbox, `(código, destinatários, parâmetros)`, por
/// destinatário.
pub async fn finish_notices(db: &PgPool) -> Vec<(String, Vec<i64>, Value)> {
    let mut list: Vec<_> = notices(&outbox(db).await)
        .into_iter()
        .filter(|(code, ..)| {
            ["tournament.won", "tournament.team_won", "tournament.placed"].contains(&code.as_str())
        })
        .collect();
    list.sort_by_key(|(_, to, _)| to.clone());
    list
}

pub fn won(who: &User) -> (String, Vec<i64>, Value) {
    (
        "tournament.won".to_owned(),
        vec![who.id],
        json!({ "tournament": "Copa de Outubro" }),
    )
}

pub fn placed(who: &User, champion: &str, position: &str) -> (String, Vec<i64>, Value) {
    (
        "tournament.placed".to_owned(),
        vec![who.id],
        json!({ "tournament": "Copa de Outubro", "champion": champion, "position": position }),
    )
}

#[tokio::test]
async fn each_player_hears_how_it_ended_and_the_eliminated_get_a_range() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    favorite_wins(&app, &p, "wb-1-1", 1).await;
    favorite_wins(&app, &p, "wb-1-2", 1).await;
    assert_eq!(finish_notices(&db).await, []);
    clear(&db).await;

    favorite_wins(&app, &p, "wb-2-1", 1).await;
    let b = bracket(&app, &p[0]).await;
    assert_eq!(b["status"], "finished");
    // Ana organizou e não jogou: nada para ela.
    assert_eq!(
        finish_notices(&db).await,
        [
            won(&p[0]),
            placed(&p[1], "P1", "2"),
            placed(&p[2], "P1", "3–4"),
            placed(&p[3], "P1", "3–4"),
        ]
    );
    let events = outbox(&db).await;
    let notice = events
        .iter()
        .find(|e| e["notice"]["code"] == "tournament.won")
        .unwrap();
    assert_eq!(notice["notice"]["path"], "/quake-br/copa");
}

#[tokio::test]
async fn each_player_of_a_team_hears_with_the_team_name() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let teams = started(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let uri = format!("{COPA}/teams/{}", teams[0]);
    let (status, body) = patch(&app, &uri, &ana, json!({ "name": "Os Fraggers" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // P2 saiu da comunidade: o time segue, sem o aviso para ele.
    let member = format!("/tenants/quake-br/members/{}", p[1].id);
    let (status, body) = delete(&app, &member, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    clear(&db).await;

    let (status, body) = set_result(&app, &ana, "wb-1-1", walkover("a"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let mut users = vec![p[2].id, p[3].id];
    users.sort_unstable();
    assert_eq!(
        finish_notices(&db).await,
        [
            (
                "tournament.team_won".to_owned(),
                vec![p[0].id],
                json!({ "tournament": "Copa de Outubro", "team": "Os Fraggers" })
            ),
            (
                "tournament.placed".to_owned(),
                users,
                json!({ "tournament": "Copa de Outubro", "champion": "Os Fraggers", "position": "2" })
            ),
        ]
    );
}

#[tokio::test]
async fn who_withdrew_hears_nothing_when_the_withdrawal_ends_the_tournament() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    clear(&db).await;
    let team = team_ids(&app, &ana, "copa", &[&p[1]]).await[0];
    let uri = format!("{COPA}/teams/{team}/withdrawal");
    let (status, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(bracket(&app, &p[0]).await["status"], "finished");
    assert_eq!(finish_notices(&db).await, [won(&p[0])]);
}
