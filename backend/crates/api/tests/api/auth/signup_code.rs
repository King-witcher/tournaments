//! O código do email de cadastro: cria a conta no lugar do link, vale o que
//! vier primeiro, e morre depois de 5 erros (o link continua).

use api::mail::Email;
use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::send_json;
use crate::common::{User, app, err, get, link_token, mailbox};

const PASSWORD: &str = "senha forte 1";

/// Pede o cadastro e devolve o email que saiu.
async fn start(app: &Router, db: &PgPool, username: &str, address: &str) -> Email {
    let body = json!({ "username": username, "password": PASSWORD, "email": address });
    let (status, body, _) = send_json(app, Method::POST, "/auth/signup", None, body).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    mailbox(db).next().await
}

/// O código do email: a linha só com os 6 dígitos.
fn code_of(email: &Email) -> String {
    email
        .text
        .lines()
        .find(|line| line.len() == 6 && line.bytes().all(|b| b.is_ascii_digit()))
        .expect("email sem código")
        .to_owned()
}

/// Um código de 6 dígitos que não é `code`.
fn other_code(code: &str) -> String {
    let n: u32 = code.parse().unwrap();
    format!("{:06}", (n + 1) % 1_000_000)
}

async fn send_code(app: &Router, email: &str, code: &str) -> (StatusCode, Value, Option<String>) {
    let body = json!({ "email": email, "code": code });
    send_json(app, Method::POST, "/auth/signup/code", None, body).await
}

async fn open_link(app: &Router, email: &Email) -> (StatusCode, Value) {
    let body = json!({ "token": link_token(email) });
    let (status, body, _) = send_json(app, Method::POST, "/auth/signup/confirm", None, body).await;
    (status, body)
}

#[tokio::test]
async fn code_creates_the_account_and_the_link_stops_working() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let email = start(&app, &db, "Rapha", "rapha@outlook.com").await;
    let code = code_of(&email);
    assert!(
        email.html.contains(&format!(">{code}</td>")),
        "código no HTML"
    );

    // O email como foi digitado também vale: a API o põe na forma canônica.
    let (status, body, session) = send_code(&app, "Rapha@Outlook.com", &code).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let me = User {
        id: body["id"].as_i64().unwrap(),
        cookie: session.expect("entra na conta"),
    };
    assert_eq!(get(&app, "/me", &me).await.1["display_name"], "Rapha");

    // Valeu o código: o link e o próprio código não valem mais.
    assert_eq!(
        open_link(&app, &email).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
    let (status, body, _) = send_code(&app, "rapha@outlook.com", &code).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.code_expired")));

    // E ao contrário: aberto o link, o código não vale.
    let email = start(&app, &db, "Bia", "bia@outlook.com").await;
    assert_eq!(open_link(&app, &email).await.0, StatusCode::CREATED);
    let (status, body, session) = send_code(&app, "bia@outlook.com", &code_of(&email)).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.code_expired")));
    assert_eq!(session, None);
}

#[tokio::test]
async fn five_wrong_codes_kill_the_code_but_not_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let email = start(&app, &db, "Rapha", "rapha@outlook.com").await;
    let code = code_of(&email);
    let wrong = other_code(&code);

    for left in (0..5).rev() {
        let (status, body, session) = send_code(&app, "rapha@outlook.com", &wrong).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(
            body,
            json!({ "error": "auth.code_invalid", "params": { "attempts_left": left } })
        );
        assert_eq!(session, None);
    }
    // Esgotado, nem o certo entra.
    let (status, body, _) = send_code(&app, "rapha@outlook.com", &code).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.code_expired")));

    // O link continua valendo.
    assert_eq!(open_link(&app, &email).await.0, StatusCode::CREATED);
}

#[tokio::test]
async fn code_needs_six_digits_and_a_pending_signup() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    for code in ["12345", "1234567", "12a456", " 123456"] {
        let (status, body, _) = send_code(&app, "rapha@outlook.com", code).await;
        assert_eq!(
            (status, body),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("request.invalid_body")
            ),
            "{code}"
        );
    }
    let (status, body, _) = send_code(&app, "ninguem@outlook.com", "123456").await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.code_expired")));

    // O código de um cadastro não serve a outro email.
    let email = start(&app, &db, "Rapha", "rapha@outlook.com").await;
    let (status, body, _) = send_code(&app, "outro@outlook.com", &code_of(&email)).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.code_expired")));
}
