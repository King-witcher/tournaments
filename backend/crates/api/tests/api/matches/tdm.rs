//! A série no TDM: qualquer jogador do time age pelo lado, a janela
//! simultânea de campeões (cada um escolhe o de cada partida, ao vivo, com o
//! sorteio no fim e sem repetir no time) e o rating de uma partida pela
//! escalação.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::realtime::{clear, notices, outbox};
use crate::teams::{started, tdm};

const KEY: &str = "wb-1-1";

fn uri(action: &str) -> String {
    format!("{COPA}/matches/{KEY}/veto/{action}")
}

async fn ready(app: &Router, who: &User) -> (StatusCode, Value) {
    post_empty(app, &uri("ready"), who).await
}

/// O ban do lado.
async fn ban(app: &Router, who: &User, champion: &str) -> (StatusCode, Value) {
    post(app, &uri("steps"), who, json!({ "champion": champion })).await
}

/// O pick de quem age para a partida `game`, na janela simultânea.
async fn step(app: &Router, who: &User, game: u8, champion: &str) -> (StatusCode, Value) {
    post(
        app,
        &uri("steps"),
        who,
        json!({ "champion": champion, "game": game }),
    )
    .await
}

async fn ok(app: &Router, who: &User, game: u8, champion: &str) -> Value {
    let (status, body) = step(app, who, game, champion).await;
    assert_eq!(status, StatusCode::OK, "{champion}: {body}");
    body
}

async fn ok_ban(app: &Router, who: &User, champion: &str) -> Value {
    let (status, body) = ban(app, who, champion).await;
    assert_eq!(status, StatusCode::OK, "{champion}: {body}");
    body
}

fn rules(bans: u8, same_team: bool) -> Value {
    json!({ "bans_per_series": bans, "mirror": true, "same_team": same_team, "no_repeat": null })
}

/// 2v2 na final única md1, mapa combinado fora, `bans` de cada lado na
/// série: A = P1 (capitão) e P2; B = P3 (capitão) e P4.
async fn final_2v2(db: &PgPool, bans: u8, extra: Value) -> (Router, User, Vec<User>) {
    let (app, ana, _) = quake_br(db).await;
    let p = players(&app, db, &ana, 4).await;
    let body = tdm(
        2,
        with(
            json!({
                "veto": { "map_mode": "off", "champion_phase": true },
                "champion_rules": rules(bans, false)
            }),
            extra,
        ),
    );
    started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    (app, ana, p)
}

/// Volta `seconds` segundos no relógio de todos os eventos de veto.
async fn age_veto(db: &PgPool, seconds: i32) {
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

fn picks(detail: &Value, game: usize) -> Vec<(i64, String)> {
    detail["veto"]["games"][game - 1]["team_picks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["user_id"].as_i64().unwrap(),
                p["champion"]["slug"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[tokio::test]
async fn anyone_of_the_team_acts_for_the_side() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, p) = final_2v2(&db, 1, json!({})).await;
    // Sem capitão nas ações: P2 também está "pronto" pelo time.
    assert_eq!(detail(&app, &p[1], KEY).await["summary"]["task"], "ready");
    let (status, body) = ready(&app, &p[1]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["veto"]["ready_a"], true);
    assert_eq!(
        detail(&app, &p[0], KEY).await["summary"]["task"],
        "await_opponent"
    );
    clear(&db).await;
    let (status, body) = ready(&app, &p[3]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // O começo do veto avisa todos dos dois times.
    let started: Vec<Vec<i64>> = notices(&outbox(&db).await)
        .into_iter()
        .filter(|(code, _, _)| code == "match.veto_started")
        .map(|(_, users, _)| users)
        .collect();
    let mut a = vec![p[0].id, p[1].id];
    a.sort();
    let mut b = vec![p[2].id, p[3].id];
    b.sort();
    assert!(started.contains(&a) && started.contains(&b), "{started:?}");

    // O ban é do lado B (o seed pior): qualquer um dos dois bane, e vale o
    // primeiro; o histórico grava quem fez.
    let d = detail(&app, &p[3], KEY).await;
    assert_eq!(d["summary"]["task"], "veto");
    assert_eq!(d["veto"]["options"].as_array().unwrap().len(), 16);
    assert_eq!(
        ban(&app, &p[0], "anarki").await,
        (StatusCode::CONFLICT, err("veto.not_your_turn"))
    );
    let d = ok_ban(&app, &p[3], "anarki").await;
    let last = d["veto"]["history"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(
        (&last["kind"], &last["actor"]),
        (&json!("ban"), &json!("P4"))
    );
    // Pedir para desfazer também é de qualquer um do lado; responder, do
    // outro.
    let (status, body) = post_empty(&app, &uri("undo-request"), &p[2]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        post(
            &app,
            &uri("undo-request/answer"),
            &p[3],
            json!({ "accept": true })
        )
        .await,
        (StatusCode::FORBIDDEN, err("veto.own_request"))
    );
    let (status, body) = post(
        &app,
        &uri("undo-request/answer"),
        &p[1],
        json!({ "accept": true }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["veto"]["bans"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn everyone_picks_every_game_at_once_without_repeating_in_the_team() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, p) = final_2v2(&db, 1, single(3, 3)).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[2]).await;
    ok_ban(&app, &p[2], "anarki").await;
    clear(&db).await;
    let d = ok_ban(&app, &p[0], "athena").await;

    // A janela abre para os quatro, avisados, com as três partidas.
    assert_eq!(d["veto"]["turn"]["action"], "team_pick");
    assert_eq!(d["veto"]["turn"]["sides"], json!(["a", "b"]));
    assert_eq!(d["veto"]["team_pick_seconds"], 60);
    let missing = d["veto"]["missing"].as_array().unwrap();
    assert_eq!(missing.len(), 4);
    assert_eq!(missing[0]["games"], json!([1, 2, 3]));
    let mut everyone: Vec<i64> = p.iter().map(|u| u.id).collect();
    everyone.sort();
    assert_eq!(
        notices(&outbox(&db).await),
        [(
            "match.team_pick_open".to_owned(),
            everyone,
            json!({ "tournament": "Copa de Outubro", "match": KEY, "game": 1 })
        )]
    );
    // Cada um vê as próprias opções, por partida; os banidos não estão.
    let d = detail(&app, &p[1], KEY).await;
    assert_eq!(d["summary"]["task"], "veto");
    let slots = d["veto"]["picks"].as_array().unwrap();
    assert_eq!(slots.len(), 3);
    let options = slots[0]["options"].as_array().unwrap();
    assert_eq!(options.len(), 14);
    assert!(!options.contains(&json!("anarki")));

    // P2 escolhe antes do capitão; P1 não repete o campeão de P2 na mesma
    // partida, mas pode em outra.
    ok(&app, &p[1], 1, "clutch").await;
    assert_eq!(
        step(&app, &p[0], 1, "clutch").await,
        (StatusCode::CONFLICT, err("veto.champion_taken"))
    );
    ok(&app, &p[0], 2, "clutch").await;
    // Espelho entre times vale.
    ok(&app, &p[3], 1, "clutch").await;
    // Definitiva.
    assert_eq!(
        step(&app, &p[1], 1, "eisen").await,
        (StatusCode::CONFLICT, err("veto.already_picked"))
    );
    // Sem a partida, ou fora da série.
    assert_eq!(
        ban(&app, &p[1], "eisen").await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
    assert_eq!(
        step(&app, &p[1], 4, "eisen").await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("veto.option_invalid"))
    );
    for (who, game, champion) in [
        (&p[0], 1, "eisen"),
        (&p[0], 3, "galena"),
        (&p[1], 2, "eisen"),
        (&p[1], 3, "keel"),
        (&p[2], 1, "galena"),
        (&p[2], 2, "galena"),
        (&p[2], 3, "galena"),
        (&p[3], 2, "keel"),
    ] {
        ok(&app, who, game, champion).await;
    }
    let d = detail(&app, &p[1], KEY).await;
    assert_eq!(d["summary"]["task"], "await_opponent");
    let d = ok(&app, &p[3], 3, "nyx").await;
    assert_eq!(d["veto"]["status"], "done");
    assert_eq!(d["veto"]["reportable"], true);
    assert_eq!(
        picks(&d, 1),
        [
            (p[1].id, "clutch".to_owned()),
            (p[3].id, "clutch".to_owned()),
            (p[0].id, "eisen".to_owned()),
            (p[2].id, "galena".to_owned()),
        ]
    );
    let history = d["veto"]["history"].as_array().unwrap();
    let last = history.last().unwrap();
    assert_eq!(
        (&last["player"], &last["pick_game"]),
        (&json!("P4"), &json!(3))
    );
}

#[tokio::test]
async fn the_team_can_be_allowed_to_repeat() {
    let (db, _lease) = crate::db::test_db().await;
    let extra = json!({ "champion_rules": rules(0, true) });
    let (app, _, p) = final_2v2(&db, 0, extra).await;
    ready(&app, &p[0]).await;
    ready(&app, &p[2]).await;
    ok(&app, &p[0], 1, "clutch").await;
    ok(&app, &p[1], 1, "clutch").await;
}

#[tokio::test]
async fn the_window_closes_with_a_draw_for_whoever_did_not_pick() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, p) = final_2v2(
        &db,
        0,
        json!({ "veto": {
        "map_mode": "off", "champion_phase": true, "team_pick_seconds": 15
    } }),
    )
    .await;
    ready(&app, &p[0]).await;
    ready(&app, &p[2]).await;
    ok(&app, &p[0], 1, "anarki").await;
    // O prazo é um só para a janela: o pick de P1 não o reinicia.
    let d = detail(&app, &p[0], KEY).await;
    let deadline = d["veto"]["deadline"].as_str().unwrap().to_owned();
    ok(&app, &p[3], 1, "athena").await;
    assert_eq!(detail(&app, &p[0], KEY).await["veto"]["deadline"], deadline);

    age_veto(&db, 16).await;
    let d = detail(&app, &p[0], KEY).await;
    assert_eq!(d["veto"]["status"], "done", "{d}");
    let all = picks(&d, 1);
    assert_eq!(all.len(), 4);
    let drawn: Vec<&Value> = d["veto"]["games"][0]["team_picks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["timeout"] == true)
        .collect();
    assert_eq!(drawn.len(), 2);
    // O sorteado de P2 não repete o de P1, o colega.
    let of = |u: &User| all.iter().find(|(id, _)| *id == u.id).unwrap().1.clone();
    assert_ne!(of(&p[1]), of(&p[0]));
    assert_ne!(of(&p[2]), of(&p[3]));
}

#[tokio::test]
async fn a_tdm_map_counts_for_everyone_in_the_lineup() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    started(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let (status, body) = report(&app, &p[1], KEY, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[3], KEY, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, ranking) = get(&app, "/tenants/quake-br/ratings", &ana).await;
    assert_eq!(status, StatusCode::OK, "{ranking}");
    let rating = |u: &User| {
        ranking["players"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["user_id"] == u.id)
            .map(|r| (r["rating"].as_i64().unwrap(), r["games"].as_i64().unwrap()))
    };
    // Todos com 1500: E = 0,5 e K = 80 no primeiro mapa.
    assert_eq!(rating(&p[0]), Some((1540, 1)));
    assert_eq!(rating(&p[1]), Some((1540, 1)));
    assert_eq!(rating(&p[2]), Some((1460, 1)));
    assert_eq!(rating(&p[3]), Some((1460, 1)));
}

#[tokio::test]
async fn teams_play_a_round_robin_too() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 6).await;
    let body = tdm(2, table("round_robin", json!({ "best_of": 1 })));
    let who: Vec<&User> = p.iter().collect();
    started(&app, &ana, body, 2, &who).await;
    let b = bracket(&app, &p[0]).await;
    // Três times: três partidas, cada time com os dois jogadores.
    assert_eq!(keys_in(&b, "ready").len(), 3);
    assert!(
        b["entrants"]
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t["members"].as_array().unwrap().len() == 2)
    );
    // Na tabela, o mapa conta para o time.
    let key = keys_in(&b, "ready")[0].clone();
    let d = detail(&app, &p[0], &key).await;
    let a_player = d["a"]["members"][1]["user_id"].as_i64().unwrap();
    let b_player = d["b"]["members"][1]["user_id"].as_i64().unwrap();
    let of = |id: i64| p.iter().find(|u| u.id == id).unwrap();
    let (status, body) = report(&app, of(a_player), &key, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, of(b_player), &key, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &p[0]).await;
    assert_eq!(find(&b, &key)["state"], "completed");
}

#[tokio::test]
async fn a_sent_game_does_not_reopen_the_picks_for_a_substitute() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = tdm(
        2,
        json!({
            "veto": { "map_mode": "off", "champion_phase": true },
            "champion_rules": rules(0, false)
        }),
    );
    let teams = started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    let substitution = format!("{COPA}/teams/{}/substitution", teams[0]);
    let sub = |out: &User, sub: &User| json!({ "out": out.id, "in": sub.id });
    ready(&app, &p[0]).await;
    ready(&app, &p[2]).await;
    for (who, champion) in [(&p[0], "anarki"), (&p[1], "athena"), (&p[2], "anarki")] {
        ok(&app, who, 1, champion).await;
    }
    let d = ok(&app, &p[3], 1, "athena").await;
    assert_eq!(d["veto"]["status"], "done");

    // Antes do envio, quem entra no time passa a faltar e escolhe; a
    // escolha de quem saiu fica no histórico e não conta.
    let (status, f) = post(&app, &substitution, &ana, sub(&p[1], &bia)).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let d = detail(&app, &bia, KEY).await;
    assert_eq!(d["veto"]["status"], "in_progress");
    assert_eq!(d["veto"]["missing"][0]["user_id"], bia.id);
    assert!(
        d["veto"]["picks"][0]["options"]
            .as_array()
            .unwrap()
            .contains(&json!("athena"))
    );
    ok(&app, &bia, 1, "athena").await;

    // Enviada a partida, a troca não reabre a escolha.
    let (status, body) = report(&app, &bia, KEY, 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, f) = post(&app, &substitution, &ana, sub(&bia, &p[1])).await;
    assert_eq!(status, StatusCode::OK, "{f}");
    let d = detail(&app, &p[1], KEY).await;
    assert_eq!(d["veto"]["status"], "done", "{d}");
    assert!(d["veto"]["missing"].as_array().unwrap().is_empty());
    assert!(d["veto"]["picks"].as_array().unwrap().is_empty());
    assert_eq!(
        step(&app, &p[1], 1, "clutch").await,
        (StatusCode::CONFLICT, err("veto.finished"))
    );
    // Contam os de quem jogou: bia, não P2.
    let all = picks(&d, 1);
    assert_eq!(all.len(), 4);
    assert!(all.contains(&(bia.id, "athena".to_owned())));
    assert!(!all.iter().any(|(id, _)| *id == p[1].id));
}

#[tokio::test]
async fn the_screen_says_when_the_rule_gave_way() {
    // 2v2 md3, regra do time na série, pool de 3 campeões ativos: para a
    // partida 2, o time A tem só um livre; quem escolhe por último fica com a
    // regra cedida, e a tela diz.
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = tdm(
        2,
        with(
            json!({
                "veto": { "map_mode": "off", "champion_phase": true },
                "champion_rules": {
                    "bans_per_series": 0, "mirror": true, "same_team": false,
                    "no_repeat": { "who": "team", "scope": "series" }
                }
            }),
            single(3, 3),
        ),
    );
    started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    // Depois de iniciar (a criação e o início conferem o pool).
    sqlx::query("update champions set active = slug in ('anarki', 'athena', 'clutch')")
        .execute(&db)
        .await
        .unwrap();
    ready(&app, &p[0]).await;
    ready(&app, &p[2]).await;
    ok(&app, &p[0], 1, "anarki").await;
    ok(&app, &p[1], 1, "athena").await;

    // Partida 2: o time já tem anarki e athena; só clutch é livre.
    let d = detail(&app, &p[0], KEY).await;
    let slot = |d: &Value, game: i64| -> Value {
        d["veto"]["picks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["game"] == game)
            .unwrap()
            .clone()
    };
    assert_eq!(slot(&d, 2)["options"], json!(["clutch"]));
    assert!(slot(&d, 2)["gave_way"].is_null());
    ok(&app, &p[1], 2, "clutch").await;
    let d = detail(&app, &p[0], KEY).await;
    let two = slot(&d, 2);
    assert_eq!(two["gave_way"], "rule");
    assert_eq!(two["options"], json!(["anarki", "athena"]));
    assert_eq!(
        two["unavailable"],
        json!([
            { "slug": "anarki", "reason": "repeat" },
            { "slug": "athena", "reason": "repeat" },
            { "slug": "clutch", "reason": "teammate" }
        ])
    );
    ok(&app, &p[0], 2, "anarki").await;
}
