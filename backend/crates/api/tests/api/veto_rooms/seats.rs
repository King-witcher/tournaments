//! Entrar num lugar, logado e com nome, e o que cada link pode fazer.

use axum::http::{Method, StatusCode};
use serde_json::json;

use super::*;

#[tokio::test]
async fn logged_in_enters_with_the_account_name_and_the_rest_with_a_typed_one() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, room(2, 1)).await;
    let ana = user(&db, "Ana").await;
    // Logada, vale o nome da conta, mesmo que venha outro.
    let uri = format!("/veto-rooms/{}/join", links.seats[0]);
    let (status, body) = post(&app, &uri, &ana, json!({ "name": "Outra" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["seat"]["name"], "Ana");
    assert_eq!(body["seats"][0]["name"], "Ana");
    // Sem corpo também.
    let bia = user(&db, "Bia").await;
    let uri = format!("/veto-rooms/{}/join", links.seats[1]);
    let (status, body) = post_empty(&app, &uri, &bia).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["seat"]["name"], "Bia");

    // Sem login, com nome.
    let body = enter(&app, &links.seats[2], "  thallis ").await;
    assert_eq!(body["seat"]["name"], "thallis");
    let invalid = (
        StatusCode::UNPROCESSABLE_ENTITY,
        err("veto_room.name_invalid"),
    );
    assert_eq!(act(&app, &links.seats[3], "join", None).await, invalid);
    for name in ["", "   ", "a\nb", &"x".repeat(33)] {
        assert_eq!(
            act(&app, &links.seats[3], "join", Some(json!({ "name": name }))).await,
            invalid,
            "{name:?}"
        );
    }
    assert_eq!(
        enter(&app, &links.seats[3], &"é".repeat(32)).await["seat"]["name"],
        "é".repeat(32)
    );
}

#[tokio::test]
async fn the_name_is_set_once_and_the_link_is_the_seat() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let links = links_of(&app, room(1, 1)).await;
    enter(&app, &links.seats[0], "king").await;
    // Reabrir o link, ou abri-lo em outro aparelho, é o mesmo lugar.
    let body = enter(&app, &links.seats[0], "outro").await;
    assert_eq!(body["seat"]["name"], "king");
    let ana = user(&db, "Ana").await;
    let (status, body) = call(
        &app,
        Method::GET,
        &format!("/veto-rooms/{}", links.seats[0]),
        Some(&ana),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["seat"]["name"], "king");
    let uri = format!("/veto-rooms/{}/join", links.seats[0]);
    let (_, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(body["seat"]["name"], "king");
    // Os outros veem quem entrou.
    let manage = detail(&app, &links.manage).await;
    assert_eq!(manage["seats"][0]["name"], "king");
    assert!(manage["seats"][1]["name"].is_null());
}

#[tokio::test]
async fn only_a_seat_that_joined_acts() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, room(1, 1)).await;
    let not_player = (StatusCode::FORBIDDEN, err("veto_room.not_player"));
    let choice = json!({ "map": "awoken" });
    for token in [&links.manage, &links.spectator] {
        assert_eq!(
            act(&app, token, "join", Some(json!({ "name": "x" }))).await,
            not_player
        );
        assert_eq!(ready(&app, token).await, not_player);
        assert_eq!(step(&app, token, choice.clone()).await, not_player);
    }
    let not_joined = (StatusCode::CONFLICT, err("veto_room.not_joined"));
    assert_eq!(ready(&app, &links.seats[0]).await, not_joined);
    assert_eq!(
        step(&app, &links.seats[0], choice.clone()).await,
        not_joined
    );
    // Quem não entrou não vê opções, mesmo com a sala andando.
    let (a, b) = (&links.seats[0], &links.seats[1]);
    enter(&app, a, "king").await;
    assert_eq!(
        step(&app, a, choice.clone()).await,
        (StatusCode::CONFLICT, err("veto.not_begun"))
    );
    let not_found = (StatusCode::NOT_FOUND, err("veto_room.not_found"));
    let nobody = "1".repeat(22);
    assert_eq!(read(&app, &nobody).await, not_found);
    assert_eq!(ready(&app, &nobody).await, not_found);
    assert_eq!(
        act(&app, &nobody, "join", Some(json!({ "name": "x" }))).await,
        not_found
    );
    assert_eq!(step(&app, &nobody, choice).await, not_found);
    assert_eq!(detail(&app, b).await["veto"]["options"], json!([]));
}
