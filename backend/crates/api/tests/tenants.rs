mod common;

use axum::{
    Router,
    http::{Method, StatusCode},
};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

async fn create(app: &Router, owner: &User, slug: &str) -> Value {
    let (status, body) = post(
        app,
        "/tenants",
        owner,
        json!({ "name": format!("Tenant {slug}"), "slug": slug }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body
}

/// Gera um convite em `slug` e devolve o token.
async fn invite(app: &Router, by: &User, slug: &str) -> String {
    let (status, body) = post_empty(app, &format!("/tenants/{slug}/invites"), by).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["token"].as_str().unwrap().to_owned()
}

async fn join(app: &Router, who: &User, token: &str) -> Value {
    let (status, body) = post_empty(app, &format!("/invites/{token}/accept"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

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

#[sqlx::test(migrations = "../../migrations")]
async fn invite_link_adds_a_player(db: PgPool) {
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    let caio = user(&db, "Caio").await;
    create(&app, &ana, "quake-br").await;

    let (status, created) = post_empty(&app, "/tenants/quake-br/invites", &ana).await;
    assert_eq!(status, StatusCode::CREATED);
    let token = created["token"].as_str().unwrap();
    assert_eq!(token.len(), 22);
    assert!(token.chars().all(|c| c.is_ascii_alphanumeric()), "{token}");

    // O banco guarda só o hash do token.
    let stored = sqlx::query_scalar!("select token_hash from tenant_invites")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(stored, api::auth::hash_token(token).to_vec());

    // Ver o convite não exige login.
    let (status, preview) = call(&app, Method::GET, &format!("/invites/{token}"), None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(preview["tenant_name"], "Tenant quake-br");
    assert_eq!(preview["expires_at"], created["expires_at"]);

    let joined = json!({ "slug": "quake-br", "name": "Tenant quake-br", "role": "player", "nickname": "Bia" });
    assert_eq!(join(&app, &bia, token).await, joined);
    // Multiuso, e aceitar de novo não muda nada.
    join(&app, &caio, token).await;
    assert_eq!(join(&app, &bia, token).await, joined);
    // O owner aceitando o próprio link continua owner.
    assert_eq!(join(&app, &ana, token).await["role"], "owner");

    let (_, members) = get(&app, "/tenants/quake-br/members", &bia).await;
    let names: Vec<_> = members
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["nickname"].as_str().unwrap(), m["role"].as_str().unwrap()))
        .collect();
    assert_eq!(
        names,
        [("Ana", "owner"), ("Bia", "player"), ("Caio", "player")]
    );

    let (status, invites) = get(&app, "/tenants/quake-br/invites", &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(invites.as_array().unwrap().len(), 1);
    assert_eq!(invites[0]["id"], created["id"]);
    assert_eq!(invites[0]["created_by"], "Ana");
    assert_eq!(invites[0]["uses"], 2);
    assert!(invites[0].get("token").is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_organizers_and_owner_manage_invites(db: PgPool) {
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    let token = invite(&app, &ana, "quake-br").await;
    join(&app, &bia, &token).await;
    let invite_id = get(&app, "/tenants/quake-br/invites", &ana).await.1[0]["id"].clone();

    let forbidden = (StatusCode::FORBIDDEN, err("tenant.forbidden"));
    assert_eq!(
        post_empty(&app, "/tenants/quake-br/invites", &bia).await,
        forbidden
    );
    assert_eq!(
        get(&app, "/tenants/quake-br/invites", &bia).await,
        forbidden
    );
    assert_eq!(
        delete(
            &app,
            &format!("/tenants/quake-br/invites/{invite_id}"),
            &bia
        )
        .await,
        forbidden
    );

    let (status, _) = put(
        &app,
        &format!("/tenants/quake-br/members/{}/role", bia.id),
        &ana,
        json!({ "role": "organizer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let new_token = invite(&app, &bia, "quake-br").await;
    assert_ne!(new_token, token);

    let (_, invites) = get(&app, "/tenants/quake-br/invites", &bia).await;
    assert_eq!(invites[0]["created_by"], "Bia");
    assert_eq!(
        delete(
            &app,
            &format!("/tenants/quake-br/invites/{invite_id}"),
            &bia
        )
        .await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn expired_revoked_and_unknown_invites(db: PgPool) {
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    let expired = invite(&app, &ana, "quake-br").await;
    let revoked = invite(&app, &ana, "quake-br").await;

    sqlx::query!(
        "update tenant_invites set expires_at = now() - interval '1 second' where token_hash = $1",
        &api::auth::hash_token(&expired)[..]
    )
    .execute(&db)
    .await
    .unwrap();
    let (_, invites) = get(&app, "/tenants/quake-br/invites", &ana).await;
    assert_eq!(invites.as_array().unwrap().len(), 1);
    let revoke_uri = format!("/tenants/quake-br/invites/{}", invites[0]["id"]);
    for _ in 0..2 {
        // Revogar de novo também responde 204.
        assert_eq!(
            delete(&app, &revoke_uri, &ana).await.0,
            StatusCode::NO_CONTENT
        );
    }
    assert_eq!(
        get(&app, "/tenants/quake-br/invites", &ana).await.1,
        json!([])
    );

    for (token, status, code) in [
        (expired.as_str(), StatusCode::GONE, "invite.expired"),
        (revoked.as_str(), StatusCode::GONE, "invite.revoked"),
        ("naoexiste", StatusCode::NOT_FOUND, "invite.not_found"),
    ] {
        let preview = call(&app, Method::GET, &format!("/invites/{token}"), None, None).await;
        assert_eq!(preview, (status, err(code)), "{token}");
        let accept = post_empty(&app, &format!("/invites/{token}/accept"), &bia).await;
        assert_eq!(accept, (status, err(code)), "{token}");
    }
    assert_eq!(get(&app, "/tenants", &bia).await.1, json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn invite_ids_do_not_cross_tenants(db: PgPool) {
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let eve = user(&db, "Eve").await;
    create(&app, &ana, "da-ana").await;
    create(&app, &eve, "da-eve").await;
    let token = invite(&app, &ana, "da-ana").await;
    let ana_invite = get(&app, "/tenants/da-ana/invites", &ana).await.1[0]["id"].clone();

    // Eve é owner do tenant dela, mas o convite é do tenant da Ana.
    assert_eq!(
        delete(&app, &format!("/tenants/da-eve/invites/{ana_invite}"), &eve).await,
        (StatusCode::NOT_FOUND, err("invite.not_found"))
    );
    assert_eq!(
        delete(&app, "/tenants/da-eve/invites/abc", &eve).await,
        (StatusCode::BAD_REQUEST, err("request.invalid_path"))
    );
    let (status, _) = call(&app, Method::GET, &format!("/invites/{token}"), None, None).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn owner_promotes_and_demotes(db: PgPool) {
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

#[sqlx::test(migrations = "../../migrations")]
async fn member_edits_own_nickname(db: PgPool) {
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
    assert_eq!(join(&app, &ana, &token).await["nickname"], "Ana");

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
