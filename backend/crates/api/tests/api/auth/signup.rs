//! Cadastro com senha: o pedido guarda o cadastro no Redis e manda o link; a
//! conta só nasce quando o link é aberto.

use api::{AppState, mail::Mail};
use axum::{
    Router,
    http::{Method, StatusCode},
};
use redis::Cmd;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{confirm_signup, send_json, set_email, signup, skip_cooldown};
use crate::common::{app, app_from, err, get, link_token, mailbox, redis, state, user};

const PASSWORD: &str = "senha forte 1";

async fn start(app: &Router, username: &str, email: &str) -> (StatusCode, Value) {
    let body = json!({ "username": username, "password": PASSWORD, "email": email });
    let (status, body, session) = send_json(app, Method::POST, "/auth/signup", None, body).await;
    assert!(session.is_none(), "o pedido não cria sessão");
    (status, body)
}

async fn confirm(app: &Router, token: &str) -> (StatusCode, Value) {
    let body = json!({ "token": token });
    let (status, body, _) = send_json(app, Method::POST, "/auth/signup/confirm", None, body).await;
    (status, body)
}

async fn users(db: &PgPool) -> i64 {
    sqlx::query_scalar!(r#"select count(*) as "n!" from users"#)
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn account_is_created_only_by_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let (status, body) = start(&app, " Rapha ", "Joao.Silva+quake@GoogleMail.com").await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert_eq!(body, json!({ "email": "joaosilva@gmail.com" }));
    assert_eq!(users(&db).await, 0, "nada no banco antes do link");

    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "joaosilva@gmail.com");
    assert!(
        email
            .text
            .contains("http://localhost:4001/pt/signup/confirm#"),
        "{}",
        email.text
    );
    assert!(email.html.contains(&link_token(&email)));

    let me = confirm_signup(&app, &email).await;
    let (_, body) = get(&app, "/me", &me).await;
    assert_eq!(body["display_name"], "Rapha");
    let (_, body) = get(&app, "/auth/logins", &me).await;
    assert_eq!(
        body,
        json!({ "discord": false, "google": false, "username": "Rapha", "email": "joaosilva@gmail.com" })
    );

    // O banco guarda o username como digitado e só o hash da senha.
    let login = sqlx::query!("select user_id, username, password_hash from password_credentials")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!((login.user_id, login.username.as_str()), (me.id, "Rapha"));
    assert!(login.password_hash.starts_with("$argon2id$"));
    assert!(!login.password_hash.contains(PASSWORD));

    // O link vale uma vez, e o cadastro sai do Redis.
    let (status, body) = confirm(&app, &link_token(&email)).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
    let redis = redis(&db);
    for key in ["signup:joaosilva@gmail.com", "signup_username:rapha"] {
        let left: bool = redis.query(&Cmd::exists(redis.key(key))).await.unwrap();
        assert!(!left, "{key}");
    }
    assert_eq!(users(&db).await, 1);
}

#[tokio::test]
async fn signup_uses_the_locale_and_refuses_bad_input() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let body = json!({ "username": "Rapha", "password": PASSWORD, "email": "r@outlook.com", "locale": "es" });
    let (status, _, _) = send_json(&app, Method::POST, "/auth/signup", None, body).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let email = mailbox(&db).next().await;
    assert!(email.text.contains("/es/signup/confirm#"));
    assert!(
        email.subject.contains("Confirma tu email"),
        "{}",
        email.subject
    );
    let me = confirm_signup(&app, &email).await;
    assert_eq!(get(&app, "/me", &me).await.1["locale"], "es");

    for (username, password, address, code) in [
        ("ab", PASSWORD, "a@outlook.com", "auth.username_invalid"),
        ("Bia", "curta", "a@outlook.com", "auth.password_invalid"),
        ("Bia", PASSWORD, "sem-arroba", "email.invalid"),
        ("Bia", PASSWORD, "bia_x@gmail.com", "email.invalid"),
        ("Bia", PASSWORD, "bia@mailinator.com", "email.disposable"),
        ("Bia", PASSWORD, "bia@duck.com", "email.disposable"),
    ] {
        let body = json!({ "username": username, "password": password, "email": address });
        let (status, body, _) = send_json(&app, Method::POST, "/auth/signup", None, body).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{address}");
        assert_eq!(body, err(code), "{address}");
    }
    mailbox(&db).assert_empty().await;
}

#[tokio::test]
async fn username_is_taken_by_accounts_and_pending_signups() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    let (status, body) = start(&app, "RAPHA", "outro@outlook.com").await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.username_taken"))
    );

    // Outro email, com outra senha, não pega o username de um cadastro
    // esperando o link...
    assert_eq!(
        start(&app, "Bia", "biaa@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    let typo = mailbox(&db).next().await;
    let intruder =
        json!({ "username": "bia", "password": "outra senha 9", "email": "x@outlook.com" });
    let (status, body, _) = send_json(&app, Method::POST, "/auth/signup", None, intruder).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.username_taken"))
    );
    mailbox(&db).assert_empty().await;

    // ...mas quem fez o cadastro (a mesma senha) corrige o email digitado
    // errado, e o link do endereço errado para de valer.
    assert_eq!(
        start(&app, "Bia", "bia@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    let fixed = mailbox(&db).next().await;
    assert_eq!(fixed.to, "bia@outlook.com");
    assert_eq!(
        confirm(&app, &link_token(&typo)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
    let me = confirm_signup(&app, &fixed).await;
    assert_eq!(get(&app, "/me", &me).await.1["display_name"], "Bia");
}

#[tokio::test]
async fn asking_again_replaces_the_pending_signup() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    assert_eq!(
        start(&app, "Bia", "bia@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    let first = mailbox(&db).next().await;

    // Logo em seguida, o mesmo endereço espera.
    let (status, body) = start(&app, "Bia", "bia@outlook.com").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "email.too_soon");
    assert!((1..=60).contains(&body["params"]["seconds"].as_i64().unwrap()));

    // Depois da espera, o pedido novo (com outro username) troca o anterior.
    skip_cooldown(&db, "bia@outlook.com").await;
    assert_eq!(
        start(&app, "Bia2", "bia@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    let second = mailbox(&db).next().await;
    assert_eq!(
        confirm(&app, &link_token(&first)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
    // O username do pedido trocado fica livre.
    assert_eq!(
        start(&app, "Bia", "ana@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    mailbox(&db).next().await;

    let me = confirm_signup(&app, &second).await;
    assert_eq!(get(&app, "/me", &me).await.1["display_name"], "Bia2");
}

#[tokio::test]
async fn email_in_use_warns_its_owner_without_revealing_it() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let owner = signup(&app, &db, "Dona", PASSWORD).await;
    set_email(&db, owner.id, "dona@outlook.com").await;

    let (status, body) = start(&app, "Outra", "Dona+x@Outlook.com").await;
    assert_eq!(
        status,
        StatusCode::ACCEPTED,
        "mesma resposta de um cadastro novo"
    );
    assert_eq!(body, json!({ "email": "dona@outlook.com" }));

    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "dona@outlook.com");
    assert!(
        email.subject.contains("Você já tem uma conta"),
        "{}",
        email.subject
    );
    assert!(email.text.contains("/pt/login/forgot"));
    assert!(!email.text.contains('#'), "não leva link de cadastro");
    let pending: bool = redis(&db)
        .query(&Cmd::exists(redis(&db).key("signup:dona@outlook.com")))
        .await
        .unwrap();
    assert!(!pending);

    // Os limites valem igual aos de um email novo: a espera também não revela.
    let (status, body) = start(&app, "Outra", "dona@outlook.com").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "email.too_soon");
    mailbox(&db).assert_empty().await;

    // Conta só com Discord: o aviso leva ao login, não à recuperação.
    let discord = user(&db, "Discord").await;
    set_email(&db, discord.id, "discord@outlook.com").await;
    assert_eq!(
        start(&app, "Outra", "discord@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;
    assert!(email.text.contains("Discord"), "{}", email.text);
    assert!(
        email.text.contains("http://localhost:4001/pt/login\n"),
        "{}",
        email.text
    );
}

#[tokio::test]
async fn daily_limit_stops_sending() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(AppState {
        mail: Mail::new(mailbox(&db), 2),
        ..state(db.clone())
    });
    for n in 0..2 {
        let (status, body) = start(&app, &format!("user{n}"), &format!("u{n}@outlook.com")).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{body}");
        mailbox(&db).next().await;
    }
    let (status, body) = start(&app, "maisum", "maisum@outlook.com").await;
    assert_eq!(
        (status, body),
        (StatusCode::SERVICE_UNAVAILABLE, err("email.unavailable"))
    );
    mailbox(&db).assert_empty().await;
}

#[tokio::test]
async fn confirm_refuses_email_taken_in_between() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    assert_eq!(
        start(&app, "Bia", "bia@outlook.com").await.0,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;

    let other = user(&db, "Outra").await;
    set_email(&db, other.id, "bia@outlook.com").await;
    assert_eq!(
        confirm(&app, &link_token(&email)).await,
        (StatusCode::CONFLICT, err("auth.email_taken"))
    );
    assert_eq!(users(&db).await, 1, "a conta recusada não fica para trás");
    assert_eq!(
        confirm(&app, "nao-existe").await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
}
