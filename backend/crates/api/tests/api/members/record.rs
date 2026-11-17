//! As abas da página de um membro: a carreira (`.../career`), os mapas
//! paginados e filtrados (`.../games`) e o confronto direto por adversário
//! (`.../opponents`), de todos os torneios da comunidade, com rating ligado
//! ou não, no duelo e no TDM.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{
    COPA, MAPS, bracket, confirm, copa, game, play, players, report, set_game, set_result,
    side_player, single, start, table, walkover,
};
use crate::stages::{linked, play_out, wins};
use crate::teams::{started, tdm};

const LIGA: &str = "/tenants/quake-br/tournaments/liga";

fn uri(of: &User, tab: &str) -> String {
    format!("/tenants/quake-br/members/{}/{tab}", of.id)
}

async fn tab(app: &Router, viewer: &User, of: &User, tab: &str) -> Value {
    let (status, body) = get(app, &uri(of, tab), viewer).await;
    assert_eq!(status, StatusCode::OK, "{tab}: {body}");
    body
}

/// Cria, publica e inicia `liga` (duelo, eliminação simples, a série em
/// md`best_of`) com `who`, sem ordem da chave, com `changes`.
async fn liga(app: &Router, ana: &User, best_of: u8, who: &[&User], changes: Value) {
    let body = with(
        with(copa(single(best_of, best_of)), json!({ "slug": "liga" })),
        changes,
    );
    create_tournament(app, ana, body).await;
    publish(app, ana, "liga").await;
    register(app, "liga", who).await;
    let (status, body) = post_empty(app, &format!("{LIGA}/start"), ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

/// O lado (`"a"` ou `"b"`) de `who` na série `key` do torneio em `base`.
async fn side_of(app: &Router, base: &str, who: &User, key: &str) -> &'static str {
    let (status, b) = get(app, &format!("{base}/bracket"), who).await;
    assert_eq!(status, StatusCode::OK, "{b}");
    if side_player(&b, key, "a") == who.id {
        "a"
    } else {
        "b"
    }
}

/// `winner` vence os mapas `maps` da série `key` de `base`, em ordem, contra
/// `loser`, que confirma cada um; antes, `loser` vence os `lost` primeiros
/// mapas da lista.
async fn games_in(
    app: &Router,
    base: &str,
    (winner, loser): (&User, &User),
    key: &str,
    maps: &[String],
    lost: usize,
) {
    let side = side_of(app, base, winner, key).await;
    let other = if side == "a" { "b" } else { "a" };
    for (i, map) in maps.iter().enumerate() {
        let n = i + 1;
        let (by, to, won_by) = if i < lost {
            (loser, winner, other)
        } else {
            (winner, loser, side)
        };
        let path = format!("{base}/matches/{key}/games/{n}");
        let (status, body) = post(app, &format!("{path}/report"), by, game(won_by, map)).await;
        assert_eq!(status, StatusCode::OK, "{key} {n}: {body}");
        let (status, body) = post_empty(app, &format!("{path}/confirm"), to).await;
        assert_eq!(status, StatusCode::OK, "{key} {n}: {body}");
    }
}

/// Os slugs de todos os mapas ativos.
async fn all_maps(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar!(r#"select slug from maps where active order by slug collate "C""#)
        .fetch_all(db)
        .await
        .unwrap()
}

/// `winner` vence `loser` na série `key` (md1) de `copa`.
async fn win(app: &Router, winner: &User, loser: &User, key: &str) {
    let (a, b) = if side_player(&bracket(app, winner).await, key, "a") == winner.id {
        (1, 0)
    } else {
        (0, 1)
    };
    play(app, winner, loser, key, a, b).await;
}

#[tokio::test]
async fn career_has_series_maps_titles_and_the_streak() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // Fora do rating: a carreira conta do mesmo jeito. Semifinais em md1 e a
    // final em md3: P1 vence P4 e depois P2 por 2 x 1.
    let body = copa(with(single(1, 3), json!({ "counts_for_rating": false })));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2], &p[3]]).await;
    let empty = tab(&app, &bia, &p[0], "career").await;
    assert_eq!(
        empty["series"],
        json!({ "wins": 0, "losses": 0, "draws": 0 })
    );
    assert_eq!(empty["streak"], Value::Null);
    assert_eq!(empty["best_placement"], Value::Null);
    assert_eq!(empty["recent_series"], json!([]));

    win(&app, &p[0], &p[3], "wb-1-1").await;
    win(&app, &p[1], &p[2], "wb-1-2").await;
    let maps = all_maps(&db).await;
    games_in(&app, COPA, (&p[0], &p[1]), "wb-2-1", &maps[..3], 1).await;

    let c = tab(&app, &bia, &p[0], "career").await;
    assert_eq!(c["series"], json!({ "wins": 2, "losses": 0, "draws": 0 }));
    assert_eq!(c["games"], json!({ "wins": 3, "losses": 1 }));
    // Cada mapa vencido é 10 x 5.
    assert_eq!(
        (&c["frags_for"], &c["frags_against"]),
        (&json!(35), &json!(25))
    );
    assert_eq!((&c["titles"], &c["podiums"]), (&json!(1), &json!(1)));
    assert_eq!(c["best_placement"], json!({ "rank": 1, "tied": 1 }));
    assert_eq!(c["streak"], json!({ "won": true, "count": 2 }));
    let recent = c["recent_series"].as_array().unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0]["match"], "wb-2-1");
    assert_eq!(recent[0]["outcome"], "won");
    assert_eq!(
        (&recent[0]["wins"], &recent[0]["losses"]),
        (&json!(2), &json!(1))
    );
    assert_eq!(recent[0]["opponent"]["members"][0]["user_id"], p[1].id);
    assert_eq!(recent[0]["tournament"]["slug"], "copa");
    assert_eq!(recent[1]["match"], "wb-1-1");

    // Quem caiu na semifinal: o 3º–4º é pódio.
    let c = tab(&app, &bia, &p[3], "career").await;
    assert_eq!(c["series"], json!({ "wins": 0, "losses": 1, "draws": 0 }));
    assert_eq!(c["streak"], json!({ "won": false, "count": 1 }));
    assert_eq!((&c["titles"], &c["podiums"]), (&json!(0), &json!(1)));
    assert_eq!(c["best_placement"], json!({ "rank": 3, "tied": 2 }));
}

/// Quantos mapas de `of` o filtro `query` acha, como `viewer` vê.
async fn total_of(app: &Router, viewer: &User, of: &User, query: String) -> i64 {
    let (status, body) = get(app, &format!("{}?{query}", uri(of, "games")), viewer).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["total"].as_i64().unwrap()
}

#[tokio::test]
async fn games_are_paged_filtered_and_show_the_rating_where_it_counted() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 2).await;
    let maps = all_maps(&db).await;
    // `copa`, com rating: a final em md19, P1 vence por 10 x 9 (19 mapas).
    start(&app, &ana, copa(single(19, 19)), &[&p[0], &p[1]]).await;
    games_in(&app, COPA, (&p[0], &p[1]), "wb-1-1", &maps[..19], 9).await;
    // `liga`, sem rating: P1 vence por 8 x 0, depois.
    let off = json!({ "counts_for_rating": false });
    liga(&app, &ana, 15, &[&p[0], &p[1]], off).await;
    games_in(&app, LIGA, (&p[0], &p[1]), "wb-1-1", &maps[..8], 0).await;

    let page = tab(&app, &bia, &p[0], "games").await;
    assert_eq!(page["total"], 27);
    assert_eq!((&page["page"], &page["page_size"]), (&json!(1), &json!(25)));
    let games = page["games"].as_array().unwrap();
    assert_eq!(games.len(), 25);
    // Do mais recente: o último mapa da liga, sem rating.
    let last = &games[0];
    assert_eq!(last["tournament"]["slug"], "liga");
    assert_eq!(last["game"], 8);
    assert_eq!(
        (&last["match"], &last["single"]),
        (&json!("wb-1-1"), &json!(true))
    );
    assert_eq!(last["map"]["slug"], maps[7].as_str());
    assert_eq!(last["won"], true);
    assert_eq!(
        (&last["frags"], &last["opponent_frags"]),
        (&json!(10), &json!(5))
    );
    assert_eq!(last["rating"], Value::Null);
    assert_eq!(last["opponent"]["members"][0]["user_id"], p[1].id);
    // Os da copa contaram: o ± e o rating depois.
    let rated = &games[8];
    assert_eq!(rated["tournament"]["slug"], "copa");
    assert_eq!(rated["game"], 19);
    assert!(rated["rating"]["delta"].as_i64().unwrap() > 0, "{rated}");
    assert!(rated["rating"]["rating_after"].is_i64(), "{rated}");

    let (status, second) = get(&app, &format!("{}?page=2", uri(&p[0], "games")), &bia).await;
    assert_eq!(status, StatusCode::OK, "{second}");
    let rest = second["games"].as_array().unwrap();
    assert_eq!(rest.len(), 2);
    // O primeiro mapa da copa, perdido.
    assert_eq!(
        (&rest[1]["game"], &rest[1]["won"]),
        (&json!(1), &json!(false))
    );
    // Além do fim: vazia, com o total.
    let (status, beyond) = get(&app, &format!("{}?page=9", uri(&p[0], "games")), &bia).await;
    assert_eq!(status, StatusCode::OK, "{beyond}");
    assert_eq!(
        (&beyond["total"], &beyond["games"]),
        (&json!(27), &json!([]))
    );

    // Filtros: torneio, mapa, adversário.
    assert_eq!(
        total_of(&app, &bia, &p[0], "tournament=liga".into()).await,
        8
    );
    assert_eq!(
        total_of(&app, &bia, &p[0], format!("map={}", maps[0])).await,
        2
    );
    assert_eq!(
        total_of(&app, &bia, &p[0], format!("map={}", maps[19])).await,
        0
    );
    assert_eq!(
        total_of(&app, &bia, &p[0], format!("opponent={}", p[1].id)).await,
        27
    );
    assert_eq!(
        total_of(&app, &bia, &p[0], format!("opponent={}", bia.id)).await,
        0
    );
    assert_eq!(
        total_of(
            &app,
            &bia,
            &p[0],
            format!("tournament=copa&map={}", maps[8])
        )
        .await,
        1
    );

    // As opções dos filtros são de todos os mapas da pessoa.
    let filters = &page["filters"];
    let slugs: Vec<_> = filters["tournaments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["slug"].as_str().unwrap())
        .collect();
    assert_eq!(slugs, ["liga", "copa"]);
    assert_eq!(filters["maps"].as_array().unwrap().len(), 19);
    assert_eq!(filters["opponents"][0]["user_id"], p[1].id);
    assert_eq!(filters["opponents"][0]["active"], true);
    assert_eq!(filters["opponents"].as_array().unwrap().len(), 1);

    let (status, body) = get(&app, &format!("{}?page=0", uri(&p[0], "games")), &bia).await;
    assert_eq!(
        (status, body),
        (StatusCode::BAD_REQUEST, err("request.invalid_query"))
    );
}

#[tokio::test]
async fn opponents_join_duel_and_tdm_and_count_walkovers() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // TDM: P1 e P2 contra P3 e P4; o time de P1 vence o único mapa.
    started(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let (status, body) = post(
        &app,
        &format!("{COPA}/matches/wb-1-1/games/1/report"),
        &p[1],
        game("a", "awoken"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(
        &app,
        &format!("{COPA}/matches/wb-1-1/games/1/confirm"),
        &p[3],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Duelo, sem rating: P3 vence P1 por W.O., sem mapa.
    liga(
        &app,
        &ana,
        1,
        &[&p[0], &p[2]],
        json!({ "counts_for_rating": false }),
    )
    .await;
    let side = side_of(&app, LIGA, &p[2], "wb-1-1").await;
    let (status, body) = put(
        &app,
        &format!("{LIGA}/matches/wb-1-1/result"),
        &ana,
        json!({ "result": { "kind": "walkover", "winner": side }, "comment": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let rows = |body: Value| -> Vec<(i64, Value, Value)> {
        body["opponents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| {
                (
                    o["person"]["user_id"].as_i64().unwrap(),
                    o["series"].clone(),
                    o["games"].clone(),
                )
            })
            .collect()
    };
    let record = |w: i32, l: i32| json!({ "wins": w, "losses": l, "draws": 0 });
    let maps = |w: i32, l: i32| json!({ "wins": w, "losses": l });

    // Todas as modalidades: P3 em duas séries (uma no TDM, uma no duelo).
    let all = tab(&app, &p[1], &p[0], "opponents").await;
    assert_eq!(
        rows(all.clone()),
        [
            (p[2].id, record(1, 1), maps(1, 0)),
            (p[3].id, record(1, 0), maps(1, 0)),
        ]
    );
    assert_eq!(all["opponents"][0]["person"]["nickname"], "P3");
    assert_eq!(all["opponents"][0]["person"]["active"], true);
    assert!(all["opponents"][0]["last_at"].is_string());

    let duel = tab(&app, &p[1], &p[0], "opponents?modality=duel").await;
    assert_eq!(rows(duel), [(p[2].id, record(0, 1), maps(0, 0))]);
    let teams = tab(&app, &p[1], &p[0], "opponents?modality=tdm").await;
    assert_eq!(rows(teams).len(), 2);
    // O colega de time não é adversário.
    let of_p2 = tab(&app, &p[0], &p[1], "opponents").await;
    assert!(rows(of_p2).iter().all(|(id, ..)| *id != p[0].id));

    // A carreira conta o W.O. como série, sem mapa.
    let c = tab(&app, &p[1], &p[0], "career").await;
    assert_eq!(c["series"], record(1, 1));
    assert_eq!(c["games"], maps(1, 0));
    assert_eq!(c["recent_series"][0]["outcome"], "lost");
    assert_eq!(c["recent_series"][0]["wins"], Value::Null);

    let (status, body) = get(&app, &uri(&p[0], "opponents?modality=ffa"), &p[1]).await;
    assert_eq!(
        (status, body),
        (StatusCode::BAD_REQUEST, err("request.invalid_query"))
    );
}

#[tokio::test]
async fn only_members_see_the_tabs_of_active_members() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    let eve = user(&db, "Eve").await;
    create(&app, &eve, "da-eve").await;
    let (status, _) = delete(&app, "/tenants/quake-br/me", &caio).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    for t in ["tournaments", "career", "games", "opponents"] {
        assert_eq!(
            get(&app, &uri(&bia, t), &eve).await,
            (StatusCode::NOT_FOUND, err("tenant.not_found")),
            "{t}"
        );
        assert_eq!(
            call(&app, Method::GET, &uri(&bia, t), None, None).await,
            (StatusCode::UNAUTHORIZED, err("auth.unauthenticated")),
            "{t}"
        );
        assert_eq!(
            get(&app, &uri(&caio, t), &bia).await,
            (StatusCode::NOT_FOUND, err("member.not_found")),
            "{t}"
        );
        // O próprio perfil, sem nada jogado.
        let (status, _) = get(&app, &uri(&bia, t), &bia).await;
        assert_eq!(status, StatusCode::OK, "{t}");
    }
}

#[tokio::test]
async fn a_later_stage_keeps_its_prefix_and_a_correction_keeps_the_order() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    // Todos contra todos e a disputa do 1º com o 2º; P1 vence as três.
    start(&app, &ana, linked(json!({ "advance": 2 })), &all).await;
    play_out(&app, &p).await;
    let before = tab(&app, &bia, &p[0], "career").await["recent_series"].clone();
    let keys = |series: &Value| -> Vec<String> {
        series
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["match"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(keys(&before).len(), 3);

    // A correção que vira a série mais antiga (contra P4) a regrava, mas ela
    // continua no lugar dela: a sequência é das duas mais recentes.
    let oldest = keys(&before)[2].clone();
    let side = side_of(&app, COPA, &p[0], &oldest).await;
    let (a_frags, b_frags) = if side == "a" { (3, 20) } else { (20, 3) };
    let frags = json!({ "map": MAPS[0], "a_frags": a_frags, "b_frags": b_frags });
    let (code, body) = set_game(&app, &ana, &oldest, 1, frags, None).await;
    assert_eq!(code, StatusCode::OK, "{body}");
    let after = tab(&app, &bia, &p[0], "career").await;
    assert_eq!(keys(&after["recent_series"]), keys(&before));
    assert_eq!(after["recent_series"][2]["outcome"], "lost");
    assert_eq!(after["streak"], json!({ "won": true, "count": 2 }));

    // A série e o mapa da fase 2 saem com o código público, que abre a série.
    let (code, s) = post_empty(&app, &format!("{COPA}/stages/2/start"), &ana).await;
    assert_eq!(code, StatusCode::OK, "{s}");
    wins(&app, &p[0], &p[1], "s2-pl-1-1").await;
    let c = tab(&app, &bia, &p[0], "career").await;
    assert_eq!(c["recent_series"][0]["match"], "s2-pl-1-1");
    assert_eq!(c["series"], json!({ "wins": 3, "losses": 1, "draws": 0 }));
    let g = tab(&app, &bia, &p[0], "games").await;
    assert_eq!(
        (&g["games"][0]["match"], &g["games"][0]["single"]),
        (&json!("s2-pl-1-1"), &json!(false))
    );
    assert!(g["games"][1]["match"].as_str().unwrap().starts_with("rr-"));
    let (status, d) = get(&app, &format!("{COPA}/matches/s2-pl-1-1"), &bia).await;
    assert_eq!((status, &d["stage"]), (StatusCode::OK, &json!(2)), "{d}");
}

#[tokio::test]
async fn a_drawn_series_breaks_the_streak_and_counts_against_the_opponent() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    let all: Vec<&User> = p.iter().collect();
    // Round robin md2: P1 x P2 em `rr-3-1`, um mapa para cada lado.
    let body = copa(table("round_robin", json!({ "best_of": 2 })));
    start(&app, &ana, body, &all).await;
    for (n, won_by) in [(1, "a"), (2, "b")] {
        let map = MAPS[usize::from(n) - 1];
        let (code, body) = report(&app, &p[0], "rr-3-1", n, game(won_by, map)).await;
        assert_eq!(code, StatusCode::OK, "{body}");
        let (code, body) = confirm(&app, &p[1], "rr-3-1", n).await;
        assert_eq!(code, StatusCode::OK, "{body}");
    }
    let c = tab(&app, &bia, &p[0], "career").await;
    assert_eq!(c["series"], json!({ "wins": 0, "losses": 0, "draws": 1 }));
    assert_eq!(c["games"], json!({ "wins": 1, "losses": 1 }));
    assert_eq!(c["streak"], Value::Null);
    assert_eq!(c["recent_series"][0]["outcome"], "draw");
    let o = tab(&app, &bia, &p[0], "opponents").await;
    assert_eq!(o["opponents"][0]["person"]["user_id"], p[1].id);
    assert_eq!(
        o["opponents"][0]["series"],
        json!({ "wins": 0, "losses": 0, "draws": 1 })
    );
}

#[tokio::test]
async fn a_tdm_walkover_counts_against_the_other_team_and_who_left_stays() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // TDM: P1 e P2 contra P3 e P4, decidido por W.O., sem mapa.
    started(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let (status, body) = set_result(&app, &ana, "wb-1-1", walkover("a"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _) = delete(&app, "/tenants/quake-br/me", &p[2]).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let o = tab(&app, &bia, &p[0], "opponents").await;
    let mut rows: Vec<(i64, Value, Value, Value)> = o["opponents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["person"]["user_id"].as_i64().unwrap(),
                r["person"]["active"].clone(),
                r["series"].clone(),
                r["games"].clone(),
            )
        })
        .collect();
    rows.sort_by_key(|r| r.0);
    let career = tab(&app, &bia, &p[0], "career").await;
    let series = if career["recent_series"][0]["outcome"] == "won" {
        json!({ "wins": 1, "losses": 0, "draws": 0 })
    } else {
        json!({ "wins": 0, "losses": 1, "draws": 0 })
    };
    let none = json!({ "wins": 0, "losses": 0 });
    assert_eq!(
        rows,
        [
            (p[2].id, json!(false), series.clone(), none.clone()),
            (p[3].id, json!(true), series, none),
        ]
    );
}
