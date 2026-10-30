//! A configuração do owner: só ele vê e muda, só o endereço do Discord entra,
//! e o endereço (com o token) nunca volta pela API.

use axum::http::{Method, StatusCode};
use serde_json::json;

use super::*;
use crate::common::*;

#[tokio::test]
async fn only_the_owner_reaches_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let fora = user(&db, "Fora").await;
    let calls = [
        (Method::GET, URI.to_owned(), None),
        (Method::PUT, URI.to_owned(), Some(config(&["match.ready"]))),
        (Method::DELETE, URI.to_owned(), None),
        (Method::POST, format!("{URI}/test"), None),
    ];
    for (method, uri, body) in calls {
        for (who, expected) in [
            (&bia, err("tenant.forbidden")),
            (&olga, err("tenant.forbidden")),
            (&fora, err("tenant.not_found")),
        ] {
            let (status, got) = call(&app, method.clone(), &uri, Some(who), body.clone()).await;
            assert_eq!(got, expected, "{method} {uri}");
            assert!(status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND);
        }
        let (status, _) = call(&app, method.clone(), &uri, None, body).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn a_new_community_has_no_webhook_and_lists_the_notices() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let (status, body) = get(&app, URI, &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["webhook"], json!(null));
    let available: Vec<(&str, bool)> = body["available"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["code"].as_str().unwrap(),
                e["default_on"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        available,
        [
            ("check_in.opened", true),
            ("tournament.started", true),
            ("match.ready", true),
            ("match.opponent_ready", true),
            ("match.veto_started", false),
            ("tournament.start_due", true),
            ("match.game_contested", true),
            ("tournament.finished", true),
            ("match.series_decided", false),
        ]
    );
}

#[tokio::test]
async fn saving_never_returns_the_address_or_the_token() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let events = ["match.ready", "tournament.finished", "match.ready"];
    let (status, put_body) = put(&app, URI, &ana, config(&events)).await;
    assert_eq!(status, StatusCode::OK, "{put_body}");
    // Sem repetir e na ordem do catálogo.
    assert_eq!(
        put_body["webhook"],
        json!({
            "webhook_id": WEBHOOK,
            "broken": false,
            "events": ["match.ready", "tournament.finished"],
            "locale": "pt",
            "mention_players": true
        })
    );
    let (_, got) = get(&app, URI, &ana).await;
    assert_eq!(got, put_body);
    for body in [put_body, got] {
        let text = body.to_string();
        assert!(
            !text.contains(TOKEN) && !text.contains("discord.com"),
            "{text}"
        );
    }
}

#[tokio::test]
async fn only_discord_addresses_are_accepted() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let bad = [
        "https://example.com/api/webhooks/1/abcdefghijklmnopqrstuvwxyz",
        "http://discord.com/api/webhooks/1/abcdefghijklmnopqrstuvwxyz",
        "https://127.0.0.1/api/webhooks/1/abcdefghijklmnopqrstuvwxyz",
        "https://discord.com.evil.io/api/webhooks/1/abcdefghijklmnopqrstuvwxyz",
        "https://discord.com/api/webhooks/1/abc",
        "não é um endereço",
        "",
    ];
    for url in bad {
        let body = with(config(&["match.ready"]), json!({ "url": url }));
        assert_eq!(
            put(&app, URI, &ana, body).await,
            (StatusCode::UNPROCESSABLE_ENTITY, err("discord.url_invalid")),
            "{url}"
        );
    }
    for host in ["ptb.discord.com", "canary.discord.com", "discordapp.com"] {
        let url = format!("https://{host}/api/webhooks/{WEBHOOK}/{TOKEN}");
        let body = with(config(&["match.ready"]), json!({ "url": url }));
        assert_eq!(put(&app, URI, &ana, body).await.0, StatusCode::OK, "{host}");
    }
}

#[tokio::test]
async fn unknown_notices_and_a_missing_address_are_refused() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let body = with(config(&["match.ready", "nope.nope"]), json!({}));
    assert_eq!(
        put(&app, URI, &ana, body).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("discord.event_unknown")
        )
    );
    // Sem webhook ainda, só a configuração não basta.
    let body = with(config(&["match.ready"]), json!({ "url": null }));
    assert_eq!(
        put(&app, URI, &ana, body).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("discord.url_required")
        )
    );
}

#[tokio::test]
async fn changing_the_choices_keeps_the_webhook() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    put(&app, URI, &ana, config(&["match.ready"])).await;
    let body = json!({ "events": ["check_in.opened"], "locale": "es", "mention_players": false });
    let (status, got) = put(&app, URI, &ana, body).await;
    assert_eq!(status, StatusCode::OK, "{got}");
    assert_eq!(
        got["webhook"],
        json!({
            "webhook_id": WEBHOOK,
            "broken": false,
            "events": ["check_in.opened"],
            "locale": "es",
            "mention_players": false
        })
    );
    // O token continua o mesmo.
    let stored: String = sqlx::query_scalar("select token from discord_webhooks")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(stored, TOKEN);
}

#[tokio::test]
async fn removing_it_clears_the_queue() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.ready"]).await;
    sqlx::query(
        "insert into discord_messages (tenant_id, code, params, path) \
         select tenant_id, 'match.ready', '{}', '/quake-br/copa' from discord_webhooks",
    )
    .execute(&db)
    .await
    .unwrap();
    let (status, _) = delete(&s.app, URI, &s.ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(get(&s.app, URI, &s.ana).await.1["webhook"], json!(null));
    let queued: i64 = sqlx::query_scalar("select count(*) from discord_messages")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(queued, 0);
    // Sem webhook, apagar de novo e testar.
    assert_eq!(delete(&s.app, URI, &s.ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(
        post_empty(&s.app, &format!("{URI}/test"), &s.ana).await,
        (StatusCode::NOT_FOUND, err("discord.not_configured"))
    );
}

#[tokio::test]
async fn the_test_message_reaches_the_channel_in_the_chosen_language() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.ready"]).await;
    let (status, body) = post_empty(&s.app, &format!("{URI}/test"), &s.ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    let hits = s.fake.hits();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, format!("/api/webhooks/{WEBHOOK}/{TOKEN}"));
    assert_eq!(
        hits[0].body["content"],
        "Teste de **Tenant quake-br**. Os avisos da comunidade vão chegar neste canal.\nhttp://localhost:4001/pt/quake-br"
    );
    assert_eq!(
        hits[0].body["allowed_mentions"],
        json!({ "parse": [], "users": [] })
    );

    let es = json!({ "events": [], "locale": "es", "mention_players": true });
    put(&s.app, URI, &s.ana, es).await;
    post_empty(&s.app, &format!("{URI}/test"), &s.ana).await;
    assert!(s.fake.texts()[1].starts_with("Prueba de **Tenant quake-br**"));
}

#[tokio::test]
async fn a_failing_test_says_why() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.ready"]).await;
    let uri = format!("{URI}/test");
    s.fake
        .reply(&[Reply::TooMany(2.5), Reply::Status(500), Reply::Status(400)]);
    assert_eq!(
        post_empty(&s.app, &uri, &s.ana).await,
        (
            StatusCode::TOO_MANY_REQUESTS,
            json!({ "error": "discord.rate_limited", "params": { "retry_after": 3 } })
        )
    );
    assert_eq!(
        post_empty(&s.app, &uri, &s.ana).await,
        (StatusCode::BAD_GATEWAY, err("discord.unreachable"))
    );
    assert_eq!(
        post_empty(&s.app, &uri, &s.ana).await,
        (StatusCode::BAD_GATEWAY, err("discord.rejected"))
    );
    // Nada disso quebra o webhook.
    assert_eq!(get(&s.app, URI, &s.ana).await.1["webhook"]["broken"], false);
}

#[tokio::test]
async fn a_deleted_webhook_is_marked_broken_until_another_is_pasted() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.ready"]).await;
    s.fake.reply(&[Reply::Status(404)]);
    assert_eq!(
        post_empty(&s.app, &format!("{URI}/test"), &s.ana).await,
        (StatusCode::CONFLICT, err("discord.webhook_gone"))
    );
    assert_eq!(get(&s.app, URI, &s.ana).await.1["webhook"]["broken"], true);

    let (_, body) = put(&s.app, URI, &s.ana, config(&["match.ready"])).await;
    assert_eq!(body["webhook"]["broken"], false);
}
