use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::*;

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
