//! Testes de `matches`: a chave (`bracket`), envio e confirmação mapa a mapa
//! (`results`), contestação e organizador (`organizer`), mapa anulado para
//! ser jogado de novo (`replay`), correções e fim (`corrections`), as regras
//! do banco para mapas, séries e histórico (`schema`) e a conversão das
//! séries antigas para mapas (`migration`). Helpers para montar um torneio
//! iniciado com jogadores em ordem de seed.

mod bracket;
mod corrections;
mod migration;
mod organizer;
mod replay;
mod results;
mod schema;

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

pub const COPA: &str = "/tenants/quake-br/tournaments/copa";

/// Mapas do pool de duelo, um para cada mapa de uma série.
pub const MAPS: [&str; 5] = ["awoken", "exile", "insomnia", "blood-run", "deep-embrace"];

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

pub async fn detail(app: &Router, who: &User, key: &str) -> Value {
    let (status, body) = get(app, &format!("{COPA}/matches/{key}"), who).await;
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

fn game_uri(key: &str, game: u8, action: &str) -> String {
    format!("{COPA}/matches/{key}/games/{game}{action}")
}

/// Um mapa vencido pelo lado `winner` (`"a"` ou `"b"`) em `map`, sem frags.
pub fn game(winner: &str, map: &str) -> Value {
    json!({ "winner": winner, "map": map })
}

/// O mapa `n` de `key`, enviado por um jogador.
pub async fn report(
    app: &Router,
    who: &User,
    key: &str,
    n: u8,
    body: Value,
) -> (StatusCode, Value) {
    post(app, &game_uri(key, n, "/report"), who, body).await
}

pub async fn confirm(app: &Router, who: &User, key: &str, n: u8) -> (StatusCode, Value) {
    post_empty(app, &game_uri(key, n, "/confirm"), who).await
}

pub async fn contest(
    app: &Router,
    who: &User,
    key: &str,
    n: u8,
    comment: &str,
) -> (StatusCode, Value) {
    post(
        app,
        &game_uri(key, n, "/contest"),
        who,
        json!({ "comment": comment }),
    )
    .await
}

/// O organizador lança ou corrige o mapa `n`.
pub async fn set_game(
    app: &Router,
    who: &User,
    key: &str,
    n: u8,
    body: Value,
    comment: Option<&str>,
) -> (StatusCode, Value) {
    put(
        app,
        &game_uri(key, n, ""),
        who,
        json!({ "game": body, "comment": comment }),
    )
    .await
}

/// O organizador dá W.O. na série para o lado `winner`.
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

/// W.O. para `set_result`.
pub fn walkover(winner: &str) -> Value {
    json!({ "kind": "walkover", "winner": winner })
}

/// A série `key` jogada `a` × `b` em mapas: os do perdedor primeiro, depois
/// os do vencedor, cada um enviado por `reporter` e confirmado por
/// `opponent`.
pub async fn play(app: &Router, reporter: &User, opponent: &User, key: &str, a: u8, b: u8) {
    let (winner, loser) = if a > b { ("a", "b") } else { ("b", "a") };
    let order = std::iter::repeat_n(loser, a.min(b) as usize)
        .chain(std::iter::repeat_n(winner, a.max(b) as usize));
    for (i, side) in order.enumerate() {
        let n = i as u8 + 1;
        let (status, body) = report(app, reporter, key, n, game(side, MAPS[i])).await;
        assert_eq!(status, StatusCode::OK, "{key} {n}: {body}");
        let (status, body) = confirm(app, opponent, key, n).await;
        assert_eq!(status, StatusCode::OK, "{key} {n}: {body}");
    }
}

/// Os `kind` do histórico da partida, na ordem, com o mapa (`reported:1`) nos
/// eventos de um mapa.
pub fn kinds(detail: &Value) -> Vec<String> {
    detail["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let kind = e["kind"].as_str().unwrap();
            match e["game"].as_i64() {
                Some(n) => format!("{kind}:{n}"),
                None => kind.to_owned(),
            }
        })
        .collect()
}

/// Volta o relógio `minutes` minutos em todos os mapas e eventos, em vez de
/// esperar a janela de contestação.
pub async fn age(db: &PgPool, minutes: i32) {
    sqlx::query!(
        "update match_events set created_at = created_at - make_interval(mins => $1)",
        minutes
    )
    .execute(db)
    .await
    .unwrap();
    sqlx::query!(
        "update match_games set reported_at = reported_at - make_interval(mins => $1),
                                confirm_by = confirm_by - make_interval(mins => $1)
         where status = 'pending'",
        minutes
    )
    .execute(db)
    .await
    .unwrap();
}
