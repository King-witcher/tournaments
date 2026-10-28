use axum::{
    Router,
    http::{Method, StatusCode},
};
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

    let joined = json!({
        "tenant": { "slug": "quake-br", "name": "Tenant quake-br", "role": "player", "nickname": "Bia" },
        "tournament": null
    });
    assert_eq!(join(&app, &bia, token).await, joined);
    // Multiuso, e aceitar de novo não muda nada.
    join(&app, &caio, token).await;
    assert_eq!(join(&app, &bia, token).await, joined);
    // O owner aceitando o próprio link continua owner.
    assert_eq!(join(&app, &ana, token).await["tenant"]["role"], "owner");

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

const COPA: &str = "/tenants/quake-br/tournaments/copa";

/// Gera um link de inscrição no torneio `copa` de `quake-br` e devolve o token.
async fn copa_invite(app: &Router, by: &User) -> String {
    let (status, body) = post_empty(app, &format!("{COPA}/invites"), by).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["token"].as_str().unwrap().to_owned()
}

async fn preview(app: &Router, token: &str) -> (StatusCode, Value) {
    call(app, Method::GET, &format!("/invites/{token}"), None, None).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn tournament_link_joins_the_tenant_and_registers(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let caio = user(&db, "Caio").await;
    let private = with(duel("copa"), json!({ "visibility": "private" }));
    create_tournament(&app, &ana, private).await;

    // Já no rascunho, para mandar antes de publicar.
    let token = copa_invite(&app, &ana).await;
    assert_eq!(
        preview(&app, &token).await,
        (
            StatusCode::OK,
            json!({
                "tenant_slug": "quake-br",
                "tenant_name": "Tenant quake-br",
                "expires_at": null,
                "tournament": {
                    "slug": "copa", "locale": "pt", "name": "Copa de Outubro", "translations": [],
                    "status": "draft", "max_participants": 16, "participants": 0,
                    "starts_at": "2099-01-01T20:00:00Z"
                },
                "viewer": null
            })
        )
    );
    // Sem a inscrição aberta, aceitar não entra nem no tenant.
    let accept = format!("/invites/{token}/accept");
    assert_eq!(
        post_empty(&app, &accept, &caio).await,
        (StatusCode::CONFLICT, err("tournament.registration_closed"))
    );
    assert_eq!(get(&app, "/tenants", &caio).await.1, json!([]));

    publish(&app, &ana, "copa").await;
    let accepted = json!({
        "tenant": { "slug": "quake-br", "name": "Tenant quake-br", "role": "player", "nickname": "Caio" },
        "tournament": "copa"
    });
    assert_eq!(join(&app, &caio, &token).await, accepted);
    assert_eq!(join(&app, &caio, &token).await, accepted);
    let (status, seen) = get(&app, COPA, &caio).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seen["registered"], true);

    // O link fica no torneio, não na lista do tenant, e conta os inscritos por ele.
    let (status, links) = get(&app, &format!("{COPA}/invites"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(links.as_array().unwrap().len(), 1);
    assert_eq!(links[0]["created_by"], "Ana");
    assert_eq!(links[0]["uses"], 1);
    assert!(links[0].get("token").is_none());
    let (_, tenant_invites) = get(&app, "/tenants/quake-br/invites", &ana).await;
    let ids: Vec<_> = tenant_invites
        .as_array()
        .unwrap()
        .iter()
        .map(|i| &i["id"])
        .collect();
    assert!(!ids.contains(&&links[0]["id"]), "{tenant_invites}");

    // Revoga pela rota de sempre.
    let revoke = format!("/tenants/quake-br/invites/{}", links[0]["id"]);
    assert_eq!(delete(&app, &revoke, &ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(
        preview(&app, &token).await,
        (StatusCode::GONE, err("invite.revoked"))
    );
    assert_eq!(
        get(&app, &format!("{COPA}/invites"), &ana).await.1,
        json!([])
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn tournament_link_needs_a_free_slot(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = user(&db, "Caio").await;
    create_tournament(
        &app,
        &ana,
        with(duel("copa"), json!({ "max_participants": 2 })),
    )
    .await;
    publish(&app, &ana, "copa").await;
    let token = copa_invite(&app, &ana).await;

    // Quem já é membro só se inscreve; o owner continua owner.
    assert_eq!(join(&app, &bia, &token).await["tournament"], "copa");
    assert_eq!(join(&app, &ana, &token).await["tenant"]["role"], "owner");
    assert_eq!(
        post_empty(&app, &format!("/invites/{token}/accept"), &caio).await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "tournament.full", "params": { "max": 2 } })
        )
    );
    assert_eq!(get(&app, "/tenants", &caio).await.1, json!([]));
    assert_eq!(
        preview(&app, &token).await.1["tournament"]["participants"],
        2
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn tournament_links_only_before_the_start(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let links = format!("{COPA}/invites");
    let forbidden = (StatusCode::FORBIDDEN, err("tenant.forbidden"));
    assert_eq!(post_empty(&app, &links, &bia).await, forbidden);
    assert_eq!(get(&app, &links, &bia).await, forbidden);
    assert_eq!(
        post_empty(&app, "/tenants/quake-br/tournaments/outra/invites", &ana).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );

    publish(&app, &ana, "copa").await;
    let token = copa_invite(&app, &ana).await;
    set_status(&db, "copa", "in_progress").await;
    let closed = (StatusCode::CONFLICT, err("tournament.registration_closed"));
    assert_eq!(post_empty(&app, &links, &ana).await, closed);
    let caio = user(&db, "Caio").await;
    assert_eq!(
        post_empty(&app, &format!("/invites/{token}/accept"), &caio).await,
        closed
    );

    // Excluir o rascunho apaga os links dele.
    create_tournament(&app, &ana, duel("rascunho")).await;
    let draft = "/tenants/quake-br/tournaments/rascunho";
    let (_, created) = post_empty(&app, &format!("{draft}/invites"), &ana).await;
    assert_eq!(delete(&app, draft, &ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(
        preview(&app, created["token"].as_str().unwrap()).await,
        (StatusCode::NOT_FOUND, err("invite.not_found"))
    );
}

/// `viewer` da prévia como `who` a vê.
async fn viewer(app: &Router, token: &str, who: &User) -> Value {
    let (status, body) = get(app, &format!("/invites/{token}"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["viewer"].clone()
}

#[sqlx::test(migrations = "../../migrations")]
async fn preview_knows_who_opens_it(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = user(&db, "Caio").await;
    let token = invite(&app, &ana, "quake-br").await;

    // Sem login, só o convite; logado, se é membro.
    assert_eq!(preview(&app, &token).await.1["tenant_slug"], "quake-br");
    assert_eq!(
        viewer(&app, &token, &caio).await,
        json!({ "membership": "none", "nickname": null, "registered": false })
    );
    assert_eq!(
        viewer(&app, &token, &bia).await,
        json!({ "membership": "member", "nickname": "Bia", "registered": false })
    );
    // Quem saiu volta com o apelido de antes.
    let (status, _) = patch(
        &app,
        "/tenants/quake-br/me",
        &bia,
        json!({ "nickname": "Bia QC" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = delete(&app, "/tenants/quake-br/me", &bia).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    assert_eq!(
        viewer(&app, &token, &bia).await,
        json!({ "membership": "former", "nickname": "Bia QC", "registered": false })
    );

    // No link de inscrição, também se já está inscrito.
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    let link = copa_invite(&app, &ana).await;
    assert_eq!(viewer(&app, &link, &ana).await["registered"], false);
    register(&app, "copa", &[&ana]).await;
    assert_eq!(
        viewer(&app, &link, &ana).await,
        json!({ "membership": "member", "nickname": "Ana", "registered": true })
    );
}
