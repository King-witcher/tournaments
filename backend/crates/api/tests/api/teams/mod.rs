//! Testes de `teams` no TDM: a formação antes do início (`formation`: troca
//! de modo com inscritos, sorteio aleatório e balanceado, salvar a formação
//! inteira, iniciar com time incompleto) e o time depois de formado
//! (`roster`: nome, capitão, substituição, quem sai da comunidade). O
//! seeding fica em `participants::seeding`. Helpers para montar um TDM.

mod formation;
mod roster;

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};

use crate::common::*;
use crate::matches::{COPA, single};

/// TDM `copa` com times de `size`, sem check-in, eliminação simples md1, sem
/// veto e com todos os jogadores agindo pelo time (sem dizer, a API põe o
/// Balanceado), com `changes`.
pub fn tdm(size: u8, changes: Value) -> Value {
    with(
        duel("copa"),
        with(
            with(
                json!({
                    "mode": "tdm",
                    "team_size": size,
                    "check_in": false,
                    "team_rules": {
                        "veto": "anyone", "ban": "anyone", "pick": "each_player", "report": "anyone"
                    }
                }),
                single(1, 1),
            ),
            changes,
        ),
    )
}

pub async fn formation(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, &format!("{COPA}/teams"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

pub async fn draw(app: &Router, who: &User, method: &str) -> (StatusCode, Value) {
    post(
        app,
        &format!("{COPA}/teams/draw"),
        who,
        json!({ "method": method }),
    )
    .await
}

/// Salva a formação: `teams` é a lista de times (`id`, `members`, `captain`).
pub async fn save(app: &Router, who: &User, teams: Value) -> (StatusCode, Value) {
    put(
        app,
        &format!("{COPA}/teams/formation"),
        who,
        json!({ "teams": teams }),
    )
    .await
}

/// Um time novo, para `save`: o primeiro de `players` é o capitão.
pub fn new_team(players: &[&User]) -> Value {
    let ids: Vec<i64> = players.iter().map(|p| p.id).collect();
    json!({ "members": ids, "captain": ids[0] })
}

/// O time de `player` na formação (`None`: reserva).
pub fn team_of(formation: &Value, player: &User) -> Option<Value> {
    formation["teams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| {
            t["members"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["user_id"] == player.id)
        })
        .cloned()
}

/// Os `user_id` de cada time, na ordem da formação.
pub fn rosters(formation: &Value) -> Vec<Vec<i64>> {
    formation["teams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            t["members"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| m["user_id"].as_i64().unwrap())
                .collect()
        })
        .collect()
}

pub fn reserves(formation: &Value) -> Vec<i64> {
    formation["reserves"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["user_id"].as_i64().unwrap())
        .collect()
}

/// Cria e publica o TDM `body`, inscreve `who` nessa ordem e forma os times
/// à mão: cada fatia de `size` jogadores é um time, o primeiro de cada uma é
/// o capitão. Devolve os ids dos times, na ordem.
pub async fn formed(app: &Router, ana: &User, body: Value, size: usize, who: &[&User]) -> Vec<i64> {
    create_tournament(app, ana, body).await;
    publish(app, ana, "copa").await;
    register(app, "copa", who).await;
    let (status, f) = save(
        app,
        ana,
        Value::Array(who.chunks(size).map(new_team).collect()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{f}");
    who.chunks(size)
        .map(|chunk| team_of(&f, chunk[0]).unwrap()["id"].as_i64().unwrap())
        .collect()
}

/// `formed` e inicia, com a ordem da chave na ordem dos times.
pub async fn started(
    app: &Router,
    ana: &User,
    body: Value,
    size: usize,
    who: &[&User],
) -> Vec<i64> {
    let teams = formed(app, ana, body, size, who).await;
    let (status, seeded) = put(
        app,
        &format!("{COPA}/seeding"),
        ana,
        json!({ "team_ids": teams }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{seeded}");
    let (status, body) = post_empty(app, &format!("{COPA}/start"), ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    teams
}
