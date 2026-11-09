use api::scanners::SECRET_HEADER;
use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::common::*;

const ROTATE: &str = "/tenants/quake-br/invite/rotate";

/// O código de convite novo: 8 caracteres, minúsculas e dígitos sem ambíguos.
fn assert_short(token: &str) {
    assert_eq!(token.len(), 8, "{token}");
    assert!(
        token
            .bytes()
            .all(|b| b"23456789abcdefghjkmnpqrstuvwxyz".contains(&b)),
        "{token}"
    );
}

fn not_found() -> (StatusCode, Value) {
    (StatusCode::NOT_FOUND, err("invite.not_found"))
}

async fn preview(app: &Router, token: &str) -> (StatusCode, Value) {
    call(app, Method::GET, &format!("/invites/{token}"), None, None).await
}

/// A prévia e o aceite de `token` respondem que o link não existe.
async fn assert_dead(app: &Router, token: &str, who: &User) {
    assert_eq!(preview(app, token).await, not_found(), "{token}");
    let accept = post_empty(app, &format!("/invites/{token}/accept"), who).await;
    assert_eq!(accept, not_found(), "{token}");
}

/// Troca o link em `uri` e devolve o token novo.
async fn rotate(app: &Router, uri: &str, by: &User) -> String {
    let (status, body) = post_empty(app, uri, by).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["token"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn invite_link_adds_a_player() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    let caio = user(&db, "Caio").await;
    create(&app, &ana, "quake-br").await;

    // A comunidade já nasce com o link, e ele não muda a cada leitura.
    let token = invite(&app, &ana, "quake-br").await;
    assert_short(&token);
    assert_eq!(invite(&app, &ana, "quake-br").await, token);
    let token = token.as_str();

    // Ver o convite não exige login.
    let (status, seen) = preview(&app, token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seen["tenant_name"], "Tenant quake-br");
    // Quem vai entrar vê quem é o dono e quantos já estão lá.
    assert_eq!(seen["owner_name"], "Ana");
    assert_eq!(seen["member_count"], 1);
    assert_eq!(seen["tenant_description"], Value::Null);
    // E o que a comunidade diz de si.
    let edit = json!({ "description": "Duelos às quintas." });
    assert_eq!(
        patch(&app, "/tenants/quake-br", &ana, edit).await.0,
        StatusCode::OK
    );
    let (_, seen) = preview(&app, token).await;
    assert_eq!(seen["tenant_description"], "Duelos às quintas.");

    let joined = json!({
        "tenant": { "slug": "quake-br", "name": "Tenant quake-br", "description": "Duelos às quintas.", "role": "player", "nickname": "Bia", "deletion_scheduled_for": null, "logo_url": null, "cover_url": null },
        "tournament": null
    });
    assert_eq!(join(&app, &bia, token).await, joined);
    // Multiuso, e aceitar de novo não muda nada.
    join(&app, &caio, token).await;
    assert_eq!(join(&app, &bia, token).await, joined);
    // O owner aceitando o próprio link continua owner.
    assert_eq!(join(&app, &ana, token).await["tenant"]["role"], "owner");
    let (_, after) = preview(&app, token).await;
    assert_eq!(after["member_count"], 3);

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
}

#[tokio::test]
async fn only_organizers_and_owner_see_and_rotate_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let token = invite(&app, &ana, "quake-br").await;

    let forbidden = (StatusCode::FORBIDDEN, err("tenant.forbidden"));
    assert_eq!(get(&app, "/tenants/quake-br/invite", &bia).await, forbidden);
    assert_eq!(post_empty(&app, ROTATE, &bia).await, forbidden);
    assert_eq!(invite(&app, &ana, "quake-br").await, token);

    let (status, _) = put(
        &app,
        &format!("/tenants/quake-br/members/{}/role", bia.id),
        &ana,
        json!({ "role": "organizer" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(invite(&app, &bia, "quake-br").await, token);
    let rotated = rotate(&app, ROTATE, &bia).await;
    assert_ne!(rotated, token);
    assert_eq!(invite(&app, &ana, "quake-br").await, rotated);

    // Nenhuma outra resposta da comunidade traz o token.
    for uri in ["/tenants/quake-br", "/tenants", "/tenants/quake-br/members"] {
        let (status, body) = get(&app, uri, &ana).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert!(!body.to_string().contains(&rotated), "{uri}: {body}");
    }
}

#[tokio::test]
async fn rotating_kills_the_old_link_and_keeps_who_joined() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = user(&db, "Caio").await;
    let old = invite(&app, &ana, "quake-br").await;

    let new = rotate(&app, ROTATE, &ana).await;
    assert_short(&new);
    assert_ne!(new, old);

    assert_dead(&app, &old, &caio).await;
    assert_eq!(get(&app, "/tenants", &caio).await.1, json!([]));
    assert_eq!(
        get(&app, "/tenants/quake-br", &bia).await.0,
        StatusCode::OK,
        "quem entrou pelo link antigo continua"
    );
    assert_eq!(join(&app, &caio, &new).await["tenant"]["role"], "player");

    assert_dead(&app, "naoexiste", &caio).await;
}

#[tokio::test]
async fn each_community_has_its_own_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let eve = user(&db, "Eve").await;
    create(&app, &ana, "da-ana").await;
    create(&app, &eve, "da-eve").await;
    let token = invite(&app, &ana, "da-ana").await;
    assert_ne!(invite(&app, &eve, "da-eve").await, token);

    // Eve é owner da comunidade dela, e trocar o link dela não mexe no da Ana.
    rotate(&app, "/tenants/da-eve/invite/rotate", &eve).await;
    assert_eq!(invite(&app, &ana, "da-ana").await, token);
    let (status, _) = preview(&app, &token).await;
    assert_eq!(status, StatusCode::OK);
}

const COPA: &str = "/tenants/quake-br/tournaments/copa";

/// O link de inscrição do torneio `copa` de `quake-br`: o token.
async fn copa_invite(app: &Router, by: &User) -> String {
    let (status, body) = get(app, &format!("{COPA}/invite"), by).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["token"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn tournament_link_joins_the_tenant_and_registers() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let caio = user(&db, "Caio").await;
    let dani = user(&db, "Dani").await;
    let invite_only = with(duel("copa"), json!({ "admission": "invite_only" }));
    create_tournament(&app, &ana, invite_only).await;

    // O torneio nasce com o link, já no rascunho, para mandar antes de publicar.
    let token = copa_invite(&app, &ana).await;
    assert_ne!(token, invite(&app, &ana, "quake-br").await);
    assert_eq!(
        preview(&app, &token).await,
        (
            StatusCode::OK,
            json!({
                "tenant_slug": "quake-br",
                "tenant_name": "Tenant quake-br",
                "tenant_description": null,
                "tenant_logo_url": null,
                "tenant_cover_url": null,
                "owner_name": "Ana",
                "member_count": 2,
                "tournament": {
                    "slug": "copa", "locale": "pt", "name": "Copa de Outubro",
                    "description": "Duelo **aberto**.", "rules": "- md3\n- md5 na grande final",
                    "mode": "duel", "team_size": 1, "translations": [],
                    "status": "draft", "max_participants": 16, "participants": 0,
                    "starts_at": "2099-01-01T20:00:00Z", "cover_url": null
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
        "tenant": { "slug": "quake-br", "name": "Tenant quake-br", "description": null, "role": "player", "nickname": "Caio", "deletion_scheduled_for": null, "logo_url": null, "cover_url": null },
        "tournament": "copa"
    });
    assert_eq!(join(&app, &caio, &token).await, accepted);
    assert_eq!(join(&app, &caio, &token).await, accepted);
    let (status, seen) = get(&app, COPA, &caio).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(seen["registered"], true);
    // A resposta do torneio não traz o token, nem para quem organiza.
    let (_, seen) = get(&app, COPA, &ana).await;
    assert!(!seen.to_string().contains(&token), "{seen}");

    // Trocar o link: o antigo morre, e quem se inscreveu por ele continua.
    let rotated = rotate(&app, &format!("{COPA}/invite/rotate"), &ana).await;
    assert_short(&rotated);
    assert_ne!(rotated, token);
    assert_eq!(copa_invite(&app, &ana).await, rotated);
    assert_dead(&app, &token, &dani).await;
    let (_, seen) = get(&app, COPA, &caio).await;
    assert_eq!(seen["registered"], true);
    assert_eq!(join(&app, &dani, &rotated).await["tournament"], "copa");
}

#[tokio::test]
async fn tournament_link_needs_a_free_slot() {
    let (db, _lease) = crate::db::test_db().await;
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

#[tokio::test]
async fn tournament_links_only_before_the_start() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let link = format!("{COPA}/invite");
    let rotation = format!("{COPA}/invite/rotate");
    let forbidden = (StatusCode::FORBIDDEN, err("tenant.forbidden"));
    assert_eq!(get(&app, &link, &bia).await, forbidden);
    assert_eq!(post_empty(&app, &rotation, &bia).await, forbidden);
    assert_eq!(
        get(&app, "/tenants/quake-br/tournaments/outra/invite", &ana).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );

    publish(&app, &ana, "copa").await;
    let token = copa_invite(&app, &ana).await;
    // Durante o check-in o link ainda vale, e quem entra já faz o check-in.
    set_status(&db, "copa", "check_in").await;
    let dani = user(&db, "Dani").await;
    assert_eq!(join(&app, &dani, &token).await["tournament"], "copa");
    let (_, seen) = get(&app, COPA, &dani).await;
    assert_eq!(
        (&seen["registered"], &seen["checked_in"]),
        (&json!(true), &json!(true))
    );
    let token = rotate(&app, &rotation, &ana).await;

    set_status(&db, "copa", "in_progress").await;
    let closed = (StatusCode::CONFLICT, err("tournament.registration_closed"));
    assert_eq!(post_empty(&app, &rotation, &ana).await, closed);
    let caio = user(&db, "Caio").await;
    assert_eq!(
        post_empty(&app, &format!("/invites/{token}/accept"), &caio).await,
        closed
    );

    // Excluir o rascunho apaga o link dele.
    create_tournament(&app, &ana, duel("rascunho")).await;
    let draft = "/tenants/quake-br/tournaments/rascunho";
    let (_, created) = get(&app, &format!("{draft}/invite"), &ana).await;
    assert_eq!(delete(&app, draft, &ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(
        preview(&app, created["token"].as_str().unwrap()).await,
        not_found()
    );
}

/// `viewer` da prévia como `who` a vê.
async fn viewer(app: &Router, token: &str, who: &User) -> Value {
    let (status, body) = get(app, &format!("/invites/{token}"), who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["viewer"].clone()
}

#[tokio::test]
async fn preview_knows_who_opens_it() {
    let (db, _lease) = crate::db::test_db().await;
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

#[tokio::test]
async fn tournaments_are_born_with_a_short_code() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    assert_short(&copa_invite(&app, &ana).await);
}

#[tokio::test]
async fn old_long_tokens_keep_working() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    let old = "7Fq2xKpR9mWbTnVc3hJdYe";
    sqlx::query("update tenants set invite_token = $1")
        .bind(old)
        .execute(&db)
        .await
        .unwrap();

    assert_eq!(preview(&app, old).await.0, StatusCode::OK);
    assert_eq!(
        join(&app, &bia, old).await["tenant"]["slug"],
        json!("quake-br")
    );
    assert_short(&rotate(&app, ROTATE, &ana).await);
    assert_dead(&app, old, &bia).await;
}

/// Prévia de um código que não existe, com os cabeçalhos dados.
async fn preview_with(app: &Router, headers: &[(&str, &str)]) -> StatusCode {
    let mut req = Request::get("/invites/abcd2345");
    for (name, value) in headers {
        req = req.header(*name, *value);
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    res.status()
}

const CF_IP: &str = "cf-connecting-ip";
const VIEWER_IP: &str = "x-viewer-ip";
/// O teto da janela (`invites/limit.rs`).
const MAX_PER_IP: usize = 120;

#[tokio::test]
async fn preview_and_accept_are_limited_per_ip() {
    let (db, _lease) = crate::db::test_db().await;
    let bia = user(&db, "Bia").await;
    let app = app_from(state(db));
    let from = [(CF_IP, "203.0.113.7")];
    for _ in 0..MAX_PER_IP {
        assert_eq!(preview_with(&app, &from).await, StatusCode::NOT_FOUND);
    }
    assert_eq!(
        preview_with(&app, &from).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    // O aceite conta no mesmo balde.
    let accept = Request::post("/invites/abcd2345/accept")
        .header(CF_IP, "203.0.113.7")
        .header(header::COOKIE, &bia.cookie)
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(accept).await.unwrap();
    assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["error"], "invite.rate_limited");
    assert_eq!(body["params"]["minutes"], 10);

    // Outro IP não sente, e o cabeçalho do Next não vale sem o segredo.
    assert_eq!(
        preview_with(&app, &[(CF_IP, "198.51.100.9")]).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        preview_with(&app, &[(CF_IP, "203.0.113.7"), (VIEWER_IP, "198.51.100.1")]).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn the_next_server_counts_each_viewer_on_their_own_ip() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(state(db));
    // O IP de saída da Vercel, o mesmo para todos.
    let next = |viewer: &'static str| {
        [
            (CF_IP, "76.76.21.21"),
            (SECRET_HEADER, INTERNAL_SECRET),
            (VIEWER_IP, viewer),
        ]
    };
    for _ in 0..MAX_PER_IP {
        let answer = preview_with(&app, &next("203.0.113.7")).await;
        assert_eq!(answer, StatusCode::NOT_FOUND);
    }
    assert_eq!(
        preview_with(&app, &next("203.0.113.7")).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        preview_with(&app, &next("198.51.100.9")).await,
        StatusCode::NOT_FOUND
    );
    // Sem dizer de quem é o pedido, não soma todos no IP da Vercel.
    let unnamed = [(CF_IP, "76.76.21.21"), (SECRET_HEADER, INTERNAL_SECRET)];
    assert_eq!(preview_with(&app, &unnamed).await, StatusCode::NOT_FOUND);
    // O balde de quem usa a página é o mesmo de quem chama a API direto.
    assert_eq!(
        preview_with(&app, &[(CF_IP, "203.0.113.7")]).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}
