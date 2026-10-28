//! Web Push: a chave, as inscrições de cada navegador e o envio pelo
//! despachante, com um serviço de push de mentira.

use std::sync::{Arc, Mutex};

use api::{
    push::{Sent, Target, WebPush},
    realtime,
};
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::COPA;

const P256DH: &str =
    "BLn9b-VR0ca83knDNZ32dCHGyjJp-1riX9ZTN40MqV8K_LpQmLqxC_DoHvqvFXO_nGdAB4W9dogZb_sM-uV4JbY";
const AUTH: &str = "_ordMnz7uTCmrpBTeUV4Bw";

/// Guarda o que mandaria; endpoint com `gone` responde como inscrição que não
/// existe mais, e com `down`, como erro de rede.
#[derive(Default)]
struct FakePush {
    sent: Mutex<Vec<(String, Value)>>,
}

#[async_trait]
impl WebPush for FakePush {
    fn public_key(&self) -> &str {
        "chave-publica"
    }

    async fn send(&self, to: &Target, payload: &[u8]) -> anyhow::Result<Sent> {
        let payload = serde_json::from_slice(payload).unwrap();
        self.sent
            .lock()
            .unwrap()
            .push((to.endpoint.clone(), payload));
        if to.endpoint.contains("down") {
            anyhow::bail!("sem rede");
        }
        Ok(if to.endpoint.contains("gone") {
            Sent::Gone
        } else {
            Sent::Delivered
        })
    }
}

fn subscription(endpoint: &str) -> Value {
    subscription_in(endpoint, "es")
}

fn subscription_in(endpoint: &str, locale: &str) -> Value {
    json!({ "endpoint": endpoint, "keys": { "p256dh": P256DH, "auth": AUTH }, "locale": locale })
}

async fn owners(db: &PgPool) -> Vec<(String, i64, String)> {
    sqlx::query_as("select endpoint, user_id, locale from push_subscriptions order by endpoint")
        .fetch_all(db)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_key_needs_vapid(db: PgPool) {
    let ana = user(&db, "Ana").await;
    assert_eq!(
        get(&app(db.clone()), "/push/key", &ana).await,
        (StatusCode::SERVICE_UNAVAILABLE, err("push.not_configured"))
    );
    let with_push = app_from(api::AppState {
        push: Some(Arc::new(FakePush::default())),
        ..state(db.clone())
    });
    assert_eq!(
        get(&with_push, "/push/key", &ana).await,
        (StatusCode::OK, json!({ "public_key": "chave-publica" }))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_browser_belongs_to_whoever_subscribed_last(db: PgPool) {
    let app = app(db.clone());
    let (ana, bia) = (user(&db, "Ana").await, user(&db, "Bia").await);
    let endpoint = "https://fcm.googleapis.com/fcm/send/abc";
    let (status, _) = put(&app, "/push/subscriptions", &bia, subscription(endpoint)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(owners(&db).await, [(endpoint.into(), bia.id, "es".into())]);

    // Outra conta entrou no mesmo navegador.
    let pt =
        json!({ "endpoint": endpoint, "keys": { "p256dh": P256DH, "auth": AUTH }, "locale": "pt" });
    let (status, _) = put(&app, "/push/subscriptions", &ana, pt).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(owners(&db).await, [(endpoint.into(), ana.id, "pt".into())]);

    // Só o dono tira.
    let body = json!({ "endpoint": endpoint });
    let remove = |who| {
        call(
            &app,
            Method::DELETE,
            "/push/subscriptions",
            Some(who),
            Some(body.clone()),
        )
    };
    assert_eq!(remove(&bia).await.0, StatusCode::NO_CONTENT);
    assert_eq!(owners(&db).await.len(), 1);
    assert_eq!(remove(&ana).await.0, StatusCode::NO_CONTENT);
    assert_eq!(owners(&db).await, []);

    for bad in [
        subscription("http://fcm.googleapis.com/x"),
        subscription("https://127.0.0.1/x"),
        json!({ "endpoint": endpoint, "keys": { "p256dh": "curta", "auth": AUTH }, "locale": "pt" }),
    ] {
        assert_eq!(
            put(&app, "/push/subscriptions", &ana, bad).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("push.subscription_invalid")
            )
        );
    }
    let english =
        json!({ "endpoint": endpoint, "keys": { "p256dh": P256DH, "auth": AUTH }, "locale": "en" });
    assert_eq!(
        put(&app, "/push/subscriptions", &ana, english).await.1,
        err("request.invalid_body")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_dispatcher_pushes_notices(db: PgPool) {
    let fake = Arc::new(FakePush::default());
    let state = api::AppState {
        push: Some(fake.clone()),
        ..state(db.clone())
    };
    let app = app_from(state.clone());
    let (ana, bia) = (user(&db, "Ana").await, user(&db, "Bia").await);
    create(&app, &ana, "quake-br").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    // Cada navegador com o idioma de quando se inscreveu.
    for (endpoint, locale) in [
        ("https://a.push.example/ok", "pt"),
        ("https://a.push.example/gone", "es"),
        ("https://a.push.example/down", "es"),
    ] {
        let body = subscription_in(endpoint, locale);
        let (status, _) = put(&app, "/push/subscriptions", &bia, body).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }
    let (status, _) = put(
        &app,
        "/push/subscriptions",
        &ana,
        subscription("https://a.push.example/ana"),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    realtime::dispatch_once(&state).await.unwrap();
    assert!(fake.sent.lock().unwrap().is_empty());

    let (status, _) = post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    realtime::dispatch_once(&state).await.unwrap();
    // O aviso com o idioma de cada inscrição, para o service worker traduzir.
    let payload = |locale: &str| {
        json!({
            "code": "check_in.opened",
            "params": { "tournament": "Copa de Outubro" },
            "path": "/quake-br/copa",
            "locale": locale
        })
    };
    let sent = fake.sent.lock().unwrap().clone();
    assert_eq!(
        sent,
        [
            ("https://a.push.example/ok".to_owned(), payload("pt")),
            ("https://a.push.example/gone".to_owned(), payload("es")),
            ("https://a.push.example/down".to_owned(), payload("es")),
        ]
    );
    // A que não existe mais sai; a da falha de rede fica; a entregue fica
    // marcada.
    let left: Vec<(String, bool)> = sqlx::query_as(
        "select endpoint, last_used_at is not null from push_subscriptions where user_id = $1 order by id",
    )
    .bind(bia.id)
    .fetch_all(&db)
    .await
    .unwrap();
    assert_eq!(
        left,
        [
            ("https://a.push.example/ok".to_owned(), true),
            ("https://a.push.example/down".to_owned(), false),
        ]
    );
}
