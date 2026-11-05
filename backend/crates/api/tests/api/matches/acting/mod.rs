//! Quem age pelo time: cada ação (vetar, votar, banir, escolher, Pronto, enviar,
//! confirmar, contestar, pedir e responder o desfazer) × cada valor da regra ×
//! capitão, colega, adversário e organizador. O capitão escolhendo por um colega,
//! o Pronto de todos com o elenco mudando no meio, o que a série diz a quem não
//! pode agir, as tarefas, os avisos e a regra congelada.
//!
//! Em todos: A = P1 (capitão) e P2; B = P3 (capitão) e P4. `ana` organiza e não
//! joga; `bia` é da comunidade e não joga.

mod accept;
mod ready;
mod report;
mod rules;
mod steps;

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::realtime::{clear, ids, notices, outbox};
use crate::schema::{CHECK, exec, rejected, rule};
use crate::teams::{started, tdm};

const KEY: &str = "wb-1-1";

const NOT_CAPTAIN: &str = "match.not_captain";
const NOT_PLAYER: &str = "match.not_player";
const NOT_YOUR_TURN: &str = "veto.not_your_turn";

fn uri(action: &str) -> String {
    format!("{COPA}/matches/{KEY}/veto/{action}")
}

fn rules(veto: &str, ban: &str, pick: &str, report: &str) -> Value {
    json!({ "team_rules": { "veto": veto, "ban": ban, "pick": pick, "report": report } })
}

fn all_captain() -> Value {
    rules("captain", "captain", "captain", "captain")
}

fn all_players() -> Value {
    rules("anyone", "anyone", "each_player", "anyone")
}

/// Sem mapas, `bans` de cada lado e os picks em `order`, numa série de `games`.
fn draft(order: &str, bans: u8, games: u8) -> Value {
    with(
        json!({
            "veto": { "map_mode": "off", "champion_phase": true, "pick_order": order },
            "champion_rules": {
                "bans_per_series": bans, "mirror": true, "same_team": false, "no_repeat": null
            },
        }),
        single(games, games),
    )
}

/// Só o veto de mapas (uma partida, sete mapas: só bans).
fn map_veto() -> Value {
    json!({ "veto": { "map_mode": "veto", "champion_phase": false } })
}

struct Cast {
    /// O estado do app, para os testes do relógio do veto.
    state: api::AppState,
    app: Router,
    ana: User,
    bia: User,
    p: Vec<User>,
    teams: Vec<i64>,
}

async fn cast(db: &PgPool, team_rules: Value, changes: Value) -> Cast {
    // Como `quake_br`, com o estado à mão.
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(db, "Ana").await;
    let bia = user(db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    let p = players(&app, db, &ana, 4).await;
    let body = tdm(2, with(team_rules, changes));
    let teams = started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    Cast {
        state,
        app,
        ana,
        bia,
        p,
        teams,
    }
}

async fn ready(c: &Cast, who: &User) -> (StatusCode, Value) {
    post_empty(&c.app, &uri("ready"), who).await
}

async fn ok_ready(c: &Cast, who: &User) -> Value {
    let (status, body) = ready(c, who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

async fn all_ready(c: &Cast) {
    for who in &c.p {
        ok_ready(c, who).await;
    }
}

async fn captains_ready(c: &Cast) {
    ok_ready(c, &c.p[0]).await;
    ok_ready(c, &c.p[2]).await;
}

async fn step(c: &Cast, who: &User, body: Value) -> (StatusCode, Value) {
    post(&c.app, &uri("steps"), who, body).await
}

async fn ok_step(c: &Cast, who: &User, body: Value) -> Value {
    let (status, response) = step(c, who, body.clone()).await;
    assert_eq!(status, StatusCode::OK, "{body}: {response}");
    response
}

/// O passo é recusado com `code`.
async fn refused(c: &Cast, who: &User, body: Value, status: StatusCode, code: &str) {
    assert_eq!(step(c, who, body).await, (status, err(code)));
}

fn last_event(detail: &Value) -> Value {
    detail["veto"]["history"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone()
}

fn users(list: &Value) -> Vec<i64> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|p| p["user_id"].as_i64().unwrap())
        .collect()
}
