//! Tempo real: a forma e o "uma vez por transação" do outbox, o formato das
//! inscrições de Web Push (que saem com o usuário) e a marca do aviso de hora
//! de iniciar.

use sqlx::PgPool;

use super::*;
use crate::common::*;

const P256DH: &str =
    "BLn9b-VR0ca83knDNZ32dCHGyjJp-1riX9ZTN40MqV8K_LpQmLqxC_DoHvqvFXO_nGdAB4W9dogZb_sM-uV4JbY";
const AUTH: &str = "_ordMnz7uTCmrpBTeUV4Bw";

#[sqlx::test(migrations = "../../migrations")]
async fn outbox_shape_and_once_per_transaction(db: PgPool) {
    for bad in [
        r#"'{"kind": "outro"}'"#,
        r#"'{"tenant": "x"}'"#,
        "'[]'",
        "'1'",
    ] {
        assert_eq!(
            rejected(exec(&db, &format!("insert into outbox (event) values ({bad})")).await),
            rule(CHECK, "outbox_event_shape")
        );
    }

    let event = r#"'{"kind": "tenant", "tenant": "quake-br"}'"#;
    let insert = format!("insert into outbox (event) values ({event})");
    let mut tx = db.begin().await.unwrap();
    let skip = format!("{insert} on conflict (xact, (md5(event::text))) do nothing");
    for _ in 0..2 {
        sqlx::query(sqlx::AssertSqlSafe(skip.clone()))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    let again = sqlx::query(sqlx::AssertSqlSafe(insert.clone()))
        .execute(&mut *tx)
        .await;
    assert_eq!(rejected(again), rule(UNIQUE, "outbox_once_per_xact"));
    tx.rollback().await.unwrap();

    // Em transações diferentes, o mesmo evento sai duas vezes.
    exec(&db, &insert).await.unwrap();
    exec(&db, &insert).await.unwrap();
    assert_eq!(count(&db, "select count(*) from outbox").await, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn push_subscriptions_keep_their_format(db: PgPool) {
    let ana = user(&db, "Ana").await;
    let insert = |endpoint: &str, p256dh: &str, auth: &str, locale: &str| {
        format!(
            "insert into push_subscriptions (user_id, endpoint, p256dh, auth, locale)
             values ({}, '{endpoint}', '{p256dh}', '{auth}', '{locale}')",
            ana.id
        )
    };
    let ok = "https://fcm.googleapis.com/fcm/send/abc";
    for (sql, name) in [
        (
            insert("http://x.com/a", P256DH, AUTH, "pt"),
            "push_subscriptions_endpoint_format",
        ),
        (
            insert("https://x.com/a b", P256DH, AUTH, "pt"),
            "push_subscriptions_endpoint_format",
        ),
        (
            insert(ok, &P256DH[1..], AUTH, "pt"),
            "push_subscriptions_p256dh_format",
        ),
        (
            insert(ok, P256DH, "curta", "pt"),
            "push_subscriptions_auth_format",
        ),
        (
            insert(ok, P256DH, AUTH, "en"),
            "push_subscriptions_locale_check",
        ),
    ] {
        assert_eq!(rejected(exec(&db, &sql).await), rule(CHECK, name), "{sql}");
    }
    exec(&db, &insert(ok, P256DH, AUTH, "pt")).await.unwrap();
    assert_eq!(
        rejected(exec(&db, &insert(ok, P256DH, AUTH, "es")).await),
        rule(UNIQUE, "push_subscriptions_endpoint_key")
    );

    // As inscrições saem com o usuário.
    exec(&db, &format!("delete from users where id = {}", ana.id))
        .await
        .unwrap();
    assert_eq!(
        count(&db, "select count(*) from push_subscriptions").await,
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_published_tournaments_are_reminded(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(
        rejected(exec(&db, "update tournaments set start_reminded_at = now()").await),
        rule(CHECK, "tournaments_start_reminded_published")
    );
    publish(&app, &ana, "copa").await;
    exec(&db, "update tournaments set start_reminded_at = now()")
        .await
        .unwrap();
}
