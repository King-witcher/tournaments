use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

use crate::common::*;

#[sqlx::test(migrations = "../../migrations")]
async fn tenant_routes_require_login(db: PgPool) {
    let app = app(db);
    for (method, uri) in [
        (Method::POST, "/tenants"),
        (Method::GET, "/tenants"),
        (Method::GET, "/tenants/quake-br"),
        (Method::PATCH, "/tenants/quake-br/me"),
        (Method::GET, "/tenants/quake-br/members"),
        (Method::PUT, "/tenants/quake-br/members/1/role"),
        (Method::POST, "/tenants/quake-br/invites"),
        (Method::GET, "/tenants/quake-br/invites"),
        (Method::DELETE, "/tenants/quake-br/invites/1"),
        (Method::POST, "/invites/abc/accept"),
    ] {
        let (status, body) = call(&app, method.clone(), uri, None, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
        assert_eq!(body, err("auth.unauthenticated"), "{method} {uri}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_makes_the_user_owner(db: PgPool) {
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
    let expected =
        json!({ "slug": "quake-br", "name": "Quake BR", "role": "owner", "nickname": "Ana" });
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

#[sqlx::test(migrations = "../../migrations")]
async fn create_validates_name_and_slug(db: PgPool) {
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

#[sqlx::test(migrations = "../../migrations")]
async fn slug_is_unique(db: PgPool) {
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

#[sqlx::test(migrations = "../../migrations")]
async fn owner_limit_counts_only_owned_tenants(db: PgPool) {
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;

    // Ser player em outro tenant não conta para o limite.
    create(&app, &bia, "da-bia").await;
    let token = invite(&app, &bia, "da-bia").await;
    join(&app, &ana, &token).await;

    for slug in ["um-1", "dois-2", "tres-3"] {
        create(&app, &ana, slug).await;
    }
    let (status, body) = post(
        &app,
        "/tenants",
        &ana,
        json!({ "name": "Quarto", "slug": "quatro-4" }),
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
        4
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn outsiders_get_404(db: PgPool) {
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let eve = user(&db, "Eve").await;
    create(&app, &ana, "quake-br").await;

    for uri in [
        "/tenants/quake-br",
        "/tenants/quake-br/members",
        "/tenants/quake-br/invites",
        "/tenants/nao-existe",
    ] {
        assert_eq!(
            get(&app, uri, &eve).await,
            (StatusCode::NOT_FOUND, err("tenant.not_found")),
            "{uri}"
        );
    }
    let (status, body) = post_empty(&app, "/tenants/quake-br/invites", &eve).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
}
