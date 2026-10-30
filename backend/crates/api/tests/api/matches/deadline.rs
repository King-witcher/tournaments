//! O prazo de confirmação: a janela de contestação depois do envio ou depois
//! da duração prevista da partida (15 minutos), contada de quando ela foi
//! liberada, o que vier por último. O envio no começo da partida está em
//! `results` (`unanswered_maps_confirm_when_the_deadline_ends`).

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;

/// Segundos entre o envio e o prazo de cada partida, por série e número.
async fn windows(db: &PgPool) -> Vec<i32> {
    sqlx::query_scalar!(
        r#"select extract(epoch from confirm_by - reported_at)::integer as "seconds!"
           from match_games order by match_key, number"#
    )
    .fetch_all(db)
    .await
    .unwrap()
}

/// O torneio começou `minutes` minutos antes.
async fn started_ago(db: &PgPool, minutes: i32) {
    sqlx::query!(
        "update tournaments set started_at = started_at - make_interval(mins => $1)",
        minutes
    )
    .execute(db)
    .await
    .unwrap();
}

#[tokio::test]
async fn after_the_expected_duration_the_window_counts_from_the_report() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    started_ago(&db, 30).await;

    // A partida 1, liberada há 30 minutos: só a janela, desde o envio.
    let (status, body) = report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(windows(&db).await, [10 * 60]);
    // O adversário confirma na hora, como sempre.
    let (status, body) = confirm(&app, &p[1], "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["games"][0]["confirmed_via"], "opponent");

    // A 2, liberada agora (no envio da 1): a duração dela e a janela.
    let (status, body) = report(&app, &p[0], "wb-1-1", 2, game("b", "exile")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(deadlines(&db).await[1], 25 * 60);
}

#[tokio::test]
async fn with_the_veto_the_game_is_released_when_it_ends() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    let veto = json!({ "veto": { "map_mode": "random", "champion_phase": false } });
    start(&app, &ana, copa(with(single(3, 3), veto)), &[&p[0], &p[1]]).await;
    // O veto começa uma hora depois do início do torneio.
    started_ago(&db, 60).await;
    let ready = format!("{COPA}/matches/wb-1-1/veto/ready");
    let (status, body) = post_empty(&app, &ready, &p[0]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, d) = post_empty(&app, &ready, &p[1]).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    let drawn = |d: &serde_json::Value, i: usize| {
        d["veto"]["games"][i]["map"]["slug"]
            .as_str()
            .unwrap_or_else(|| panic!("sem o mapa sorteado: {d}"))
            .to_owned()
    };

    // O mapa sorteado no começo do veto: a partida 1 é liberada aí, não no
    // início do torneio.
    let (status, d) = report(&app, &p[0], "wb-1-1", 1, game("a", &drawn(&d, 0))).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    let from_veto = sqlx::query_scalar!(
        r#"select extract(epoch from g.confirm_by - e.created_at)::integer as "seconds!"
           from match_games g
           join match_veto_events e on e.tournament_id = g.tournament_id and e.kind = 'begin'
           where g.number = 1"#
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(from_veto, 25 * 60);

    // A 2 é sorteada no envio da 1 e liberada aí.
    let (status, d) = report(&app, &p[1], "wb-1-1", 2, game("b", &drawn(&d, 1))).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!(deadlines(&db).await[1], 25 * 60);
}
