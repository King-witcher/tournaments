//! Recuperação de senha pelo email da conta.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{send_json, session_count, set_email, signup, skip_cooldown};
use crate::common::{User, app, err, get, link_token, mailbox, user};

const PASSWORD: &str = "senha forte 1";
const NEW_PASSWORD: &str = "senha nova 22";

async fn request(app: &Router, login: &str) -> StatusCode {
    let body = json!({ "login": login });
    let (status, body, _) = send_json(app, Method::POST, "/auth/password/reset", None, body).await;
    assert_eq!(body, Value::Null, "a resposta não diz nada");
    status
}

async fn reset(app: &Router, token: &str, password: &str) -> (StatusCode, Value, Option<String>) {
    let body = json!({ "token": token, "password": password });
    send_json(
        app,
        Method::POST,
        "/auth/password/reset/confirm",
        None,
        body,
    )
    .await
}

async fn login(app: &Router, username: &str, password: &str) -> StatusCode {
    let body = json!({ "username": username, "password": password });
    send_json(app, Method::POST, "/auth/login", None, body)
        .await
        .0
}

#[sqlx::test(migrations = "../../migrations")]
async fn link_sets_a_new_password_and_ends_every_session(db: PgPool) {
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    assert_eq!(login(&app, "rapha", PASSWORD).await, StatusCode::OK);
    assert_eq!(session_count(&app, &me).await, 2);

    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(request(&app, " RAPHA ").await, StatusCode::ACCEPTED);
    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "rapha@example.com");
    assert!(email.text.contains("/pt/login/reset#"), "{}", email.text);
    assert!(email.text.contains("Rapha"), "lembra o username");
    let token = link_token(&email);

    // Senha fora do formato não gasta o link.
    let (status, body, _) = reset(&app, &token, "curta").await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("auth.password_invalid")
        )
    );

    let (status, body, session) = reset(&app, &token, NEW_PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], me.id);
    let fresh = User {
        id: me.id,
        cookie: session.unwrap(),
    };
    assert_eq!(get(&app, "/me", &me).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(session_count(&app, &fresh).await, 1);
    assert_eq!(
        login(&app, "rapha", PASSWORD).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(login(&app, "rapha", NEW_PASSWORD).await, StatusCode::OK);

    let (status, body, _) = reset(&app, &token, "outra senha 3").await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn email_finds_the_account_in_any_spelling(db: PgPool) {
    let app = app(db.clone());
    // Conta com o email gravado direto: o cadastro não deixa espera nele.
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    set_email(&db, me.id, "joaosilva@gmail.com").await;

    assert_eq!(
        request(&app, "Joao.Silva+x@googlemail.com").await,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "joaosilva@gmail.com");
}

#[sqlx::test(migrations = "../../migrations")]
async fn same_answer_when_no_link_can_go(db: PgPool) {
    let app = app(db.clone());
    // Conta só com Discord, mas com email: não há senha para recuperar.
    let discord = user(&db, "Discord").await;
    set_email(&db, discord.id, "discord@outlook.com").await;

    for login in [
        "ninguem",
        "x",
        "ninguem@outlook.com",
        "discord@outlook.com",
        "nao é email@",
    ] {
        assert_eq!(request(&app, login).await, StatusCode::ACCEPTED, "{login}");
    }
    mailbox(&db).assert_empty().await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_request_replaces_the_link_and_limits_stay_quiet(db: PgPool) {
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    // O email do cadastro acabou de sair: o pedido fica sem envio, calado.
    assert_eq!(request(&app, "rapha").await, StatusCode::ACCEPTED);
    mailbox(&db).assert_empty().await;

    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(request(&app, "rapha").await, StatusCode::ACCEPTED);
    let first = mailbox(&db).next().await;
    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(
        request(&app, "rapha@example.com").await,
        StatusCode::ACCEPTED
    );
    let second = mailbox(&db).next().await;

    let (status, body, _) = reset(&app, &link_token(&first), NEW_PASSWORD).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
    assert_eq!(
        reset(&app, &link_token(&second), NEW_PASSWORD).await.0,
        StatusCode::OK
    );
}
