//! O retrospecto por mapa (a arena) na página da pessoa (`by_map`): os mapas
//! confirmados de todos os torneios da comunidade, com ou sem rating, no duelo
//! e no TDM, da arena mais jogada à menos.

use axum::http::StatusCode;
use serde_json::json;

use super::*;
use crate::common::*;
use crate::matches::{
    confirm, copa, detail, favorite_wins, game, play, players, report, set_game, set_result,
    single, start, walkover,
};
use crate::teams::{started, tdm};

/// `(mapa, vitórias, derrotas)` de `of` na página dele, como `who` a vê, na
/// ordem da resposta.
async fn records(app: &Router, who: &User, of: &User) -> Vec<(String, i64, i64)> {
    member_rating(app, who, of.id).await["by_map"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["slug"].as_str().unwrap().to_owned(),
                m["wins"].as_i64().unwrap(),
                m["losses"].as_i64().unwrap(),
            )
        })
        .collect()
}

fn record(slug: &str, wins: i64, losses: i64) -> (String, i64, i64) {
    (slug.to_owned(), wins, losses)
}

#[tokio::test]
async fn confirmed_maps_count_by_map_even_without_rating() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    // Fora do rating: o retrospecto por mapa conta do mesmo jeito.
    let body = copa(with(single(1, 3), json!({ "counts_for_rating": false })));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2], &p[3]]).await;
    assert_eq!(records(&app, &bia, &p[0]).await, []);

    // Semifinais em md1, as duas no primeiro mapa da lista (awoken).
    favorite_wins(&app, &p, "wb-1-1", 1).await;
    favorite_wins(&app, &p, "wb-1-2", 1).await;
    assert_eq!(records(&app, &bia, &p[0]).await, [record("awoken", 1, 0)]);
    assert_eq!(records(&app, &bia, &p[3]).await, [record("awoken", 0, 1)]);
    assert_eq!(ranking(&app, &bia).await["players"], json!([]));

    // Final md3, P1 (lado a) contra P2 (lado b). Mapa 1: P2 vence no awoken.
    let (status, body) = report(&app, &p[1], "wb-2-1", 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[0], "wb-2-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Mapa 2: P1 vence no exile.
    let (status, body) = report(&app, &p[0], "wb-2-1", 2, game("a", "exile")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[1], "wb-2-1", 2).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Do mais jogado ao menos.
    assert_eq!(
        records(&app, &bia, &p[0]).await,
        [record("awoken", 1, 1), record("exile", 1, 0)]
    );
    assert_eq!(
        records(&app, &bia, &p[1]).await,
        [record("awoken", 2, 0), record("exile", 0, 1)]
    );

    // Mapa 3 enviado e ainda sem resposta: não conta. Confirmado, conta, e o
    // empate de partidas desempata pelo nome.
    let (status, body) = report(&app, &p[0], "wb-2-1", 3, game("a", "insomnia")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(records(&app, &bia, &p[0]).await.len(), 2);
    let (status, body) = confirm(&app, &p[1], "wb-2-1", 3).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        records(&app, &bia, &p[0]).await,
        [
            record("awoken", 1, 1),
            record("exile", 1, 0),
            record("insomnia", 1, 0)
        ]
    );
    // Quem perdeu a semifinal só tem o mapa dela.
    assert_eq!(records(&app, &bia, &p[2]).await, [record("awoken", 0, 1)]);

    let first = &member_rating(&app, &bia, p[0].id).await["by_map"][0];
    assert_eq!(
        (&first["slug"], &first["name"]),
        (&json!("awoken"), &json!("Awoken"))
    );
}

#[tokio::test]
async fn a_walkover_adds_no_map() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    let (status, body) = report(&f.app, &f.p1, "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&f.app, &f.p2, "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // O W.O. decide a série sem mapa: só o jogado conta, para os dois lados.
    let (status, body) = set_result(&f.app, &f.ana, "wb-1-1", walkover("b"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        records(&f.app, &f.bia, &f.p1).await,
        [record("awoken", 1, 0)]
    );
    assert_eq!(
        records(&f.app, &f.bia, &f.p2).await,
        [record("awoken", 0, 1)]
    );
}

#[tokio::test]
async fn a_confirmed_map_voided_by_a_correction_stops_counting() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    play(&app, &p[0], &p[3], "wb-1-1", 2, 0).await;
    play(&app, &p[1], &p[2], "wb-1-2", 2, 1).await;
    // Um mapa da final (P1 x P2) já confirmado: P2 vence no awoken.
    let (status, body) = report(&app, &p[1], "wb-2-1", 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[0], "wb-2-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        records(&app, &bia, &p[0]).await,
        [record("awoken", 1, 1), record("exile", 1, 0)]
    );
    assert_eq!(
        records(&app, &bia, &p[1]).await,
        [
            record("awoken", 1, 1),
            record("exile", 1, 0),
            record("insomnia", 1, 0)
        ]
    );

    // Corrigir o mapa 1 da semifinal reabre a série, e a final perde o lado
    // de P1: o mapa dela é anulado (apagado) e deixa de contar para os dois.
    let (status, body) = set_game(&app, &ana, "wb-1-1", 1, game("b", "awoken"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let final_ = detail(&app, &p[1], "wb-2-1").await;
    assert_eq!(final_["games"], json!([]));
    assert_eq!(
        records(&app, &bia, &p[0]).await,
        [record("awoken", 0, 1), record("exile", 1, 0)]
    );
    assert_eq!(
        records(&app, &bia, &p[1]).await,
        [
            record("awoken", 0, 1),
            record("exile", 1, 0),
            record("insomnia", 1, 0)
        ]
    );
}

#[tokio::test]
async fn a_tdm_map_is_a_record_for_each_player_in_the_lineup() {
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
    // O time A (P1 e P2) vence o único mapa da série, no awoken.
    let (status, body) = report(&app, &p[1], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[3], "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for won in [&p[0], &p[1]] {
        assert_eq!(records(&app, &ana, won).await, [record("awoken", 1, 0)]);
    }
    for lost in [&p[2], &p[3]] {
        assert_eq!(records(&app, &ana, lost).await, [record("awoken", 0, 1)]);
    }
}

#[tokio::test]
async fn only_the_maps_of_this_community_count() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;

    // Os dois também estão numa segunda comunidade, com uma série num mapa que
    // não saiu na primeira.
    create(&f.app, &f.ana, "argentina").await;
    let token = invite(&f.app, &f.ana, "argentina").await;
    join(&f.app, &f.p1, &token).await;
    let token = invite(&f.app, &f.ana, "argentina").await;
    join(&f.app, &f.p2, &token).await;
    let liga = "/tenants/argentina/tournaments/liga";
    let body = with(copa(single(1, 1)), json!({ "slug": "liga" }));
    let (status, created) = post(&f.app, "/tenants/argentina/tournaments", &f.ana, body).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let (status, body) = post_empty(&f.app, &format!("{liga}/publish"), &f.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for who in [&f.p1, &f.p2] {
        let (status, body) = post_empty(&f.app, &format!("{liga}/registration"), who).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    }
    let (status, list) = get(&f.app, &format!("{liga}/participants"), &f.ana).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let team = |who: &User| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|p| p["user_id"] == who.id)
            .unwrap()["team_id"]
            .clone()
    };
    let order = json!({ "team_ids": [team(&f.p1), team(&f.p2)] });
    let (status, body) = put(&f.app, &format!("{liga}/seeding"), &f.ana, order).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(&f.app, &format!("{liga}/start"), &f.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let game_uri = format!("{liga}/matches/wb-1-1/games/1");
    let (status, body) = post(
        &f.app,
        &format!("{game_uri}/report"),
        &f.p2,
        game("b", "lockbox"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(&f.app, &format!("{game_uri}/confirm"), &f.p1).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let of = |slug: &str| format!("/tenants/{slug}/ratings/{}", f.p2.id);
    let (status, quake) = get(&f.app, &of("quake-br"), &f.ana).await;
    assert_eq!(status, StatusCode::OK, "{quake}");
    assert_eq!(
        quake["by_map"],
        json!([
            { "slug": "awoken", "name": "Awoken", "wins": 0, "losses": 1 },
            { "slug": "exile", "name": "Exile", "wins": 0, "losses": 1 }
        ])
    );
    let (status, argentina) = get(&f.app, &of("argentina"), &f.ana).await;
    assert_eq!(status, StatusCode::OK, "{argentina}");
    assert_eq!(
        argentina["by_map"],
        json!([{ "slug": "lockbox", "name": "Lockbox", "wins": 1, "losses": 0 }])
    );
}
