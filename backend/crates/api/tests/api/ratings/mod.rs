//! Testes de `ratings`: o ranking e o rating de um membro pelos mapas
//! (aqui), os ajustes manuais (`adjustments`), a configuração do Elo
//! (`settings`) e o recálculo quando o passado muda, pelo agendador e pela
//! leitura (`refresh`). As regras do banco ficam em `schema::ratings`.

mod adjustments;
mod refresh;
mod settings;

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{
    COPA, confirm, contest, copa, game, play, report, set_result, single, start, walkover,
};

pub const RATINGS: &str = "/tenants/quake-br/ratings";

/// `quake-br` com Ana (owner), Bia (player sem inscrição) e a final única
/// `copa` em md3 entre `p1` (lado a) e `p2` (lado b), iniciada.
pub struct Final {
    pub app: Router,
    pub ana: User,
    pub bia: User,
    pub p1: User,
    pub p2: User,
}

pub async fn final_md3(db: &PgPool, changes: Value) -> Final {
    let (app, ana, bia) = quake_br(db).await;
    let p1 = member(&app, db, &ana, "P1").await;
    let p2 = member(&app, db, &ana, "P2").await;
    let body = with(copa(single(3, 3)), changes);
    start(&app, &ana, body, &[&p1, &p2]).await;
    Final {
        app,
        ana,
        bia,
        p1,
        p2,
    }
}

pub async fn ranking(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, RATINGS, who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// `(user_id, rating, games, wins, losses, provisional)` de cada um, na ordem.
pub fn table(ranking: &Value) -> Vec<(i64, i64, i64, i64, i64, bool)> {
    ranking["players"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["user_id"].as_i64().unwrap(),
                p["rating"].as_i64().unwrap(),
                p["games"].as_i64().unwrap(),
                p["wins"].as_i64().unwrap(),
                p["losses"].as_i64().unwrap(),
                p["provisional"].as_bool().unwrap(),
            )
        })
        .collect()
}

pub async fn member_rating(app: &Router, who: &User, of: i64) -> Value {
    let (status, body) = get(app, &format!("{RATINGS}/{of}"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[tokio::test]
async fn each_confirmed_map_is_a_game() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    let empty = ranking(&f.app, &f.bia).await;
    assert_eq!(empty["initial"], 1500);
    assert_eq!(empty["provisional_games"], 10);
    assert_eq!(empty["players"], json!([]));

    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    // Primeiro mapa: os dois novos, K 80, +40/-40. Segundo: K 75,4 com um
    // jogo cada, e o favorito ganha 29.
    let r = ranking(&f.app, &f.bia).await;
    assert_eq!(
        table(&r),
        [
            (f.p1.id, 1569, 2, 2, 0, true),
            (f.p2.id, 1431, 2, 0, 2, true)
        ]
    );
    assert_eq!(r["players"][0]["position"], 1);
    assert_eq!(r["players"][1]["position"], 2);
    assert_eq!(r["players"][0]["nickname"], "P1");
    assert!(r["players"][0]["last_game_at"].is_string());
}

#[tokio::test]
async fn maps_waiting_or_contested_do_not_count() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    let (status, body) = report(&f.app, &f.p1, "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ranking(&f.app, &f.bia).await["players"], json!([]));
    let (status, body) = contest(&f.app, &f.p2, "wb-1-1", 1, "não foi").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(ranking(&f.app, &f.bia).await["players"], json!([]));
}

#[tokio::test]
async fn walkover_does_not_count_but_the_maps_before_it_do() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    let (status, body) = report(&f.app, &f.p1, "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&f.app, &f.p2, "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = set_result(&f.app, &f.ana, "wb-1-1", walkover("b"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // O mapa jogado conta; o W.O. que deu a série ao P2, não.
    assert_eq!(
        table(&ranking(&f.app, &f.bia).await),
        [
            (f.p1.id, 1540, 1, 1, 0, true),
            (f.p2.id, 1460, 1, 0, 1, true)
        ]
    );
}

#[tokio::test]
async fn tournament_that_does_not_count() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({ "counts_for_rating": false })).await;
    let (status, t) = get(&f.app, COPA, &f.bia).await;
    assert_eq!(status, StatusCode::OK, "{t}");
    assert_eq!(t["counts_for_rating"], false);
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 1).await;
    assert_eq!(ranking(&f.app, &f.bia).await["players"], json!([]));
    // Depois de começar, não muda mais.
    let (status, body) = patch(&f.app, COPA, &f.ana, json!({ "counts_for_rating": true })).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("tournament.setup_locked"))
    );
}

#[tokio::test]
async fn counts_for_rating_changes_until_the_start() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let created = create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(created["counts_for_rating"], true);
    publish(&app, &ana, "copa").await;
    let (status, body) = patch(&app, COPA, &ana, json!({ "counts_for_rating": false })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["counts_for_rating"], false);
}

#[tokio::test]
async fn member_rating_with_the_last_maps() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    let r = member_rating(&f.app, &f.bia, f.p2.id).await;
    assert_eq!(r["standing"]["rating"], 1431);
    assert_eq!(r["standing"]["position"], 2);
    assert_eq!(r["adjusted"], 0);
    assert_eq!(r["adjustments"], Value::Null);
    let games = r["games"].as_array().unwrap();
    assert_eq!(games.len(), 2);
    // O mais recente primeiro.
    assert_eq!(games[0]["game"], 2);
    assert_eq!(games[0]["match"], "wb-1-1");
    assert_eq!(games[0]["map"], "Exile");
    assert_eq!(games[0]["won"], false);
    assert_eq!(games[0]["delta"], -29);
    assert_eq!(games[0]["rating_after"], 1431);
    // O adversário é o time do outro lado; no duelo, de um jogador.
    let opponent = &games[0]["opponent"]["members"][0];
    assert_eq!(opponent["user_id"], f.p1.id);
    assert_eq!(opponent["nickname"], "P1");
    assert_eq!(opponent["left"], false);
    assert_eq!(games[0]["tournament"]["slug"], "copa");
    assert_eq!(games[1]["delta"], -40);

    // Quem ainda não jogou: sem rating e sem mapas.
    let none = member_rating(&f.app, &f.p1, f.bia.id).await;
    assert_eq!(none["standing"], Value::Null);
    assert_eq!(none["initial"], 1500);
    assert_eq!(none["games"], json!([]));

    let (status, body) = get(&f.app, &format!("{RATINGS}/999999"), &f.bia).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
}

#[tokio::test]
async fn maps_of_a_private_tournament_only_for_who_sees_it() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    let (status, body) = patch(&f.app, COPA, &f.ana, json!({ "visibility": "private" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    // Bia não vê o torneio: vê o rating, não os mapas.
    let r = member_rating(&f.app, &f.bia, f.p1.id).await;
    assert_eq!(r["standing"]["rating"], 1569);
    assert_eq!(r["games"], json!([]));
    for who in [&f.p2, &f.ana] {
        let r = member_rating(&f.app, who, f.p1.id).await;
        assert_eq!(r["games"].as_array().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn who_left_leaves_the_ranking() {
    let (db, _lease) = crate::db::test_db().await;
    let f = final_md3(&db, json!({})).await;
    play(&f.app, &f.p1, &f.p2, "wb-1-1", 2, 0).await;
    let (status, body) = delete(&f.app, "/tenants/quake-br/me", &f.p2).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    // Os mapas dele continuam no rating dos outros.
    assert_eq!(
        table(&ranking(&f.app, &f.bia).await),
        [(f.p1.id, 1569, 2, 2, 0, true)]
    );
    let r = member_rating(&f.app, &f.bia, f.p1.id).await;
    assert_eq!(r["games"][0]["opponent"]["members"][0]["left"], true);
    let (status, body) = get(&f.app, &format!("{RATINGS}/{}", f.p2.id), &f.bia).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
}

#[tokio::test]
async fn only_members_see_the_ranking() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, _) = quake_br(&db).await;
    let outsider = user(&db, "Zé").await;
    let (status, body) = get(&app, RATINGS, &outsider).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
}
