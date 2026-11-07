//! As séries pares no round robin, com a md2 no caminho completo: fecham na
//! maioria ou, com todos os mapas e metade para cada lado, empatam; o saldo de
//! séries na tabela, o que muda entre vitória e empate (contestação e correção),
//! W.O., desistência e o veto até sobrarem N mapas. A md4 entra nos placares que
//! a md2 não tem.

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::realtime::{clear, outbox};

fn even(best_of: u8, extra: Value) -> Value {
    copa(with(
        table("round_robin", json!({ "best_of": best_of })),
        extra,
    ))
}

/// Round robin de `n` jogadores na melhor de `best_of`, sem veto, já iniciado.
async fn started_best_of(db: &PgPool, n: usize, best_of: u8) -> (Router, User, Vec<User>) {
    let (app, ana, _) = quake_br(db).await;
    let p = players(&app, db, &ana, n).await;
    let seeded: Vec<&User> = p.iter().collect();
    start(&app, &ana, even(best_of, json!({})), &seeded).await;
    (app, ana, p)
}

/// Round robin md2 de `n` jogadores. Com 3, as séries são `rr-1-1` (P2 x P3),
/// `rr-2-1` (P1 x P3) e `rr-3-1` (P1 x P2).
async fn started(db: &PgPool, n: usize) -> (Router, User, Vec<User>) {
    started_best_of(db, n, 2).await
}

/// O mapa `n` enviado por `reporter`, com o vencedor `winner` (o lado), e
/// confirmado por `opponent`.
async fn map_played(
    app: &Router,
    (reporter, opponent): (&User, &User),
    key: &str,
    n: u8,
    winner: &str,
) {
    let body = game(winner, MAPS[usize::from(n) - 1]);
    let (status, body) = report(app, reporter, key, n, body).await;
    assert_eq!(status, StatusCode::OK, "{key} {n}: {body}");
    let (status, body) = confirm(app, opponent, key, n).await;
    assert_eq!(status, StatusCode::OK, "{key} {n}: {body}");
}

/// A série inteira: o lado que vence cada mapa, na ordem.
async fn series_of(app: &Router, players: (&User, &User), key: &str, winners: &[&str]) {
    for (i, winner) in winners.iter().enumerate() {
        map_played(app, players, key, i as u8 + 1, winner).await;
    }
}

/// A série md2 inteira.
async fn series(app: &Router, players: (&User, &User), key: &str, winners: [&str; 2]) {
    series_of(app, players, key, &winners).await;
}

/// `(vitórias, empates, derrotas, saldo)` da linha de `who`.
fn record(b: &Value, who: &User) -> (i64, i64, i64, i64) {
    let r = table_row(b, who);
    (
        r["wins"].as_i64().unwrap(),
        r["draws"].as_i64().unwrap(),
        r["losses"].as_i64().unwrap(),
        r["series_diff"].as_i64().unwrap(),
    )
}

/// `(vencedor, mapas de a, mapas de b, walkover)` da série decidida.
fn result_of(b: &Value, key: &str) -> (Value, Value, Value, Value) {
    let r = &find(b, key)["result"];
    (
        r["winner"].clone(),
        r["a_wins"].clone(),
        r["b_wins"].clone(),
        r["walkover"].clone(),
    )
}

fn draw() -> (Value, Value, Value, Value) {
    (Value::Null, json!(1), json!(1), json!(false))
}

fn won(side: &str, a: u8, b: u8) -> (Value, Value, Value, Value) {
    (json!(side), json!(a), json!(b), json!(false))
}

#[tokio::test]
async fn one_map_never_decides_and_a_third_does_not_exist() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started(&db, 2).await;
    let (a, b) = (&p[0], &p[1]);
    let k = "rr-1-1";

    // Um mapa confirmado não fecha a série, e o segundo pode vir.
    map_played(&app, (a, b), k, 1, "a").await;
    let d = detail(&app, &ana, k).await;
    assert_eq!(d["summary"]["state"], "ready");
    assert_eq!(d["next_game"], 2);
    assert_eq!(
        (&d["summary"]["a_maps"], &d["summary"]["b_maps"]),
        (&json!(1), &json!(0))
    );
    assert_eq!(d["summary"]["result"], Value::Null);

    // 1 x 1 pendente: acabaram os mapas, e ainda não fechou.
    let (status, body) = report(&app, a, k, 2, game("b", MAPS[1])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["next_game"], Value::Null);
    assert_eq!(body["summary"]["state"], "ready");
    assert_eq!(
        report(&app, a, k, 3, game("a", MAPS[2])).await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "match.no_games_left", "params": { "best_of": 2 } })
        )
    );
    // Confirmado o segundo, a série fecha empatada e o torneio de 2 acaba.
    let (status, body) = confirm(&app, b, k, 2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "completed");
    assert_eq!(body["status"], "finished");

    let frags = (&body["summary"]["a_frags"], &body["summary"]["b_frags"]);
    assert_eq!(frags, (&json!(15), &json!(15)));
}

#[tokio::test]
async fn two_nil_decides_only_with_both_maps_confirmed() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started(&db, 2).await;
    let (a, b) = (&p[0], &p[1]);
    let k = "rr-1-1";
    let (status, body) = report(&app, a, k, 1, game("a", MAPS[0])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = report(&app, a, k, 2, game("a", MAPS[1])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Pelos enviados, a série acabou, mas confirma em sequência.
    assert_eq!(
        report(&app, a, k, 3, game("a", MAPS[2])).await,
        (StatusCode::CONFLICT, err("match.series_decided"))
    );
    let (status, body) = confirm(&app, b, k, 2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "ready");
    let (status, body) = confirm(&app, b, k, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "completed");
    let bracket = bracket(&app, &ana).await;
    assert_eq!(result_of(&bracket, k), won("a", 2, 0));
}

#[tokio::test]
async fn a_round_robin_plays_to_the_end_and_a_draw_closes_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _ana, p) = started(&db, 3).await;
    let (p1, p2, p3) = (&p[0], &p[1], &p[2]);
    let b = bracket(&app, p1).await;
    assert_eq!(
        b["format"],
        json!({ "format": "round_robin", "best_of": 2, "tiebreakers": ["head_to_head", "map_diff", "frag_diff"], "final_tie": "seed", "advance": null, "check_in": false })
    );
    assert_eq!(find(&b, "rr-1-1")["best_of"], 2);

    // P2 2 x 0 P3: saldo +1 e −1, o W de sempre.
    series(&app, (p2, p3), "rr-1-1", ["a", "a"]).await;
    let b = bracket(&app, p1).await;
    assert_eq!(result_of(&b, "rr-1-1"), won("a", 2, 0));
    assert_eq!(record(&b, p2), (1, 0, 0, 1));
    assert_eq!(record(&b, p3), (0, 0, 1, -1));
    assert_eq!(table_row(&b, p2)["map_diff"], 2);

    // P1 1 x 1 P3: o empate vale 0 e ninguém leva nada a mais.
    clear(&db).await;
    series(&app, (p1, p3), "rr-2-1", ["a", "b"]).await;
    let b = bracket(&app, p1).await;
    assert_eq!(b["status"], "in_progress");
    assert_eq!(result_of(&b, "rr-2-1"), draw());
    assert_eq!(find(&b, "rr-2-1")["state"], "completed");
    assert_eq!(record(&b, p1), (0, 1, 0, 0));
    assert_eq!(record(&b, p3), (0, 1, 1, -1));
    let row = table_row(&b, p1);
    assert_eq!(
        (
            &row["maps_won"],
            &row["maps_lost"],
            &row["played"],
            &row["points"]
        ),
        (&json!(1), &json!(1), &json!(1), &json!(0))
    );
    assert_eq!(
        (&row["frags_for"], &row["frags_against"], &row["frag_diff"]),
        (&json!(15), &json!(15), &json!(0))
    );
    // No histórico, a série fechou sem vencedor, em 1 x 1.
    let d = detail(&app, p1, "rr-2-1").await;
    let closed = d["history"].as_array().unwrap().last().unwrap();
    assert_eq!(closed["kind"], "decided");
    assert_eq!(
        closed["outcome"],
        json!({ "winner": null, "walkover": false, "winner_maps": 1, "loser_maps": 1 })
    );
    // O aviso do canal do Discord não diz que alguém venceu.
    let events = outbox(&db).await;
    let drawn = events
        .iter()
        .find(|e| e["announce"]["code"] == "match.series_drawn")
        .expect("o anúncio do empate");
    assert_eq!(
        drawn["announce"]["params"],
        json!({
            "tournament": "Copa de Outubro", "match": "rr-2-1",
            "side_a": "P1", "side_b": "P3", "score": "1-1",
        })
    );
    assert!(
        !events
            .iter()
            .any(|e| e["announce"]["code"] == "match.series_decided")
    );

    // A última série empatada encerra o torneio, com a classificação.
    series(&app, (p1, p2), "rr-3-1", ["b", "a"]).await;
    let b = bracket(&app, p1).await;
    assert_eq!(b["status"], "finished");
    assert_eq!(result_of(&b, "rr-3-1"), draw());
    // P2 (+1), P1 (0) e P3 (−1); o saldo é o primeiro critério.
    assert_eq!(
        standings(&b),
        json!([
            { "user_id": p2.id, "rank": 1, "tied": 1 },
            { "user_id": p1.id, "rank": 2, "tied": 1 },
            { "user_id": p3.id, "rank": 3, "tied": 1 },
        ])
    );
    let ahead: Vec<&Value> = (0..3).map(|i| &b["table"][i]["ahead_by"]).collect();
    assert_eq!(
        ahead,
        [&json!("series_diff"), &json!("series_diff"), &Value::Null]
    );
    assert_eq!(keys_in(&b, "ready"), Vec::<String>::new());
}

#[tokio::test]
async fn a_draw_between_two_goes_to_the_frags_on_the_table() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _ana, p) = started(&db, 3).await;
    let (p1, p2, p3) = (&p[0], &p[1], &p[2]);
    // P1 e P2 empatam entre si e vencem P3; P2 faz mais frags sobre ele.
    series(&app, (p1, p2), "rr-3-1", ["a", "b"]).await;
    series(&app, (p1, p3), "rr-2-1", ["a", "a"]).await;
    for (n, map) in [(1, MAPS[0]), (2, MAPS[1])] {
        let score = json!({ "map": map, "a_frags": 30, "b_frags": 1 });
        let (status, body) = report(&app, p2, "rr-1-1", n, score).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let (status, body) = confirm(&app, p3, "rr-1-1", n).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let b = bracket(&app, p1).await;
    let first = &b["table"][0];
    assert_eq!(player(&b, &first["team_id"]), json!(p2.id));
    assert_eq!(first["ahead_by"], "frag_diff");
    assert_eq!(record(&b, p1), (1, 1, 0, 1));
    assert_eq!(record(&b, p2), (1, 1, 0, 1));
}

#[tokio::test]
async fn correcting_moves_between_a_win_and_a_draw() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started(&db, 3).await;
    let (p2, p3) = (&p[1], &p[2]);
    let k = "rr-1-1";

    series(&app, (p2, p3), k, ["a", "a"]).await;
    assert_eq!(result_of(&bracket(&app, &ana).await, k), won("a", 2, 0));

    // Vitória vira empate: corrigir o mapa 2.
    clear(&db).await;
    let (status, body) = set_game(&app, &ana, k, 2, game("b", MAPS[1]), Some("placar")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &ana).await;
    assert_eq!(result_of(&b, k), draw());
    assert_eq!(record(&b, p2), (0, 1, 0, 0));
    assert_eq!(record(&b, p3), (0, 1, 0, 0));
    let events = outbox(&db).await;
    assert!(
        events
            .iter()
            .any(|e| e["announce"]["code"] == "match.series_drawn")
    );

    // Empate vira vitória do outro lado: corrigir o mapa 1. A posição no replay
    // continua a mesma.
    let (status, body) = set_game(&app, &ana, k, 1, game("b", MAPS[0]), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &ana).await;
    assert_eq!(result_of(&b, k), won("b", 0, 2));
    assert_eq!(record(&b, p3), (1, 0, 0, 1));
    assert_eq!(record(&b, p2), (0, 0, 1, -1));
    let d = detail(&app, &ana, k).await;
    assert_eq!(
        kinds(&d)[kinds(&d).len() - 4..],
        ["corrected:2", "decided", "corrected:1", "decided"]
    );
    let closed = d["history"].as_array().unwrap().last().unwrap();
    assert_eq!(closed["outcome"]["winner_maps"], 2);
}

#[tokio::test]
async fn contesting_reopens_a_draw_and_accepting_makes_it_a_win() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _ana, p) = started(&db, 3).await;
    let (p2, p3) = (&p[1], &p[2]);
    let k = "rr-1-1";

    series(&app, (p2, p3), k, ["a", "b"]).await;
    let d = detail(&app, p3, k).await;
    assert_eq!(d["summary"]["state"], "completed");
    assert_eq!(d["games"][0]["can_contest"], true);

    // Contestação tardia: a série empatada reabre.
    let (status, body) = contest(&app, p3, k, 1, "Ganhei este").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "ready");
    assert_eq!(body["summary"]["result"], Value::Null);
    assert_eq!(body["next_game"], Value::Null);
    let b = bracket(&app, p2).await;
    assert_eq!(record(&b, p2), (0, 0, 0, 0));
    assert_eq!(b["status"], "in_progress");

    // O capitão adversário aceita com o placar certo: P3 venceu os dois.
    let (status, body) = accept(&app, p2, k, 1, game("b", MAPS[0])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "completed");
    let b = bracket(&app, p2).await;
    assert_eq!(result_of(&b, k), won("b", 0, 2));
    assert_eq!(record(&b, p3), (1, 0, 0, 1));
}

#[tokio::test]
async fn contesting_a_win_and_resolving_it_as_a_draw() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started(&db, 3).await;
    let (p2, p3) = (&p[1], &p[2]);
    let k = "rr-1-1";

    series(&app, (p2, p3), k, ["a", "a"]).await;
    let (status, body) = contest(&app, p3, k, 2, "Este foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "ready");
    assert_eq!(kinds(&body).last().unwrap(), "reopened");

    let (status, body) = set_game(&app, &ana, k, 2, game("b", MAPS[1]), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["summary"]["state"], "completed");
    assert_eq!(result_of(&bracket(&app, &ana).await, k), draw());
}

#[tokio::test]
async fn a_walkover_is_worth_two_nil_and_replaces_a_draw() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started(&db, 3).await;
    let (p2, p3) = (&p[1], &p[2]);
    let k = "rr-1-1";

    // Com um mapa confirmado: o W.O. fecha, com vencedor, e o mapa fica.
    map_played(&app, (p2, p3), k, 1, "a").await;
    let (status, body) = set_result(&app, &ana, k, walkover("b"), Some("sumiu")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &ana).await;
    let r = &find(&b, k)["result"];
    assert_eq!((&r["walkover"], &r["winner"]), (&json!(true), &json!("b")));
    let row = table_row(&b, p3);
    assert_eq!(
        (&row["maps_won"], &row["maps_lost"]),
        (&json!(2), &json!(0))
    );
    assert_eq!(record(&b, p3), (1, 0, 0, 1));
    assert_eq!(record(&b, p2), (0, 0, 1, -1));

    // Numa série empatada, o W.O. passa por cima, e não há W.O. empatado.
    let k = "rr-2-1";
    series(&app, (&p[0], p3), k, ["a", "b"]).await;
    let (status, body) = set_result(&app, &ana, k, walkover("a"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &ana).await;
    let r = &find(&b, k)["result"];
    assert_eq!((&r["walkover"], &r["winner"]), (&json!(true), &json!("a")));
    assert_eq!(record(&b, &p[0]), (1, 0, 0, 1));
    assert_eq!((&r["a_wins"], &r["b_wins"]), (&Value::Null, &Value::Null));
    // Sem vencedor não existe: o corpo exige um lado.
    let (status, body) = set_result(&app, &ana, k, json!({ "kind": "walkover" }), None).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("request.invalid_body")
        )
    );
}

#[tokio::test]
async fn a_withdrawal_is_a_walkover_and_keeps_the_draws_already_played() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started(&db, 3).await;
    let (p1, p2, p3) = (&p[0], &p[1], &p[2]);
    series(&app, (p2, p3), "rr-1-1", ["a", "b"]).await;

    let team = team_ids(&app, &ana, "copa", &[p3]).await[0];
    let uri = format!("{COPA}/teams/{team}/withdrawal");
    let (status, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    // A série que faltava virou W.O.; o empate jogado fica.
    let b = bracket(&app, p1).await;
    assert_eq!(result_of(&b, "rr-1-1"), draw());
    let gone = &find(&b, "rr-2-1")["result"];
    assert_eq!(
        (&gone["walkover"], &gone["winner"], &gone["confirmed_via"]),
        (&json!(true), &json!("a"), &json!("withdrawal"))
    );
    assert_eq!(record(&b, p3), (0, 1, 1, -1));
    assert_eq!(record(&b, p1), (1, 0, 0, 1));
}

/// A contestação não trava o mapa seguinte, então um mapa confirmado pode vir
/// depois de um sem confirmar. O W.O. anula a partir do primeiro sem confirmar,
/// inclusive o confirmado depois dele: a numeração não pode ficar com buraco.
#[tokio::test]
async fn a_walkover_voids_from_the_first_unconfirmed_map_on() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started(&db, 3).await;
    let (p1, p2, p3) = (&p[0], &p[1], &p[2]);
    let maps_of = |b: &Value, k: &str| {
        let m = find(b, k);
        (m["a_maps"].clone(), m["b_maps"].clone())
    };

    // Do organizador: mapa 1 contestado e mapa 2 confirmado.
    let k = "rr-1-1";
    let (status, body) = report(&app, p2, k, 1, game("a", MAPS[0])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = contest(&app, p3, k, 1, "não foi assim").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    map_played(&app, (p2, p3), k, 2, "a").await;
    let (status, body) = set_result(&app, &ana, k, walkover("b"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &ana).await;
    assert_eq!(find(&b, k)["result"]["winner"], json!("b"));
    assert_eq!(maps_of(&b, k), (json!(0), json!(0)));

    // Automático, na desistência: mapa 1 ainda no prazo e mapa 2 confirmado.
    let k = "rr-3-1";
    let (status, body) = report(&app, p1, k, 1, game("a", MAPS[0])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    map_played(&app, (p2, p1), k, 2, "b").await;
    let team = team_ids(&app, &ana, "copa", &[p2]).await[0];
    let uri = format!("{COPA}/teams/{team}/withdrawal");
    let (status, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let b = bracket(&app, &ana).await;
    assert_eq!(find(&b, k)["result"]["winner"], json!("a"));
    assert_eq!(maps_of(&b, k), (json!(0), json!(0)));
}

/// Round robin de 2 jogadores com o veto de mapas (sem draft), iniciado.
async fn with_veto_best_of(db: &PgPool, best_of: u8, extra: Value) -> (Router, Vec<User>) {
    let (app, ana, _) = quake_br(db).await;
    let p = players(&app, db, &ana, 2).await;
    let veto = with(
        json!({ "map_mode": "veto", "champion_phase": false }),
        extra,
    );
    let body = even(best_of, json!({ "veto": veto }));
    start(&app, &ana, body, &[&p[0], &p[1]]).await;
    (app, p)
}

async fn with_veto(db: &PgPool, extra: Value) -> (Router, Vec<User>) {
    with_veto_best_of(db, 2, extra).await
}

fn veto_uri(action: &str) -> String {
    format!("{COPA}/matches/rr-1-1/veto/{action}")
}

async fn both_ready(app: &Router, p: &[User]) {
    for who in p {
        let (status, body) = post_empty(app, &veto_uri("ready"), who).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
}

#[tokio::test]
async fn the_veto_bans_until_two_are_left_and_gives_each_game_its_map() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, p) = with_veto(&db, json!({})).await;
    let k = "rr-1-1";
    both_ready(&app, &p).await;

    // Pool padrão de 7: cinco bans, alternados a partir do seed melhor. O
    // último fica fora da ordem do pool, para os dois que sobram não serem os
    // dois últimos.
    let bans = [
        (&p[0], "awoken"),
        (&p[1], "molten-falls"),
        (&p[0], "corrupted-keep"),
        (&p[1], "vale-of-pnath"),
    ];
    for (who, map) in bans {
        let d = detail(&app, who, k).await;
        assert_eq!(d["veto"]["turn"]["action"], "ban");
        let (status, body) = post(&app, &veto_uri("steps"), who, json!({ "map": map })).await;
        assert_eq!(status, StatusCode::OK, "{map}: {body}");
    }
    let d = detail(&app, &p[0], k).await;
    assert_eq!(d["veto"]["games"][0]["map"], Value::Null);
    assert_eq!(d["veto"]["remaining"].as_array().unwrap().len(), 3);
    assert_eq!(
        report(&app, &p[0], k, 1, game("a", "blood-covenant")).await,
        (StatusCode::CONFLICT, err("match.veto_pending"))
    );
    let (status, d) = post(
        &app,
        &veto_uri("steps"),
        &p[0],
        json!({ "map": "ruins-of-sarnath" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{d}");

    // Sobram dois, na ordem do pool, sem quem escolheu.
    let v = &d["veto"];
    let remaining: Vec<&str> = v["remaining"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["slug"].as_str().unwrap())
        .collect();
    assert_eq!(remaining, ["blood-covenant", "deep-embrace"]);
    assert_eq!(v["games"].as_array().unwrap().len(), 2);
    assert_eq!(v["games"][0]["map"]["slug"], "blood-covenant");
    assert_eq!(v["games"][1]["map"]["slug"], "deep-embrace");
    assert_eq!(v["games"][0]["picked_by"], Value::Null);
    assert_eq!(v["games"][1]["picked_by"], Value::Null);
    assert_eq!(d["summary"]["next_map"]["slug"], "blood-covenant");

    // Cada partida no mapa dela.
    assert_eq!(
        report(&app, &p[0], k, 1, game("a", "deep-embrace")).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("match.map_mismatch"))
    );
    let (status, d) = report(&app, &p[0], k, 1, game("a", "blood-covenant")).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!(d["summary"]["next_map"]["slug"], "deep-embrace");
    assert_eq!(
        report(&app, &p[0], k, 2, game("b", "blood-covenant")).await,
        (StatusCode::UNPROCESSABLE_ENTITY, err("match.map_mismatch"))
    );
    let (status, d) = report(&app, &p[0], k, 2, game("b", "deep-embrace")).await;
    assert_eq!(status, StatusCode::OK, "{d}");
    for n in [1, 2] {
        let (status, body) = confirm(&app, &p[1], k, n).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let b = bracket(&app, &p[0]).await;
    assert_eq!(result_of(&b, k), draw());
    assert_eq!(b["status"], "finished");
}

#[tokio::test]
async fn the_veto_order_does_not_change_it_and_the_pool_can_be_just_two() {
    let (db, _lease) = crate::db::test_db().await;
    let pool = ["vestibule-of-exile", "awoken"];
    let (app, p) = with_veto(&db, json!({ "map_pool": pool, "order": "alternating" })).await;
    both_ready(&app, &p).await;
    let d = detail(&app, &p[0], "rr-1-1").await;
    // Sem o que banir, o veto já acabou. A ordem é a do pool: a do formulário.
    assert_eq!(d["veto"]["status"], "done");
    assert_eq!(d["veto"]["games"][0]["map"]["slug"], pool[0]);
    assert_eq!(d["veto"]["games"][1]["map"]["slug"], pool[1]);
}

#[tokio::test]
async fn the_pool_needs_a_map_per_game() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let too_small = |min: u8| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "tournament.map_pool_too_small", "params": { "min": min } }),
        )
    };
    let short = |best_of: u8, pool: &[&str]| {
        let veto = json!({ "map_mode": "veto", "champion_phase": false, "map_pool": pool });
        even(best_of, json!({ "veto": veto }))
    };
    let uri = "/tenants/quake-br/tournaments";
    assert_eq!(
        post(&app, uri, &ana, short(2, &MAPS[..1])).await,
        too_small(2)
    );
    assert_eq!(
        post(&app, uri, &ana, short(4, &MAPS[..3])).await,
        too_small(4)
    );
    assert_eq!(
        post(&app, uri, &ana, short(6, &MAPS[..5])).await,
        too_small(6)
    );
}

#[tokio::test]
async fn a_best_of_four_ties_only_with_all_four_maps() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started_best_of(&db, 3, 4).await;
    let (p2, p3) = (&p[1], &p[2]);
    let k = "rr-1-1";
    for (n, side) in [(1, "a"), (2, "b"), (3, "a")] {
        map_played(&app, (p2, p3), k, n, side).await;
    }
    // 2 x 1: ninguém na maioria, e o quarto pode vir.
    let d = detail(&app, &ana, k).await;
    assert_eq!(d["summary"]["state"], "ready");
    assert_eq!(d["next_game"], 4);
    map_played(&app, (p2, p3), k, 4, "b").await;

    let b = bracket(&app, &ana).await;
    assert_eq!(
        result_of(&b, k),
        (Value::Null, json!(2), json!(2), json!(false))
    );
    assert_eq!(record(&b, p2), (0, 1, 0, 0));
    assert_eq!(record(&b, p3), (0, 1, 0, 0));
    let row = table_row(&b, p2);
    assert_eq!(
        (&row["maps_won"], &row["maps_lost"], &row["map_diff"]),
        (&json!(2), &json!(2), &json!(0))
    );
    assert_eq!(
        report(&app, p2, k, 5, game("a", MAPS[4])).await,
        (StatusCode::CONFLICT, err("match.series_decided"))
    );
    let d = detail(&app, &ana, k).await;
    let closed = d["history"].as_array().unwrap().last().unwrap();
    assert_eq!(
        closed["outcome"],
        json!({ "winner": null, "walkover": false, "winner_maps": 2, "loser_maps": 2 })
    );
    let events = outbox(&db).await;
    let drawn = events
        .iter()
        .find(|e| e["announce"]["code"] == "match.series_drawn")
        .expect("o anúncio do empate");
    assert_eq!(drawn["announce"]["params"]["score"], "2-2");
}

#[tokio::test]
async fn a_best_of_four_ends_at_three_nil_and_refuses_the_next_map() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, p) = started_best_of(&db, 3, 4).await;
    let (p1, p2, p3) = (&p[0], &p[1], &p[2]);
    let k = "rr-1-1";

    // Pelos enviados, a série já acabou, mesmo sem confirmar.
    for (n, map) in [(1, MAPS[0]), (2, MAPS[1]), (3, MAPS[2])] {
        let (status, body) = report(&app, p2, k, n, game("a", map)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    assert_eq!(
        report(&app, p2, k, 4, game("a", MAPS[3])).await,
        (StatusCode::CONFLICT, err("match.series_decided"))
    );
    for n in [1, 2, 3] {
        let (status, body) = confirm(&app, p3, k, n).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let b = bracket(&app, &ana).await;
    assert_eq!(result_of(&b, k), won("a", 3, 0));
    assert_eq!(table_row(&b, p2)["map_diff"], 3);
    assert_eq!(
        report(&app, p2, k, 4, game("a", MAPS[3])).await,
        (StatusCode::CONFLICT, err("match.series_decided"))
    );

    // 3 x 1 vale menos que 3 x 0 no saldo de mapas, e o W.O. vale 3 x 0.
    series_of(&app, (p1, p3), "rr-2-1", &["b", "a", "a", "a"]).await;
    let b = bracket(&app, &ana).await;
    assert_eq!(result_of(&b, "rr-2-1"), won("a", 3, 1));
    assert_eq!(table_row(&b, p1)["map_diff"], 2);
    // P2 e P1 com saldo +1, sem se enfrentar: os mapas separam.
    assert_eq!(player(&b, &b["table"][0]["team_id"]), json!(p2.id));
    assert_eq!(b["table"][0]["ahead_by"], "map_diff");

    let (status, body) = set_result(&app, &ana, "rr-3-1", walkover("a"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let b = bracket(&app, &ana).await;
    let row = table_row(&b, p1);
    assert_eq!(
        (&row["maps_won"], &row["maps_lost"]),
        (&json!(6), &json!(1))
    );
}

#[tokio::test]
async fn the_veto_of_a_best_of_four_leaves_four_maps_in_pool_order() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, p) = with_veto_best_of(&db, 4, json!({})).await;
    let k = "rr-1-1";
    both_ready(&app, &p).await;
    // Pool padrão de 7: três bans, alternados, a partir do seed melhor.
    for (who, map) in [
        (&p[0], "blood-covenant"),
        (&p[1], "ruins-of-sarnath"),
        (&p[0], "deep-embrace"),
    ] {
        let d = detail(&app, who, k).await;
        assert_eq!(d["veto"]["turn"]["action"], "ban");
        let (status, body) = post(&app, &veto_uri("steps"), who, json!({ "map": map })).await;
        assert_eq!(status, StatusCode::OK, "{map}: {body}");
    }
    let d = detail(&app, &p[0], k).await;
    let slugs: Vec<&str> = d["veto"]["games"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| {
            assert_eq!(g["picked_by"], Value::Null);
            g["map"]["slug"].as_str().unwrap()
        })
        .collect();
    assert_eq!(
        slugs,
        ["awoken", "corrupted-keep", "molten-falls", "vale-of-pnath"]
    );
    assert_eq!(d["veto"]["status"], "done");
}

#[tokio::test]
async fn only_the_round_robin_takes_an_even_series() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let invalid = |best_of: u8| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "format.invalid_best_of", "params": { "best_of": best_of, "stage": 1 } }),
        )
    };
    let uri = "/tenants/quake-br/tournaments";
    for best_of in [2, 4, 6, 8] {
        for stages in [
            table("swiss", json!({ "best_of": best_of, "rounds": null })),
            single(best_of, 3),
            single(3, best_of),
        ] {
            let body = copa(stages);
            let got = post(&app, uri, &ana, body.clone()).await;
            assert_eq!(got, invalid(best_of), "{body}");
        }
    }
    // No round robin, qualquer N par: a md8 entra, a md0 não.
    create_tournament(&app, &ana, even(8, json!({ "slug": "liga-md8" }))).await;
    assert_eq!(post(&app, uri, &ana, even(0, json!({}))).await, invalid(0));
    // Trocar para o suíço com série par também é recusado.
    create_tournament(&app, &ana, even(4, json!({}))).await;
    assert_eq!(
        patch(
            &app,
            COPA,
            &ana,
            table("swiss", json!({ "best_of": 4, "rounds": null }))
        )
        .await,
        invalid(4)
    );
}
