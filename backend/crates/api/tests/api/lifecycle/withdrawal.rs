//! Desistência num torneio em andamento (marcada pelo organizador ou porque o
//! inscrito saiu da comunidade) e o W.O. automático nas partidas prontas com
//! quem desistiu, agora e depois.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{
    COPA, bracket, copa, find, kinds, play, players, report, score, single, start,
};

async fn withdraw(app: &Router, by: &User, who: &User) -> (StatusCode, Value) {
    let uri = format!("{COPA}/participants/{}/withdrawal", who.id);
    post_empty(app, &uri, by).await
}

async fn ok(app: &Router, by: &User, who: &User) {
    let (status, body) = withdraw(app, by, who).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

async fn detail(app: &Router, who: &User, key: &str) -> Value {
    let (status, body) = get(app, &format!("{COPA}/matches/{key}"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// O inscrito `who` na lista de inscritos.
async fn participant(app: &Router, viewer: &User, who: &User) -> Value {
    let (_, list) = get(app, &format!("{COPA}/participants"), viewer).await;
    list.as_array()
        .unwrap()
        .iter()
        .find(|p| p["user_id"] == who.id)
        .cloned()
        .unwrap()
}

/// O W.O. automático de `key`: o histórico e o vencedor (`user_id`).
async fn auto_walkover(app: &Router, viewer: &User, key: &str) -> (Vec<String>, Value) {
    let d = detail(app, viewer, key).await;
    let result = &d["summary"]["result"];
    assert_eq!(
        (
            &result["walkover"],
            &result["confirmed_via"],
            &result["status"]
        ),
        (&json!(true), &json!("withdrawal"), &json!("confirmed")),
        "{key}: {d}"
    );
    let last = d["history"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["actor"], Value::Null, "{key}");
    let history = kinds(&d).into_iter().map(str::to_owned).collect();
    (history, last["outcome"]["winner"].clone())
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_organizer_marks_a_withdrawal_and_the_ready_match_is_a_walkover(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;

    assert_eq!(
        withdraw(&app, &p[0], &p[3]).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    ok(&app, &ana, &p[3]).await;
    // De novo não muda nada.
    ok(&app, &ana, &p[3]).await;

    let (history, winner) = auto_walkover(&app, &ana, "wb-1-1").await;
    assert_eq!(
        (history, winner),
        (vec!["auto_walkover".to_owned()], json!(p[0].id))
    );
    // Desistiu do torneio, mas continua na comunidade e na chave.
    let mine = participant(&app, &ana, &p[3]).await;
    assert_eq!(
        (&mine["withdrawn"], &mine["left"]),
        (&json!(true), &json!(false))
    );
    assert_eq!(
        get(&app, "/tenants/quake-br", &p[3]).await.0,
        StatusCode::OK
    );
    let b = bracket(&app, &p[3]).await;
    let entrant = b["entrants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["user_id"] == p[3].id);
    assert_eq!(entrant.unwrap()["withdrawn"], true);

    // O resto segue normal.
    play(&app, &p[1], &p[2], "wb-1-2", 2, 0).await;
    let b = bracket(&app, &ana).await;
    assert_eq!(find(&b, "wb-2-1")["state"], "ready");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_pending_report_gives_way_to_the_walkover(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let (status, _) = report(&app, &p[0], "wb-1-1", score(2, 0)).await;
    assert_eq!(status, StatusCode::OK);

    ok(&app, &ana, &p[0]).await;
    let (history, winner) = auto_walkover(&app, &ana, "wb-1-1").await;
    assert_eq!(history, ["reported", "auto_walkover"]);
    assert_eq!(winner, p[3].id);
    let d = detail(&app, &ana, "wb-1-1").await;
    assert_eq!(d["summary"]["result"]["reported_side"], Value::Null);
}

#[sqlx::test(migrations = "../../migrations")]
async fn leaving_mid_tournament_walks_over_now_and_later(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // Eliminação dupla: quem perde na chave de cima cai na de baixo.
    start(&app, &ana, copa(json!({})), &[&p[0], &p[1], &p[2], &p[3]]).await;

    let (status, body) = delete(&app, "/tenants/quake-br/me", &p[3]).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let (_, winner) = auto_walkover(&app, &ana, "wb-1-1").await;
    assert_eq!(winner, p[0].id);
    let mine = participant(&app, &ana, &p[3]).await;
    assert_eq!(
        (&mine["withdrawn"], &mine["left"]),
        (&json!(true), &json!(true))
    );
    // Na chave de baixo ele espera o perdedor de wb-1-2.
    assert_eq!(
        find(&bracket(&app, &ana).await, "lb-1-1")["state"],
        "pending"
    );

    // Quando a partida fica pronta, vira W.O. também.
    play(&app, &p[1], &p[2], "wb-1-2", 2, 1).await;
    let (history, winner) = auto_walkover(&app, &ana, "lb-1-1").await;
    assert_eq!(
        (history, winner),
        (vec!["auto_walkover".to_owned()], json!(p[2].id))
    );
    let b = bracket(&app, &ana).await;
    assert_eq!(find(&b, "wb-2-1")["state"], "ready");
    assert_eq!(b["status"], "in_progress");
}

#[sqlx::test(migrations = "../../migrations")]
async fn with_both_sides_out_the_better_seed_advances(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(&app, &ana, copa(json!({})), &[&p[0], &p[1], &p[2], &p[3]]).await;

    ok(&app, &ana, &p[3]).await;
    ok(&app, &ana, &p[2]).await;
    // P4 e P3 caem na chave de baixo, os dois fora: avança o seed 3.
    let (_, winner) = auto_walkover(&app, &ana, "lb-1-1").await;
    assert_eq!(winner, p[2].id);
    // Na próxima, ele leva W.O. de quem perder a final da chave de cima.
    play(&app, &p[0], &p[1], "wb-2-1", 2, 0).await;
    let (_, winner) = auto_walkover(&app, &ana, "lb-2-1").await;
    assert_eq!(winner, p[1].id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn walkovers_follow_confirmations_by_deadline(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    play(&app, &p[0], &p[3], "wb-1-1", 2, 0).await;
    // P1 desiste já classificado para a final, que ainda espera wb-1-2.
    ok(&app, &ana, &p[0]).await;
    assert_eq!(
        find(&bracket(&app, &ana).await, "wb-2-1")["state"],
        "pending"
    );

    let (status, _) = report(&app, &p[1], "wb-1-2", score(2, 0)).await;
    assert_eq!(status, StatusCode::OK);
    sqlx::query!(
        "update match_results set reported_at = reported_at - interval '1 hour',
                                  confirm_by = confirm_by - interval '1 hour'
         where status = 'pending'"
    )
    .execute(&db)
    .await
    .unwrap();

    // Ler a chave confirma pelo tempo, e a final vira W.O.: acabou.
    let b = bracket(&app, &ana).await;
    assert_eq!(find(&b, "wb-1-2")["result"]["confirmed_via"], "deadline");
    assert_eq!(b["status"], "finished");
    let (_, winner) = auto_walkover(&app, &ana, "wb-2-1").await;
    assert_eq!(winner, p[1].id);
    assert_eq!(b["standings"][0]["user_id"], p[1].id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_bracket_entrants_of_tournaments_in_progress_withdraw(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    // Com check-in: P3 se inscreve e não faz, fica fora da chave.
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&p[0], &p[1], &p[2]]).await;
    assert_eq!(
        withdraw(&app, &ana, &p[0]).await,
        (StatusCode::CONFLICT, err("tournament.not_in_progress"))
    );
    let (status, _) = post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    for who in [&p[0], &p[1]] {
        let (status, _) = post_empty(&app, &format!("{COPA}/check-in"), who).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    let (status, _) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(
        withdraw(&app, &ana, &p[2]).await,
        (StatusCode::CONFLICT, err("participant.not_in_bracket"))
    );
    assert_eq!(
        withdraw(&app, &ana, &bia).await,
        (StatusCode::NOT_FOUND, err("participant.not_found"))
    );
    let missing = format!(
        "/tenants/quake-br/tournaments/nada/participants/{}/withdrawal",
        bia.id
    );
    assert_eq!(
        post_empty(&app, &missing, &ana).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );

    // Com dois na chave, os W.O. levam o torneio até o fim.
    ok(&app, &ana, &p[1]).await;
    let b = bracket(&app, &ana).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(
        withdraw(&app, &ana, &p[0]).await,
        (StatusCode::CONFLICT, err("tournament.not_in_progress"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn undoing_the_start_drops_the_withdrawn(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1], &p[2]]).await;
    ok(&app, &ana, &p[2]).await;

    let (status, body) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, list) = get(&app, &format!("{COPA}/participants"), &ana).await;
    let ids: Vec<_> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["user_id"].clone())
        .collect();
    assert_eq!(ids, [json!(p[0].id), json!(p[1].id)]);
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, list) = get(&app, &format!("{COPA}/participants"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .all(|p| p["withdrawn"] == false)
    );
}
