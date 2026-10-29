//! O perfil de um membro na comunidade: os torneios dela em que ele se
//! inscreveu, só os que quem vê vê na lista da comunidade, com a situação e a
//! colocação dele.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};

use crate::common::*;
use crate::matches::{bracket, copa, play, players, side_player, single, start};

fn uri(of: &User) -> String {
    format!("/tenants/quake-br/members/{}", of.id)
}

async fn profile(app: &Router, viewer: &User, of: &User) -> Value {
    let (status, body) = get(app, &uri(of), viewer).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// `(torneio, situação)` de cada item do perfil, na ordem.
fn situations(profile: &Value) -> Vec<(String, String)> {
    profile["tournaments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| {
            (
                t["tournament"]["slug"].as_str().unwrap().to_owned(),
                t["situation"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// Os slugs dos torneios do perfil, na ordem.
fn slugs(profile: &Value) -> Vec<String> {
    situations(profile)
        .into_iter()
        .map(|(slug, _)| slug)
        .collect()
}

/// O organizador `by` inscreve `who` no torneio `slug`.
async fn add(app: &Router, by: &User, slug: &str, who: &User) {
    let uri = format!("/tenants/quake-br/tournaments/{slug}/participants");
    let (status, body) = post(app, &uri, by, json!({ "user_id": who.id })).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
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
async fn a_member_sees_another_members_tournaments() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;

    // `copa`, encerrado: P1 campeão, P4 cai na primeira rodada (3º–4º).
    start(
        &app,
        &ana,
        copa(single(1, 1)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    win(&app, &p[0], &p[3], "wb-1-1").await;
    win(&app, &p[1], &p[2], "wb-1-2").await;
    win(&app, &p[0], &p[1], "wb-2-1").await;
    starts_in(&db, "copa", -24 * 60).await;

    // `liga`, em andamento, com P4 na chave.
    create_tournament(
        &app,
        &ana,
        with(copa(single(1, 1)), json!({ "slug": "liga" })),
    )
    .await;
    publish(&app, &ana, "liga").await;
    register(&app, "liga", &[&p[3], &p[0]]).await;
    let (status, body) = post_empty(&app, "/tenants/quake-br/tournaments/liga/start", &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // `aberto`, com a inscrição aberta, criado por último e com início entre
    // os dois: a ordem é pelo início, não pela criação. Bia também se inscreve.
    create_tournament(&app, &ana, duel("aberto")).await;
    publish(&app, &ana, "aberto").await;
    register(&app, "aberto", &[&p[3], &bia]).await;
    starts_in(&db, "aberto", 600).await;
    // Torneio em que P4 não está: fica de fora.
    create_tournament(&app, &ana, duel("outro")).await;
    publish(&app, &ana, "outro").await;

    let p4 = profile(&app, &bia, &p[3]).await;
    assert_eq!(p4["user_id"], p[3].id);
    assert_eq!(p4["nickname"], "P4");
    assert_eq!(p4["display_name"], "P4");
    assert_eq!(p4["role"], "player");
    assert!(p4["joined_at"].is_string(), "{p4}");
    assert_eq!(
        situations(&p4),
        [
            ("liga".to_owned(), "playing".to_owned()),
            ("aberto".to_owned(), "registered".to_owned()),
            ("copa".to_owned(), "eliminated".to_owned()),
        ]
    );
    let [in_liga, in_aberto, in_copa] = [0, 1, 2].map(|i| &p4["tournaments"][i]);
    assert_eq!(in_liga["tournament"]["status"], "in_progress");
    assert_eq!(in_liga["placement"], Value::Null);
    assert_eq!(in_liga.get("next_match"), None);
    assert_eq!(in_copa["tournament"]["status"], "finished");
    assert_eq!(in_copa["placement"], json!({ "rank": 3, "tied": 2 }));
    // `registered` e `checked_in` do torneio são de quem vê, como na lista.
    assert_eq!(in_aberto["tournament"]["registered"], true);
    assert_eq!(in_copa["tournament"]["registered"], false);

    let p1 = profile(&app, &bia, &p[0]).await;
    assert_eq!(
        situations(&p1),
        [
            ("liga".to_owned(), "playing".to_owned()),
            ("copa".to_owned(), "champion".to_owned()),
        ]
    );
    assert_eq!(
        p1["tournaments"][1]["placement"],
        json!({ "rank": 1, "tied": 1 })
    );

    // No check-in, a situação é pelo check-in do membro, não de quem vê.
    starts_in(&db, "aberto", 10).await;
    let check_in = "/tenants/quake-br/tournaments/aberto/check-in";
    let (status, _) = post_empty(&app, check_in, &bia).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let aberto = &profile(&app, &bia, &p[3]).await["tournaments"][1];
    assert_eq!(aberto["situation"], "check_in_pending");
    assert_eq!(aberto["tournament"]["checked_in"], true);
    let (status, _) = post_empty(&app, check_in, &p[3]).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        profile(&app, &bia, &p[3]).await["tournaments"][1]["situation"],
        "checked_in"
    );
}

#[tokio::test]
async fn outsiders_do_not_see_profiles() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, _, bia) = quake_br(&db).await;
    let eve = user(&db, "Eve").await;
    create(&app, &eve, "da-eve").await;

    assert_eq!(
        get(&app, &uri(&bia), &eve).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    assert_eq!(
        call(&app, Method::GET, &uri(&bia), None, None).await,
        (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
    );
}

#[tokio::test]
async fn only_active_members_have_a_profile() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    let dani = member(&app, &db, &ana, "Dani").await;
    let eve = user(&db, "Eve").await;
    let not_found = (StatusCode::NOT_FOUND, err("member.not_found"));

    // Cada um vê o próprio perfil, e o owner aparece como owner.
    assert_eq!(profile(&app, &bia, &bia).await["tournaments"], json!([]));
    assert_eq!(profile(&app, &bia, &ana).await["role"], "owner");

    // Eve existe, mas não é membro; nenhum usuário tem o id 0.
    assert_eq!(get(&app, &uri(&eve), &bia).await, not_found);
    assert_eq!(
        get(&app, "/tenants/quake-br/members/0", &bia).await,
        not_found
    );
    assert_eq!(
        get(&app, "/tenants/quake-br/members/caio", &bia).await,
        (StatusCode::BAD_REQUEST, err("request.invalid_path"))
    );

    // Quem saiu e quem foi removido também.
    let (status, _) = delete(&app, "/tenants/quake-br/me", &caio).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = delete(&app, &uri(&dani), &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(get(&app, &uri(&caio), &bia).await, not_found);
    assert_eq!(get(&app, &uri(&dani), &ana).await, not_found);
}

#[tokio::test]
async fn players_see_only_what_the_tournament_list_shows() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;

    // Rascunho sem data, privado e público, criados nessa ordem; Caio inscrito
    // nos três pelo organizador.
    create_tournament(
        &app,
        &ana,
        with(duel("rascunho"), json!({ "starts_at": null })),
    )
    .await;
    add(&app, &ana, "rascunho", &caio).await;
    create_tournament(
        &app,
        &ana,
        with(duel("fechado"), json!({ "visibility": "private" })),
    )
    .await;
    publish(&app, &ana, "fechado").await;
    add(&app, &ana, "fechado", &caio).await;
    create_tournament(&app, &ana, duel("aberto")).await;
    publish(&app, &ana, "aberto").await;
    add(&app, &ana, "aberto", &caio).await;

    // Bia, player, não vê o rascunho nem o privado em que não está.
    assert_eq!(slugs(&profile(&app, &bia, &caio).await), ["aberto"]);
    // Caio vê o privado em que está, mas não o rascunho.
    assert_eq!(
        slugs(&profile(&app, &caio, &caio).await),
        ["aberto", "fechado"]
    );
    // O owner vê todos: o sem data primeiro, depois, com o mesmo início, o
    // criado por último.
    let all = profile(&app, &ana, &caio).await;
    assert_eq!(
        situations(&all),
        [
            ("rascunho".to_owned(), "registered".to_owned()),
            ("aberto".to_owned(), "registered".to_owned()),
            ("fechado".to_owned(), "registered".to_owned()),
        ]
    );
}
