//! O código do email de cadastro: cria a conta no lugar do link, vale o que
//! vier primeiro, e morre depois de 5 erros (o link continua). Toda recusa é a
//! mesma, para não revelar quem tem conta.

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
async fn start(app: &Router, db: &PgPool, name: &str, address: &str) -> Email {
    let body = json!({ "display_name": name, "password": PASSWORD, "email": address });
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

/// A única resposta de um código que não cria a conta.
fn rejected() -> (StatusCode, Value) {
    (StatusCode::UNPROCESSABLE_ENTITY, err("auth.code_rejected"))
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
    assert_eq!((status, body), rejected());

    // E ao contrário: aberto o link, o código não vale.
    let email = start(&app, &db, "Bia", "bia@outlook.com").await;
    assert_eq!(open_link(&app, &email).await.0, StatusCode::CREATED);
    let (status, body, session) = send_code(&app, "bia@outlook.com", &code_of(&email)).await;
    assert_eq!((status, body), rejected());
    assert_eq!(session, None);
}

#[tokio::test]
async fn five_wrong_codes_kill_the_code_but_not_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let email = start(&app, &db, "Rapha", "rapha@outlook.com").await;
    let code = code_of(&email);
    let wrong = other_code(&code);

    for _ in 0..5 {
        let (status, body, session) = send_code(&app, "rapha@outlook.com", &wrong).await;
        assert_eq!((status, body), rejected());
        assert_eq!(session, None);
    }
    // Esgotado, nem o certo entra.
    let (status, body, _) = send_code(&app, "rapha@outlook.com", &code).await;
    assert_eq!((status, body), rejected());

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
    assert_eq!((status, body), rejected());

    // O código de um cadastro não serve a outro email.
    let email = start(&app, &db, "Rapha", "rapha@outlook.com").await;
    let (status, body, _) = send_code(&app, "outro@outlook.com", &code_of(&email)).await;
    assert_eq!((status, body), rejected());
}

/// Quem pede o cadastro de um email e testa um código não descobre se ele tem
/// conta: com conta não há pedido; sem conta, há e o código está errado.
#[tokio::test]
async fn code_answers_the_same_with_or_without_an_account() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    start(&app, &db, "Dona", "dona@outlook.com").await;
    let email = start(&app, &db, "Nova", "nova@outlook.com").await;
    let (status, _, _) = send_code(&app, "nova@outlook.com", &code_of(&email)).await;
    assert_eq!(status, StatusCode::CREATED);

    let body =
        json!({ "display_name": "Sonda", "password": PASSWORD, "email": "outra@outlook.com" });
    send_json(&app, Method::POST, "/auth/signup", None, body).await;
    let free = other_code(&code_of(&mailbox(&db).next().await));
    let (status, body, _) = send_code(&app, "outra@outlook.com", &free).await;
    assert_eq!((status, body), rejected());

    super::skip_cooldown(&db, "nova@outlook.com").await;
    let body =
        json!({ "display_name": "Sonda", "password": PASSWORD, "email": "nova@outlook.com" });
    send_json(&app, Method::POST, "/auth/signup", None, body).await;
    mailbox(&db).next().await; // o aviso à dona
    let (status, body, _) = send_code(&app, "nova@outlook.com", "123456").await;
    assert_eq!((status, body), rejected());
}
