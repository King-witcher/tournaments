//! Limite por IP dos pedidos sem login que mandam email: o cadastro (e o
//! reenvio, que é pedir de novo) e entrar por email, somados.

use axum::{
    Router,
    http::{HeaderValue, Method, StatusCode},
};
use serde_json::{Value, json};

use super::{body_text, json_request, send};
use crate::common::{app, err, mailbox};

/// O cabeçalho que o Cloudflare do Render põe com o IP de quem pede.
const IP_HEADER: &str = "cf-connecting-ip";

async fn from_ip(app: &Router, ip: &str, uri: &str, body: Value) -> (StatusCode, Value) {
    let mut req = json_request(Method::POST, uri, None, body);
    req.headers_mut()
        .insert(IP_HEADER, HeaderValue::from_str(ip).unwrap());
    let res = send(app, req).await;
    let status = res.status();
    let text = body_text(res).await;
    let json = if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap()
    };
    (status, json)
}

async fn signup(app: &Router, ip: &str, name: &str) -> (StatusCode, Value) {
    let email = format!("{}@outlook.com", name.to_lowercase());
    let body = json!({ "display_name": name, "password": "senha forte 1", "email": email });
    from_ip(app, ip, "/auth/signup", body).await
}

async fn forgot(app: &Router, ip: &str, email: &str) -> (StatusCode, Value) {
    from_ip(app, ip, "/auth/email/login", json!({ "email": email })).await
}

fn assert_limited((status, body): (StatusCode, Value), max_minutes: i64) {
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "{body}");
    assert_eq!(body["error"], "email.ip_limited");
    let minutes = body["params"]["minutes"].as_i64().unwrap();
    assert!((1..=max_minutes).contains(&minutes), "{minutes}");
}

#[tokio::test]
async fn signups_and_email_logins_share_five_per_hour() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ip = "203.0.113.7";

    // Pedido recusado na validação não conta.
    assert_eq!(
        signup(&app, ip, " ").await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("auth.display_name_invalid")
        )
    );
    for name in ["Ana", "Bia", "Caio"] {
        assert_eq!(signup(&app, ip, name).await.0, StatusCode::ACCEPTED);
        mailbox(&db).next().await;
    }
    // Entrar por email conta mesmo sem conta (a resposta não diz nada).
    assert_eq!(
        forgot(&app, ip, "ninguem@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    assert_eq!(
        forgot(&app, ip, "ana@outlook.com").await.0,
        StatusCode::ACCEPTED
    );

    assert_limited(signup(&app, ip, "Duda").await, 60);
    assert_limited(forgot(&app, ip, "ninguem@outlook.com").await, 60);
    mailbox(&db).assert_empty().await;

    // Outro IP segue; o mesmo /64 do IPv6 divide o limite.
    assert_eq!(
        signup(&app, "198.51.100.1", "Duda").await.0,
        StatusCode::ACCEPTED
    );
    for n in 0..5 {
        let ip = format!("2001:db8:1:2::{n}");
        assert_eq!(
            forgot(&app, &ip, "ninguem@outlook.com").await.0,
            StatusCode::ACCEPTED
        );
    }
    assert_limited(
        forgot(&app, "2001:db8:1:2:ffff::1", "ninguem@outlook.com").await,
        60,
    );
}

/// A troca de email exige login, mas gasta o mesmo teto do dia.
#[tokio::test]
async fn changing_the_email_counts_for_the_ip() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let ip = "203.0.113.9";
    for _ in 0..5 {
        assert_eq!(
            forgot(&app, ip, "ninguem@outlook.com").await.0,
            StatusCode::ACCEPTED
        );
    }
    let me = crate::common::user(&db, "Quaker").await;
    super::add_discord(&db, me.id).await;
    let body = json!({ "email": "q@outlook.com" });
    let mut req = json_request(Method::PUT, "/auth/email", Some(&me.cookie), body);
    req.headers_mut()
        .insert(IP_HEADER, HeaderValue::from_str(ip).unwrap());
    let res = send(&app, req).await;
    let status = res.status();
    let json: Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_limited((status, json), 60);
}
