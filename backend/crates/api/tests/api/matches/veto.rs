//! Veto de mapas e draft de campeões da série: "Pronto", os passos, o tempo
//! esgotado (sorteio), desfazer, a votação, a regra de não repetição, o
//! envio que segue o veto e o que apaga o veto.

use api::realtime;
use axum::{Router, http::StatusCode};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::realtime::{clear, notices, outbox};

/// Final única entre P1 (seed 1, lado a) e P2, em md`best_of`, com `veto`.
async fn final_with(
    db: &PgPool,
    best_of: u8,
    veto: Value,
    rules: Value,
) -> (Router, User, Vec<User>) {
    let (app, ana, _) = quake_br(db).await;
    let p = players(&app, db, &ana, 2).await;
    let body = copa(with(
        single(best_of, best_of),
        json!({ "veto": veto, "champion_rules": rules }),
    ));
    start(&app, &ana, body, &[&p[0], &p[1]]).await;
    (app, ana, p)
}

fn one_ban() -> Value {
    json!({ "bans_per_series": 1, "mirror": true, "same_team": false, "no_repeat": null })
}

fn uri(action: &str) -> String {
    format!("{COPA}/matches/wb-1-1/veto/{action}")
}

async fn ready(app: &Router, who: &User) -> (StatusCode, Value) {
    post_empty(app, &uri("ready"), who).await
}

async fn pick(app: &Router, who: &User, choice: Value) -> (StatusCode, Value) {
    post(app, &uri("steps"), who, choice).await
}

/// Um passo que precisa dar certo.
async fn ok_step(app: &Router, who: &User, choice: Value) -> Value {
    let (status, body) = pick(app, who, choice.clone()).await;
    assert_eq!(status, StatusCode::OK, "{choice}: {body}");
    body
}

fn map(slug: &str) -> Value {
    json!({ "map": slug })
}

fn champ(slug: &str) -> Value {
    json!({ "champion": slug })
}

fn turn(detail: &Value) -> (Value, Value, Value) {
    let t = &detail["veto"]["turn"];
    (t["action"].clone(), t["target"].clone(), t["sides"].clone())
}

/// Volta `seconds` segundos no relógio de todos os eventos de veto, em vez de
/// esperar o tempo do passo.
async fn age_veto(db: &PgPool, seconds: i32) {
    // O histórico não muda (regra do banco): o teste desliga a regra para
    // envelhecer as datas.
    for sql in [
        "alter table match_veto_events disable trigger match_veto_events_guard".to_owned(),
        format!(
            "update match_veto_events set created_at = created_at - interval '{seconds} seconds'"
        ),
        "alter table match_veto_events enable trigger match_veto_events_guard".to_owned(),
    ] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(db)
            .await
            .unwrap();
    }
}

fn veto_on() -> Value {
    json!({ "map_mode": "veto", "champion_phase": true })
}

#[tokio::test]
async fn veto_of_maps_and_champions_then_the_report_follows_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, p) = final_with(&db, 3, veto_on(), one_ban()).await;
    let d = detail(&app, &p[0], "wb-1-1").await;
    assert_eq!(d["veto"]["status"], "waiting_ready");
    assert_eq!(d["summary"]["task"], "ready");
    assert_eq!(d["summary"]["veto"], "waiting_ready");
    assert_eq!(
        report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await,
        (StatusCode::CONFLICT, err("match.veto_pending"))
    );

    clear(&db).await;
    let (status, body) = ready(&app, &p[0]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["ready_a"], true);
    assert_eq!(body["summary"]["task"], "await_opponent");
    assert_eq!(
        notices(&outbox(&db).await),
        [(
            "match.opponent_ready".to_owned(),
            vec![p[1].id],
            json!({ "tournament": "Copa de Outubro", "match": "wb-1-1", "opponent": "P1" })
        )]
    );
    // Repetir não muda nada.
    assert_eq!(ready(&app, &p[0]).await.0, StatusCode::OK);
    clear(&db).await;
    let (_, body) = ready(&app, &p[1]).await;
    assert_eq!(body["veto"]["status"], "in_progress");
    let started: Vec<_> = notices(&outbox(&db).await)
        .into_iter()
        .map(|(code, to, _)| (code, to))
        .collect();
    assert_eq!(
        started,
        [
            ("match.veto_started".to_owned(), vec![p[0].id]),
            ("match.veto_started".to_owned(), vec![p[1].id]),
        ]
    );
    assert_eq!(
        ready(&app, &p[1]).await,
        (StatusCode::CONFLICT, err("veto.already_begun"))
    );

    // O seed melhor (a) começa o veto de mapas.
    let d = detail(&app, &p[0], "wb-1-1").await;
    assert_eq!(turn(&d), (json!("ban"), json!("map"), json!(["a"])));
    assert!(d["veto"]["deadline"].is_string());
    assert_eq!(d["veto"]["options"].as_array().unwrap().len(), 7);
    assert_eq!(d["summary"]["task"], "veto");
    assert_eq!(
        pick(&app, &p[1], map("awoken")).await,
        (StatusCode::CONFLICT, err("veto.not_your_turn"))
    );
    assert_eq!(
        pick(&app, &p[0], map("tower-of-koth")).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
    ok_step(&app, &p[0], map("awoken")).await;
    assert_eq!(
        pick(&app, &p[1], map("awoken")).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
    for (who, slug) in [
        (&p[1], "blood-covenant"),
        (&p[0], "blood-run"),
        (&p[1], "corrupted-keep"),
        (&p[0], "molten-falls"),
    ] {
        ok_step(&app, who, map(slug)).await;
    }
    let d = ok_step(&app, &p[1], map("ruins-of-sarnath")).await;
    let games = &d["veto"]["games"];
    assert_eq!(games[0]["map"]["slug"], "blood-run");
    assert_eq!(games[0]["picked_by"], "a");
    assert_eq!(
        d["veto"]["remaining"],
        json!([{ "slug": "vale-of-pnath", "name": "Vale of Pnath" }])
    );
    // O draft da série inteira: o seed pior (b) bane primeiro.
    assert_eq!(turn(&d), (json!("ban"), json!("champion"), json!(["b"])));
    ok_step(&app, &p[1], champ("anarki")).await;
    ok_step(&app, &p[0], champ("athena")).await;
    // O ban vale para os dois, em todas as partidas.
    let d = detail(&app, &p[0], "wb-1-1").await;
    assert_eq!(d["veto"]["turn"]["pick_for"], 1);
    assert_eq!(
        pick(&app, &p[0], champ("anarki")).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
    let slot = &d["veto"]["picks"][0];
    assert_eq!(slot["game"], 1);
    assert!(
        slot["unavailable"]
            .as_array()
            .unwrap()
            .contains(&json!({ "slug": "anarki", "reason": "banned" }))
    );
    // Partida 1: a escolhe e b responde; na 2, o contrário; na 3, de novo a.
    for (who, game, slug) in [
        (&p[0], 1, "clutch"),
        (&p[1], 1, "clutch"),
        (&p[1], 2, "eisen"),
        (&p[0], 2, "galena"),
        (&p[0], 3, "keel"),
    ] {
        let d = detail(&app, who, "wb-1-1").await;
        assert_eq!(d["veto"]["turn"]["pick_for"], game, "{slug}");
        ok_step(&app, who, champ(slug)).await;
    }
    let d = ok_step(&app, &p[1], champ("nyx")).await;
    assert_eq!(d["veto"]["status"], "done");
    assert_eq!(d["veto"]["reportable"], true);
    assert_eq!(d["summary"]["task"], "report");
    let games = &d["veto"]["games"];
    assert_eq!(games.as_array().unwrap().len(), 3);
    // Espelho vale por padrão.
    assert_eq!(
        (&games[0]["pick_a"]["slug"], &games[0]["pick_b"]["slug"]),
        (&json!("clutch"), &json!("clutch"))
    );
    assert_eq!(games[2]["pick_b"]["slug"], "nyx");
    assert_eq!(d["veto"]["bans"].as_array().unwrap().len(), 2);
    assert_eq!(
        pick(&app, &p[1], champ("eisen")).await,
        (StatusCode::CONFLICT, err("veto.finished"))
    );

    // O envio segue o mapa do veto.
    assert_eq!(
        report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("match.map_mismatch"))
    );
    let (status, d) = report(&app, &p[0], "wb-1-1", 1, game("b", "blood-run")).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    // A partida 2 começou e não espera nada: o draft já foi feito.
    assert_eq!(d["veto"]["game"], 2);
    assert_eq!(d["veto"]["status"], "done");
    assert_eq!(d["veto"]["reportable"], true);
    assert_eq!(
        report(&app, &p[0], "wb-1-1", 2, game("a", "blood-run")).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("match.map_mismatch"))
    );
    // O histórico guarda os passos, com a partida de cada pick.
    let history = d["veto"]["history"].as_array().unwrap();
    let kinds: Vec<&str> = history
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert_eq!(&kinds[..3], ["ready", "ready", "begin"]);
    assert_eq!(kinds.last(), Some(&"game"));
    let last_pick = history.iter().rev().find(|e| e["kind"] == "pick").unwrap();
    assert_eq!(
        (&last_pick["game"], &last_pick["pick_game"]),
        (&json!(1), &json!(3))
    );
}

#[tokio::test]
async fn without_mirror_the_first_to_pick_keeps_the_champion() {
    let (db, _lease) = crate::db::test_db().await;
    let veto = json!({ "map_mode": "off", "champion_phase": true });
    let rules =
        json!({ "bans_per_series": 0, "mirror": false, "same_team": false, "no_repeat": null });
    let (app, _, p) = final_with(&db, 1, veto, rules).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[1]).await;
    ok_step(&app, &p[0], champ("anarki")).await;
    assert_eq!(
        pick(&app, &p[1], champ("anarki")).await,
        (StatusCode::CONFLICT, err("veto.champion_taken"))
    );
    let d = detail(&app, &p[1], "wb-1-1").await;
    assert!(
        d["veto"]["picks"][0]["unavailable"]
            .as_array()
            .unwrap()
            .contains(&json!({ "slug": "anarki", "reason": "opponent" }))
    );
    ok_step(&app, &p[1], champ("athena")).await;
}

#[tokio::test]
async fn the_organizer_sending_game_one_completes_the_draft_by_draw() {
    let (db, _lease) = crate::db::test_db().await;
    let veto = json!({ "map_mode": "off", "champion_phase": true });
    let (app, ana, p) = final_with(&db, 3, veto, one_ban()).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[1]).await;
    ok_step(&app, &p[1], champ("anarki")).await;
    let (status, d) = set_game(&app, &ana, "wb-1-1", 1, game("a", "awoken"), None).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    // O resto do draft saiu pelo sorteio, e a partida 2 começou.
    assert_eq!(d["veto"]["game"], 2, "{d}");
    assert_eq!(d["veto"]["bans"].as_array().unwrap().len(), 2);
    for g in d["veto"]["games"].as_array().unwrap() {
        assert!(g["pick_a"].is_object() && g["pick_b"].is_object(), "{g}");
    }
    let drawn = d["veto"]["history"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["timeout"] == true)
        .count();
    assert_eq!(drawn, 7);
}

#[tokio::test]
async fn too_few_champions_for_the_bans_do_not_begin() {
    let (db, _lease) = crate::db::test_db().await;
    let rules =
        json!({ "bans_per_series": 3, "mirror": true, "same_team": false, "no_repeat": null });
    let (app, _, p) = final_with(&db, 1, veto_on(), rules).await;
    // Depois de iniciar (a criação e o início conferem o pool).
    sqlx::query("update champions set active = slug in ('anarki', 'athena', 'clutch')")
        .execute(&db)
        .await
        .unwrap();
    ready(&app, &p[0]).await;
    assert_eq!(
        ready(&app, &p[1]).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "champions.pool_exhausted", "params": { "required": 7, "available": 3 } })
        )
    );
}

#[tokio::test]
async fn a_timeout_picks_at_random_and_the_organizer_undoes_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = final_with(&db, 3, veto_on(), one_ban()).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[1]).await;
    age_veto(&db, 31).await;
    // Ler aplica o passo vencido: o de A, sorteado.
    let d = detail(&app, &p[1], "wb-1-1").await;
    let last = d["veto"]["history"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(
        (&last["kind"], &last["side"], &last["timeout"]),
        (&json!("ban"), &json!("a"), &json!(true))
    );
    assert!(last["actor"].is_null());
    assert_eq!(turn(&d).2, json!(["b"]));
    assert_eq!(d["veto"]["map_steps"].as_array().unwrap().len(), 1);

    assert_eq!(
        post_empty(&app, &uri("undo"), &p[1]).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let (status, d) = post_empty(&app, &uri("undo"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!(turn(&d).2, json!(["a"]));
    assert_eq!(d["veto"]["map_steps"].as_array().unwrap().len(), 0);
    assert_eq!(
        post_empty(&app, &uri("undo"), &ana).await,
        (StatusCode::CONFLICT, err("veto.nothing_to_undo"))
    );
}

#[tokio::test]
async fn the_scheduler_also_applies_timeouts() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let p = players(&app, &db, &ana, 2).await;
    let body = copa(with(single(1, 1), json!({ "veto": veto_on() })));
    start(&app, &ana, body, &[&p[0], &p[1]]).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[1]).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.vetoed, 0);
    // Dois prazos vencidos em sequência: dois passos sorteados.
    age_veto(&db, 61).await;
    clear(&db).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.vetoed, 2);
    assert!(
        outbox(&db)
            .await
            .contains(&crate::realtime::tournament("copa", Some("wb-1-1")))
    );
    let d = detail(&app, &p[0], "wb-1-1").await;
    assert_eq!(d["veto"]["map_steps"].as_array().unwrap().len(), 2);
    assert_eq!(turn(&d).2, json!(["a"]));
}

#[tokio::test]
async fn a_player_asks_to_undo_and_the_opponent_answers() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, p) = final_with(&db, 3, veto_on(), one_ban()).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[1]).await;
    assert_eq!(
        post_empty(&app, &uri("undo-request"), &p[0]).await,
        (StatusCode::CONFLICT, err("veto.nothing_to_undo"))
    );
    ok_step(&app, &p[0], map("awoken")).await;
    let (status, d) = post_empty(&app, &uri("undo-request"), &p[0]).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!(d["veto"]["undo_request"], "a");
    assert_eq!(
        post_empty(&app, &uri("undo-request"), &p[1]).await,
        (StatusCode::CONFLICT, err("veto.undo_requested"))
    );
    let answer = uri("undo-request/answer");
    assert_eq!(
        post(&app, &answer, &p[0], json!({ "accept": true })).await,
        (StatusCode::FORBIDDEN, err("veto.own_request"))
    );
    let (_, d) = post(&app, &answer, &p[1], json!({ "accept": true })).await;
    assert!(d["veto"]["undo_request"].is_null());
    assert_eq!(turn(&d).2, json!(["a"]));
    assert_eq!(d["veto"]["map_steps"], json!([]));

    ok_step(&app, &p[0], map("ruins-of-sarnath")).await;
    post_empty(&app, &uri("undo-request"), &p[0]).await;
    let (_, d) = post(&app, &answer, &p[1], json!({ "accept": false })).await;
    assert!(d["veto"]["undo_request"].is_null());
    assert_eq!(d["veto"]["map_steps"].as_array().unwrap().len(), 1);
    assert_eq!(
        post(&app, &answer, &p[1], json!({ "accept": true })).await,
        (StatusCode::CONFLICT, err("veto.no_undo_request"))
    );
    // Um passo novo cancela o pedido.
    post_empty(&app, &uri("undo-request"), &p[0]).await;
    let d = ok_step(&app, &p[1], map("awoken")).await;
    assert!(d["veto"]["undo_request"].is_null());
}

#[tokio::test]
async fn the_organizer_forces_the_start() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = final_with(&db, 1, veto_on(), one_ban()).await;
    assert_eq!(
        post_empty(&app, &uri("begin"), &p[0]).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let (status, d) = post_empty(&app, &uri("begin"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!(d["veto"]["status"], "in_progress");
    let begin = &d["veto"]["history"][0];
    assert_eq!(
        (&begin["kind"], &begin["actor"]),
        (&json!("begin"), &json!("Ana"))
    );
    assert_eq!(
        post_empty(&app, &uri("begin"), &ana).await,
        (StatusCode::CONFLICT, err("veto.already_begun"))
    );
}

#[tokio::test]
async fn vote_mode_both_vote_and_the_map_comes_out() {
    let (db, _lease) = crate::db::test_db().await;
    let veto = json!({ "map_mode": "vote", "champion_phase": false });
    let (app, _, p) = final_with(&db, 3, veto, one_ban()).await;
    ready(&app, &p[0]).await;
    let (_, d) = ready(&app, &p[1]).await;
    assert_eq!(turn(&d), (json!("vote"), json!("map"), json!(["a", "b"])));
    let offered: Vec<String> = d["veto"]["games"][0]["offered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["slug"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(offered.len(), 3);
    ok_step(&app, &p[1], map(&offered[2])).await;
    let d = ok_step(&app, &p[0], map(&offered[2])).await;
    assert_eq!(d["veto"]["games"][0]["map"]["slug"], offered[2]);
    assert_eq!(d["veto"]["status"], "done");
    let (status, d) = report(&app, &p[0], "wb-1-1", 1, game("a", &offered[2])).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    // O jogo 2 vota entre mapas que não saíram.
    let second = &d["veto"]["games"][1]["offered"];
    assert!(
        !second
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["slug"] == offered[2])
    );
}

#[tokio::test]
async fn no_repeat_takes_the_used_champion_out() {
    let (db, _lease) = crate::db::test_db().await;
    let veto = json!({ "map_mode": "off", "champion_phase": true });
    let rules = json!({
        "bans_per_series": 0, "mirror": true, "same_team": false,
        "no_repeat": { "who": "player", "scope": "series" }
    });
    let (app, _, p) = final_with(&db, 3, veto, rules).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[1]).await;
    ok_step(&app, &p[0], champ("anarki")).await;
    ok_step(&app, &p[1], champ("athena")).await;
    // Partida 2: b escolhe primeiro e pode pegar o anarki de a.
    ok_step(&app, &p[1], champ("anarki")).await;
    let d = detail(&app, &p[0], "wb-1-1").await;
    assert!(
        d["veto"]["picks"][0]["unavailable"]
            .as_array()
            .unwrap()
            .contains(&json!({ "slug": "anarki", "reason": "repeat" }))
    );
    assert_eq!(
        pick(&app, &p[0], champ("anarki")).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("veto.champion_forbidden")
        )
    );
}

#[tokio::test]
async fn walkover_and_undoing_the_start_end_the_veto() {
    let (db, _lease) = crate::db::test_db().await;
    // Semifinais: o W.O. na wb-1-1 (P1 x P4) não encerra o torneio.
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = copa(with(single(3, 3), json!({ "veto": veto_on() })));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2], &p[3]]).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[3]).await;
    ok_step(&app, &p[0], map("awoken")).await;
    let (status, d) = set_result(&app, &ana, "wb-1-1", walkover("b"), None).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    // O veto fica no histórico, mas não anda mais.
    assert_eq!(d["veto"]["map_steps"].as_array().unwrap().len(), 1);
    assert!(d["veto"]["turn"].is_null());
    assert_eq!(
        pick(&app, &p[3], map("ruins-of-sarnath")).await,
        (StatusCode::CONFLICT, err("match.series_decided"))
    );
    // Desfazer o início apaga o veto.
    let count = || async {
        sqlx::query_scalar::<_, i64>("select count(*) from match_veto_events")
            .fetch_one(&db)
            .await
            .unwrap()
    };
    assert!(count().await > 0);
    let (status, body) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(count().await, 0);
}

#[tokio::test]
async fn without_veto_nothing_changes() {
    let (db, _lease) = crate::db::test_db().await;
    let veto = json!({ "map_mode": "off", "champion_phase": false });
    let (app, _, p) = final_with(&db, 1, veto, one_ban()).await;
    let d = detail(&app, &p[0], "wb-1-1").await;
    assert!(d["veto"].is_null());
    assert_eq!(d["summary"]["task"], "report");
    assert_eq!(
        ready(&app, &p[0]).await,
        (StatusCode::CONFLICT, err("veto.off"))
    );
    let (status, body) = report(&app, &p[0], "wb-1-1", 1, game("a", "lockbox")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn the_tournament_keeps_the_veto_settings() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let mut body = duel("copa");
    body.as_object_mut().unwrap().remove("veto");
    let created = create_tournament(&app, &ana, body).await;
    assert_eq!(
        created["veto"],
        json!({
            "map_mode": "veto",
            "map_pool": ["awoken", "blood-covenant", "blood-run", "corrupted-keep",
                         "molten-falls", "ruins-of-sarnath", "vale-of-pnath"],
            "champion_phase": true,
            "step_seconds": 30,
            "team_pick_seconds": 60
        })
    );
    let tournament = "/tenants/quake-br/tournaments/copa";
    let bad = |veto: Value, code: &str, params: Value| {
        let expected = if params.is_null() {
            err(code)
        } else {
            json!({ "error": code, "params": params })
        };
        (veto, expected)
    };
    for (veto, expected) in [
        bad(
            json!({ "map_pool": ["awoken", "q3dm17"] }),
            "tournament.map_unknown",
            Value::Null,
        ),
        bad(
            json!({ "map_pool": ["awoken", "exile", "insomnia", "lockbox"] }),
            "tournament.map_pool_too_small",
            json!({ "min": 5 }),
        ),
        bad(
            json!({ "map_pool": ["awoken", "exile", "awoken", "lockbox", "insomnia"] }),
            "tournament.map_pool_repeated",
            Value::Null,
        ),
        bad(
            json!({ "step_seconds": 5 }),
            "tournament.veto_seconds_invalid",
            json!({ "min": 10, "max": 300 }),
        ),
    ] {
        assert_eq!(
            patch(&app, tournament, &ana, json!({ "veto": veto })).await,
            (StatusCode::UNPROCESSABLE_ENTITY, expected)
        );
    }
    // Desligado não precisa de pool grande; votação com outro pool.
    let (status, edited) = patch(
        &app,
        tournament,
        &ana,
        json!({ "veto": { "map_mode": "vote", "map_pool": ["lockbox", "exile", "awoken", "insomnia", "blood-run"], "step_seconds": 45 } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{edited}");
    assert_eq!(edited["veto"]["map_mode"], "vote");
    assert_eq!(edited["veto"]["map_pool"][0], "lockbox");
    assert_eq!(edited["veto"]["step_seconds"], 45);
    assert_eq!(edited["veto"]["champion_phase"], true);
}

#[tokio::test]
async fn the_veto_settings_lock_once_started() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = final_with(&db, 1, veto_on(), one_ban()).await;
    assert_eq!(
        patch(&app, COPA, &ana, json!({ "veto": { "step_seconds": 60 } })).await,
        (StatusCode::CONFLICT, err("tournament.setup_locked"))
    );
}

#[tokio::test]
async fn a_series_in_progress_reserves_its_champions_until_it_ends() {
    // Todos contra todos, cada jogador sem repetir no torneio: as três séries
    // de P1 ficam prontas ao mesmo tempo.
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let rules = json!({
        "bans_per_series": 0, "mirror": true, "same_team": false,
        "no_repeat": { "who": "player", "scope": "tournament" }
    });
    let body = copa(with(
        table("round_robin", json!({ "best_of": 1 })),
        json!({ "veto": { "map_mode": "off", "champion_phase": true }, "champion_rules": rules }),
    ));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2], &p[3]]).await;
    let b = bracket(&app, &p[0]).await;
    let mut series: Vec<(String, &User)> = Vec::new();
    for key in keys_in(&b, "ready") {
        let (a, other) = sides(&b, find(&b, &key));
        let opponent = if a == json!(p[0].id) {
            other
        } else if other == json!(p[0].id) {
            a
        } else {
            continue;
        };
        let opponent = p.iter().find(|u| json!(u.id) == opponent).unwrap();
        series.push((key, opponent));
    }
    assert_eq!(series.len(), 3);
    let uri = |key: &str, action: &str| format!("{COPA}/matches/{key}/veto/{action}");
    for (key, opponent) in &series[..2] {
        assert_eq!(
            post_empty(&app, &uri(key, "ready"), &p[0]).await.0,
            StatusCode::OK
        );
        assert_eq!(
            post_empty(&app, &uri(key, "ready"), opponent).await.0,
            StatusCode::OK
        );
    }
    // P1, o seed melhor, escolhe primeiro: o anarki na primeira série.
    let (first, second) = (&series[0].0, &series[1].0);
    let (status, body) = post(&app, &uri(first, "steps"), &p[0], champ("anarki")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Na outra, ele está reservado: a primeira ainda pode ser jogada.
    let d = detail(&app, &p[0], second).await;
    assert!(
        d["veto"]["picks"][0]["unavailable"]
            .as_array()
            .unwrap()
            .contains(&json!({ "slug": "anarki", "reason": "repeat" })),
        "{d}"
    );
    assert_eq!(
        post(&app, &uri(second, "steps"), &p[0], champ("anarki")).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("veto.champion_forbidden")
        )
    );
    // A primeira acaba por W.O., sem a partida: a reserva sai.
    let side = if sides(&b, find(&b, first)).0 == json!(p[0].id) {
        "b"
    } else {
        "a"
    };
    let (status, body) = set_result(&app, &ana, first, walkover(side), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post(&app, &uri(second, "steps"), &p[0], champ("anarki")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
