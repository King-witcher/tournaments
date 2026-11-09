//! Testes de `tenants`: criar, listar, ver e editar aqui; trocar o endereço em
//! `address`; transferir a posse em `transfer`; excluir em `deletion`.

mod address;
mod deletion;
mod transfer;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::common::*;

#[tokio::test]
async fn tenant_routes_require_login() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    for (method, uri) in [
        (Method::POST, "/tenants"),
        (Method::GET, "/tenants"),
        (Method::GET, "/tenants/quake-br"),
        (Method::PATCH, "/tenants/quake-br"),
        (Method::PATCH, "/tenants/quake-br/me"),
        (Method::GET, "/tenants/quake-br/members"),
        (Method::PUT, "/tenants/quake-br/members/1/role"),
        (Method::GET, "/tenants/quake-br/invite"),
        (Method::POST, "/tenants/quake-br/invite/rotate"),
        (Method::POST, "/invites/abc/accept"),
    ] {
        let (status, body) = call(&app, method.clone(), uri, None, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
        assert_eq!(body, err("auth.unauthenticated"), "{method} {uri}");
    }
}

#[tokio::test]
async fn create_makes_the_user_owner() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;

    let (status, body) = post(
        &app,
        "/tenants",
        &ana,
        json!({ "name": "  Quake BR  ", "slug": "quake-br" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let expected = json!({
        "slug": "quake-br",
        "name": "Quake BR",
        "description": null,
        "role": "owner",
        "nickname": "Ana",
        "deletion_scheduled_for": null,
        "logo_url": null,
        "cover_url": null
    });
    assert_eq!(body, expected);

    assert_eq!(get(&app, "/tenants", &ana).await.1, json!([expected]));
    assert_eq!(
        get(&app, "/tenants/quake-br", &ana).await,
        (StatusCode::OK, expected)
    );

    let (status, members) = get(&app, "/tenants/quake-br/members", &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(members.as_array().unwrap().len(), 1);
    assert_eq!(members[0]["user_id"], ana.id);
    assert_eq!(members[0]["role"], "owner");
    assert_eq!(members[0]["nickname"], "Ana");
    assert_eq!(members[0]["display_name"], "Ana");
}

#[tokio::test]
async fn create_validates_name_and_slug() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;

    let cases = [
        (json!({ "name": "X", "slug": "ab" }), "tenant.slug_invalid"),
        (
            json!({ "name": "X", "slug": "Quake" }),
            "tenant.slug_invalid",
        ),
        (
            json!({ "name": "X", "slug": "quake--br" }),
            "tenant.slug_invalid",
        ),
        (
            json!({ "name": "X", "slug": "a".repeat(33) }),
            "tenant.slug_invalid",
        ),
        (
            json!({ "name": "X", "slug": "new" }),
            "tenant.slug_reserved",
        ),
        (
            json!({ "name": "X", "slug": "invite" }),
            "tenant.slug_reserved",
        ),
        (
            json!({ "name": "   ", "slug": "quake-br" }),
            "tenant.name_invalid",
        ),
        (
            json!({ "name": "a".repeat(65), "slug": "quake-br" }),
            "tenant.name_invalid",
        ),
        (json!({ "name": "Quake BR" }), "request.invalid_body"),
        (
            json!({ "name": 1, "slug": "quake-br" }),
            "request.invalid_body",
        ),
    ];
    for (body, code) in cases {
        let (status, res) = post(&app, "/tenants", &ana, body.clone()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(res, err(code), "{body}");
    }
    assert_eq!(get(&app, "/tenants", &ana).await.1, json!([]));
}

#[tokio::test]
async fn slug_is_unique() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    create(&app, &ana, "quake-br").await;

    let (status, body) = post(
        &app,
        "/tenants",
        &bia,
        json!({ "name": "Outro", "slug": "quake-br" }),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("tenant.slug_taken"))
    );
    assert_eq!(get(&app, "/tenants", &bia).await.1, json!([]));
}

#[tokio::test]
async fn owner_limit_counts_only_owned_tenants() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;

    // Ser player em outro tenant não conta para o limite.
    create(&app, &bia, "da-bia").await;
    let token = invite(&app, &bia, "da-bia").await;
    join(&app, &ana, &token).await;

    for slug in ["um-1", "dois-2", "tres-3", "quatro-4", "cinco-5"] {
        create(&app, &ana, slug).await;
    }
    let (status, body) = post(
        &app,
        "/tenants",
        &ana,
        json!({ "name": "Sexto", "slug": "seis-6" }),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("tenant.limit_reached"))
    );
    assert_eq!(
        get(&app, "/tenants", &ana)
            .await
            .1
            .as_array()
            .unwrap()
            .len(),
        6
    );
}

#[tokio::test]
async fn outsiders_get_404() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let eve = user(&db, "Eve").await;
    create(&app, &ana, "quake-br").await;

    for uri in [
        "/tenants/quake-br",
        "/tenants/quake-br/members",
        "/tenants/quake-br/invite",
        "/tenants/nao-existe",
    ] {
        assert_eq!(
            get(&app, uri, &eve).await,
            (StatusCode::NOT_FOUND, err("tenant.not_found")),
            "{uri}"
        );
    }
    let (status, body) = post_empty(&app, "/tenants/quake-br/invite/rotate", &eve).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
}

#[tokio::test]
async fn the_owner_edits_name_and_description() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;

    let edit = json!({ "name": "  Quake Brasil ", "description": "\r\n Duelos às quintas.\r\n\r\nVenha. \n" });
    let (status, body) = patch(&app, "/tenants/quake-br", &ana, edit).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["slug"], "quake-br");
    assert_eq!(body["name"], "Quake Brasil");
    assert_eq!(body["description"], "Duelos às quintas.\n\nVenha.");
    assert_eq!(body["role"], "owner");
    // Todos os membros veem, na comunidade e na lista.
    let (_, seen) = get(&app, "/tenants/quake-br", &bia).await;
    assert_eq!(seen["name"], "Quake Brasil");
    assert_eq!(seen["description"], "Duelos às quintas.\n\nVenha.");
    let (_, mine) = get(&app, "/tenants", &bia).await;
    assert_eq!(mine[0]["name"], "Quake Brasil");

    // O campo que não vem fica como está.
    let (_, body) = patch(&app, "/tenants/quake-br", &ana, json!({ "name": "QB" })).await;
    assert_eq!(
        (&body["name"], &body["description"]),
        (&json!("QB"), &json!("Duelos às quintas.\n\nVenha."))
    );
    let only_description = json!({ "description": "Outra." });
    let (_, body) = patch(&app, "/tenants/quake-br", &ana, only_description).await;
    assert_eq!(
        (&body["name"], &body["description"]),
        (&json!("QB"), &json!("Outra."))
    );

    // Vazia: fica sem descrição, no banco também (que recusa string vazia).
    let (status, body) = patch(
        &app,
        "/tenants/quake-br",
        &ana,
        json!({ "description": " \n " }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["description"], Value::Null);
    let (_, seen) = get(&app, "/tenants/quake-br", &bia).await;
    assert_eq!(seen["description"], Value::Null);
}

#[tokio::test]
async fn only_the_owner_edits_the_tenant() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let zeca = user(&db, "Zeca").await;

    let edit = json!({ "name": "Tomada" });
    for who in [&olga, &bia] {
        assert_eq!(
            patch(&app, "/tenants/quake-br", who, edit.clone()).await,
            (StatusCode::FORBIDDEN, err("tenant.forbidden"))
        );
    }
    assert_eq!(
        patch(&app, "/tenants/quake-br", &zeca, edit).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    assert_eq!(
        get(&app, "/tenants/quake-br", &ana).await.1["name"],
        "Tenant quake-br"
    );
}

#[tokio::test]
async fn edit_validates_name_and_description() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;

    let too_long = json!({ "error": "tenant.description_invalid", "params": { "max": 2000 } });
    let cases = [
        (json!({ "name": "   " }), err("tenant.name_invalid")),
        (json!({ "name": "a\nb" }), err("tenant.name_invalid")),
        (
            json!({ "name": "é".repeat(65) }),
            err("tenant.name_invalid"),
        ),
        (json!({ "description": "é".repeat(2001) }), too_long.clone()),
        (json!({ "description": "a\u{7}b" }), too_long),
        (json!({ "name": 1 }), err("request.invalid_body")),
    ];
    for (body, expected) in cases {
        let (status, res) = patch(&app, "/tenants/quake-br", &ana, body.clone()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(res, expected, "{body}");
    }

    // Os limites, com caracteres de mais de um byte: 64 e 2000 ainda passam.
    let edit = json!({ "name": "é".repeat(64), "description": "é".repeat(2000) });
    let (status, body) = patch(&app, "/tenants/quake-br", &ana, edit).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
