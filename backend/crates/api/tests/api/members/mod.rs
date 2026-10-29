//! Testes de `members`: papéis e apelido aqui, sair e remover em
//! `departure`, o perfil com os torneios do membro em `profile`.

mod departure;
mod profile;

use axum::http::StatusCode;
use serde_json::json;

use crate::common::*;

#[tokio::test]
async fn owner_promotes_and_demotes() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    let caio = user(&db, "Caio").await;
    let eve = user(&db, "Eve").await;
    create(&app, &ana, "quake-br").await;
    create(&app, &eve, "da-eve").await;
    let token = invite(&app, &ana, "quake-br").await;
    join(&app, &bia, &token).await;
    join(&app, &caio, &token).await;
    let role_uri = |u: &User| format!("/tenants/quake-br/members/{}/role", u.id);

    let (status, member) = put(&app, &role_uri(&bia), &ana, json!({ "role": "organizer" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(member["user_id"], bia.id);
    assert_eq!(member["role"], "organizer");
    assert_eq!(
        get(&app, "/tenants/quake-br", &bia).await.1["role"],
        "organizer"
    );

    // Organizer não troca papéis; só o owner.
    assert_eq!(
        put(&app, &role_uri(&caio), &bia, json!({ "role": "organizer" })).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    // O papel do owner não muda por aqui, nem por ele mesmo.
    assert_eq!(
        put(&app, &role_uri(&ana), &ana, json!({ "role": "player" })).await,
        (StatusCode::CONFLICT, err("member.is_owner"))
    );
    // Ninguém vira owner por aqui.
    assert_eq!(
        put(&app, &role_uri(&caio), &ana, json!({ "role": "owner" })).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("request.invalid_body")
        )
    );
    // Eve existe, mas não é membro deste tenant.
    assert_eq!(
        put(&app, &role_uri(&eve), &ana, json!({ "role": "player" })).await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );

    let (status, member) = put(&app, &role_uri(&bia), &ana, json!({ "role": "player" })).await;
    assert_eq!(
        (status, member["role"].clone()),
        (StatusCode::OK, json!("player"))
    );
    assert_eq!(
        post_empty(&app, "/tenants/quake-br/invites", &bia).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
}

#[tokio::test]
async fn member_edits_own_nickname() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    create(&app, &bia, "da-bia").await;

    let (status, body) = patch(
        &app,
        "/tenants/quake-br/me",
        &ana,
        json!({ "nickname": "  rapha " }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["nickname"], "rapha");
    assert_eq!(body["role"], "owner");
    assert_eq!(
        get(&app, "/tenants/quake-br/members", &ana).await.1[0]["nickname"],
        "rapha"
    );
    // O apelido é por tenant.
    let token = invite(&app, &bia, "da-bia").await;
    assert_eq!(join(&app, &ana, &token).await["tenant"]["nickname"], "Ana");

    for bad in [
        json!(""),
        json!("   "),
        json!("a".repeat(33)),
        json!("a\tb"),
    ] {
        assert_eq!(
            patch(
                &app,
                "/tenants/quake-br/me",
                &ana,
                json!({ "nickname": bad })
            )
            .await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("member.nickname_invalid")
            ),
            "{bad}"
        );
    }
}

#[tokio::test]
async fn organizers_see_which_link_each_member_came_by() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let caio = user(&db, "Caio").await;
    let (status, created) =
        post_empty(&app, "/tenants/quake-br/tournaments/copa/invites", &ana).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let link = created["token"].as_str().unwrap();
    publish(&app, &ana, "copa").await;
    join(&app, &caio, link).await;

    let (status, list) = get(&app, "/tenants/quake-br/members", &ana).await;
    assert_eq!(status, StatusCode::OK);
    let via = |name: &str| {
        list.as_array()
            .unwrap()
            .iter()
            .find(|m| m["nickname"] == name)
            .unwrap()["joined_via"]
            .clone()
    };
    // O owner não entrou por convite; Bia, pelo da comunidade; Caio, pelo link do torneio.
    assert_eq!(via("Ana"), serde_json::Value::Null);
    let bia_via = via("Bia");
    assert_eq!(bia_via["tournament"], serde_json::Value::Null);
    assert_eq!(bia_via["hint"].as_str().unwrap().len(), 4);
    assert_eq!(
        via("Caio"),
        json!({ "hint": &link[..4], "tournament": "Copa de Outubro" })
    );

    // Player não vê de onde cada um veio.
    let (_, list) = get(&app, "/tenants/quake-br/members", &bia).await;
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .all(|m| m["joined_via"].is_null())
    );
}
