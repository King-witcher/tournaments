use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

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
    assert_eq!(stored, api::shared::token::hash_token(token).to_vec());

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
        &api::shared::token::hash_token(&expired)[..]
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
