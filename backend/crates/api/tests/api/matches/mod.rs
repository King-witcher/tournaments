//! Testes de `matches`: a chave (`bracket`), envio e confirmação
//! (`results`), contestação e organizador (`organizer`), correções e fim
//! (`corrections`) e as regras do banco para
//! resultados e histórico (`schema`). Helpers para montar um torneio iniciado
//! com jogadores em ordem de seed.

mod bracket;
mod corrections;
mod organizer;
mod results;
mod schema;

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

pub const COPA: &str = "/tenants/quake-br/tournaments/copa";

/// Torneio `copa` sem check-in: `duel` com `changes`.
pub fn copa(changes: Value) -> Value {
    with(
        duel("copa"),
        with(json!({ "check_in_minutes": null }), changes),
    )
}

/// Eliminação simples com a final em md`finals` e o resto em md`best_of`.
pub fn single(best_of: u8, finals: u8) -> Value {
    json!({ "stages": [{
        "format": "elimination",
        "bracket": "single",
        "best_of": best_of,
        "finals_best_of": finals
    }] })
}

/// Cria e publica `body` em `quake-br`, inscreve `players` pelo botão nessa
/// ordem de seed (o primeiro é o seed 1) e inicia. `ana` (owner) organiza.
pub async fn start(app: &Router, ana: &User, body: Value, players: &[&User]) {
    create_tournament(app, ana, body).await;
    publish(app, ana, "copa").await;
    register(app, "copa", players).await;
    let order: Vec<i64> = players.iter().map(|p| p.id).collect();
    let (status, seeded) = put(
        app,
        &format!("{COPA}/seeding"),
        ana,
        json!({ "user_ids": order }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{seeded}");
    let (status, started) = post_empty(app, &format!("{COPA}/start"), ana).await;
    assert_eq!(status, StatusCode::OK, "{started}");
}

/// `n` jogadores novos de `quake-br` (P1, P2, ...).
pub async fn players(app: &Router, db: &PgPool, ana: &User, n: usize) -> Vec<User> {
    let mut out = Vec::with_capacity(n);
    for i in 1..=n {
        out.push(member(app, db, ana, &format!("P{i}")).await);
    }
    out
}

pub async fn bracket(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, &format!("{COPA}/bracket"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// A partida `key` da chave.
pub fn find<'a>(bracket: &'a Value, key: &str) -> &'a Value {
    bracket["matches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == key)
        .unwrap_or_else(|| panic!("sem a partida {key}"))
}

/// `(user_id de a, user_id de b)` da partida.
pub fn sides(m: &Value) -> (Value, Value) {
    (m["a"]["user_id"].clone(), m["b"]["user_id"].clone())
}

fn match_uri(key: &str, action: &str) -> String {
    format!("{COPA}/matches/{key}/{action}")
}

/// Placar de `a` e de `b`, sem detalhe por mapa.
pub fn score(a: u8, b: u8) -> Value {
    json!({ "a_wins": a, "b_wins": b })
}

pub async fn report(app: &Router, who: &User, key: &str, body: Value) -> (StatusCode, Value) {
    post(app, &match_uri(key, "report"), who, body).await
}

pub async fn confirm(app: &Router, who: &User, key: &str) -> (StatusCode, Value) {
    post_empty(app, &match_uri(key, "confirm"), who).await
}

pub async fn contest(app: &Router, who: &User, key: &str, comment: &str) -> (StatusCode, Value) {
    post(
        app,
        &match_uri(key, "contest"),
        who,
        json!({ "comment": comment }),
    )
    .await
}

/// O organizador define o resultado (`result`: `{"kind": "played", ...}` ou
/// `{"kind": "walkover", "winner": "a"}`).
pub async fn set_result(
    app: &Router,
    who: &User,
    key: &str,
    result: Value,
    comment: Option<&str>,
) -> (StatusCode, Value) {
    put(
        app,
        &format!("{COPA}/matches/{key}/result"),
        who,
        json!({ "result": result, "comment": comment }),
    )
    .await
}

/// Série jogada para `set_result`.
pub fn played(a: u8, b: u8) -> Value {
    json!({ "kind": "played", "a_wins": a, "b_wins": b })
}

/// `reporter` envia `a` × `b` em `key` e `opponent` confirma.
pub async fn play(app: &Router, reporter: &User, opponent: &User, key: &str, a: u8, b: u8) {
    let (status, body) = report(app, reporter, key, score(a, b)).await;
    assert_eq!(status, StatusCode::OK, "{key}: {body}");
    let (status, body) = confirm(app, opponent, key).await;
    assert_eq!(status, StatusCode::OK, "{key}: {body}");
}

/// Os `kind` do histórico da partida, na ordem.
pub fn kinds(detail: &Value) -> Vec<&str> {
    detail["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect()
}
