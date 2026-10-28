//! Testes de `matches`: a chave (`bracket`) e as regras do banco para
//! resultados e histórico (`schema`). Helpers para montar um torneio iniciado
//! com jogadores em ordem de seed.

mod bracket;
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
