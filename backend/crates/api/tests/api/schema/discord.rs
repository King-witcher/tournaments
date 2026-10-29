//! Avisos no Discord: a forma do webhook e da fila, as mensagens juntas por
//! `merge_key` e as cascatas (a comunidade apaga o webhook, o webhook apaga a
//! fila). O evento `announce` do outbox também passa pela forma.

use sqlx::PgPool;

use super::*;
use crate::common::*;

const TOKEN: &str = "abcdefghijklmnopqrstuvwxyz0123456789";

fn webhook(tenant: i64, id: &str, token: &str, events: &str, locale: &str) -> String {
    format!(
        "insert into discord_webhooks (tenant_id, webhook_id, token, events, mention_players, locale)
         values ({tenant}, '{id}', '{token}', {events}, true, '{locale}')"
    )
}

async fn tenant_id(db: &PgPool) -> i64 {
    count(db, "select id from tenants where slug = 'quake-br'").await
}

#[tokio::test]
async fn webhooks_keep_their_format() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, _, _) = quake_br(&db).await;
    let t = tenant_id(&db).await;
    let ok_events = "array['match.ready']";
    for (sql, name) in [
        (
            webhook(t, "12a", TOKEN, ok_events, "pt"),
            "discord_webhooks_id_format",
        ),
        (
            webhook(t, "", TOKEN, ok_events, "pt"),
            "discord_webhooks_id_format",
        ),
        (
            webhook(t, "1", "curto", ok_events, "pt"),
            "discord_webhooks_token_format",
        ),
        (
            webhook(t, "1", "abc def ghi jkl mno pqr", ok_events, "pt"),
            "discord_webhooks_token_format",
        ),
        (
            webhook(t, "1", TOKEN, ok_events, "en"),
            "discord_webhooks_locale_check",
        ),
        (
            webhook(t, "1", TOKEN, "array['Match.Ready']", "pt"),
            "discord_webhooks_events_valid",
        ),
        (
            webhook(t, "1", TOKEN, "array['semponto']", "pt"),
            "discord_webhooks_events_valid",
        ),
        (
            webhook(t, "1", TOKEN, "array['a.b', 'a.b']", "pt"),
            "discord_webhooks_events_valid",
        ),
        (
            webhook(t, "1", TOKEN, "array[null]::text[]", "pt"),
            "discord_webhooks_events_valid",
        ),
        (
            webhook(t, "1", TOKEN, "null", "pt"),
            "discord_webhooks_events_valid",
        ),
    ] {
        let got = rejected(exec(&db, &sql).await);
        // `null` cai no not null antes do check.
        if sql.contains(", null,") {
            assert_eq!(got.0, "23502");
        } else {
            assert_eq!(got, rule(CHECK, name), "{sql}");
        }
    }
    // Sem avisos ligados vale; um webhook por comunidade.
    exec(&db, &webhook(t, "1", TOKEN, "'{}'", "es"))
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, &webhook(t, "2", TOKEN, ok_events, "pt")).await),
        rule(UNIQUE, "discord_webhooks_pkey")
    );
}

#[tokio::test]
async fn messages_keep_their_format_and_merge_by_key() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, _, _) = quake_br(&db).await;
    let t = tenant_id(&db).await;
    exec(&db, &webhook(t, "1", TOKEN, "'{}'", "pt"))
        .await
        .unwrap();
    let insert = |code: &str, params: &str, path: &str, key: &str, attempts: i32| {
        format!(
            "insert into discord_messages (tenant_id, code, params, path, merge_key, attempts)
             values ({t}, '{code}', '{params}', '{path}', {key}, {attempts})"
        )
    };
    for (sql, name) in [
        (
            insert("semponto", "{}", "/a/b", "null", 0),
            "discord_messages_code_format",
        ),
        (
            insert("a.b", "[]", "/a/b", "null", 0),
            "discord_messages_params_object",
        ),
        (
            insert("a.b", "{}", "a/b", "null", 0),
            "discord_messages_path_format",
        ),
        (
            insert("a.b", "{}", "/a b", "null", 0),
            "discord_messages_path_format",
        ),
        (
            insert("a.b", "{}", "/a/b", "null", 21),
            "discord_messages_attempts_range",
        ),
    ] {
        assert_eq!(rejected(exec(&db, &sql).await), rule(CHECK, name), "{sql}");
    }
    // Sem chave, as mensagens são independentes; com a mesma chave, uma só.
    exec(&db, &insert("a.b", "{}", "/a/b", "null", 0))
        .await
        .unwrap();
    exec(&db, &insert("a.b", "{}", "/a/b", "null", 0))
        .await
        .unwrap();
    exec(&db, &insert("a.b", "{}", "/a/b", "'a.b:/a/b'", 0))
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, &insert("a.b", "{}", "/a/b", "'a.b:/a/b'", 0)).await),
        rule(UNIQUE, "discord_messages_merge")
    );
}

#[tokio::test]
async fn the_webhook_goes_with_the_community_and_takes_the_queue() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, _, _) = quake_br(&db).await;
    let t = tenant_id(&db).await;
    exec(&db, &webhook(t, "1", TOKEN, "'{}'", "pt"))
        .await
        .unwrap();
    exec(
        &db,
        &format!(
            "insert into discord_messages (tenant_id, code, params, path)
             values ({t}, 'a.b', '{{}}', '/a/b')"
        ),
    )
    .await
    .unwrap();

    // Apagar o webhook leva a fila.
    exec(
        &db,
        &format!("delete from discord_webhooks where tenant_id = {t}"),
    )
    .await
    .unwrap();
    assert_eq!(count(&db, "select count(*) from discord_messages").await, 0);

    // Fila sem webhook não existe.
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!(
                    "insert into discord_messages (tenant_id, code, params, path)
                     values ({t}, 'a.b', '{{}}', '/a/b')"
                )
            )
            .await
        ),
        rule(FOREIGN_KEY, "discord_messages_tenant_id_fkey")
    );

    // Apagar a comunidade leva o webhook e a fila.
    exec(&db, &webhook(t, "1", TOKEN, "'{}'", "pt"))
        .await
        .unwrap();
    exec(
        &db,
        &format!(
            "insert into discord_messages (tenant_id, code, params, path)
             values ({t}, 'a.b', '{{}}', '/a/b')"
        ),
    )
    .await
    .unwrap();
    exec(&db, &format!("delete from tenants where id = {t}"))
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from discord_webhooks").await, 0);
    assert_eq!(count(&db, "select count(*) from discord_messages").await, 0);
}

#[tokio::test]
async fn the_outbox_accepts_announcements() {
    let (db, _lease) = crate::db::test_db().await;
    let event =
        r#"'{"kind": "announce", "code": "tournament.started", "params": {}, "path": "/a/b"}'"#;
    exec(&db, &format!("insert into outbox (event) values ({event})"))
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from outbox").await, 1);
}
