//! Avisos por email do que muda no acesso à conta: senha, Discord e email.
//! Saem sem esperar a espera entre envios, que é dos links.

use api::{AppState, mail::Mail};
use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::json;
use sqlx::PgPool;

use super::{add_discord, app, discord_user, link_provider, send_json, set_email, signup};
use crate::common::{
    self, User, app_from, call, link_token, mailbox, patch, post, put, state, user,
};

const PASSWORD: &str = "senha forte 1";
const NEW_PASSWORD: &str = "senha nova 22";

async fn remove(app: &Router, uri: &str, who: &User) -> StatusCode {
    let body = json!({ "current_password": PASSWORD });
    call(app, Method::DELETE, uri, Some(who), Some(body))
        .await
        .0
}

/// O próximo email: para `to`, com `subject` no assunto, o botão para `path`.
async fn expect_notice(db: &PgPool, to: &str, subject: &str, path: &str) -> String {
    let email = mailbox(db).next().await;
    assert_eq!(email.to, to);
    assert!(email.subject.contains(subject), "{}", email.subject);
    assert!(
        email
            .text
            .contains(&format!("http://localhost:4001{path}\n")),
        "{}",
        email.text
    );
    assert!(email.html.contains("Não foi você?") || email.html.contains("¿No fuiste tú?"));
    email.text
}

#[tokio::test]
async fn password_changes() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    add_discord(&db, me.id).await;
    let to = "rapha@example.com";

    let change = json!({ "current_password": PASSWORD, "password": NEW_PASSWORD });
    assert_eq!(
        patch(&app, "/auth/password", &me, change).await.0,
        StatusCode::OK
    );
    expect_notice(&db, to, "Sua senha foi trocada", "/pt/login/forgot").await;

    let change = json!({ "current_password": NEW_PASSWORD, "password": PASSWORD });
    assert_eq!(
        patch(&app, "/auth/password", &me, change).await.0,
        StatusCode::OK
    );
    expect_notice(&db, to, "Sua senha foi trocada", "/pt/login/forgot").await;

    assert_eq!(remove(&app, "/auth/password", &me).await, StatusCode::OK);
    expect_notice(
        &db,
        to,
        "senha da sua conta foi removida",
        "/pt/account/sessions",
    )
    .await;
}

#[tokio::test]
async fn creating_a_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    set_email(&db, me.id, "q@outlook.com").await;

    let body = json!({ "password": PASSWORD });
    assert_eq!(
        post(&app, "/auth/password", &me, body).await.0,
        StatusCode::CREATED
    );
    let text = expect_notice(&db, "q@outlook.com", "agora tem senha", "/pt/login/forgot").await;
    assert!(text.contains("com este email e a senha"), "{text}");
}

#[tokio::test]
async fn discord_link_and_unlink() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let (status, body) = link_provider(&app, "discord", &me, Some(PASSWORD)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let text = expect_notice(
        &db,
        "rapha@example.com",
        "Um Discord foi vinculado",
        "/pt/account/logins",
    )
    .await;
    assert!(text.contains("quaker"), "o nome no Discord: {text}");

    assert_eq!(remove(&app, "/auth/discord", &me).await, StatusCode::OK);
    expect_notice(
        &db,
        "rapha@example.com",
        "Discord foi desvinculado",
        "/pt/login/forgot",
    )
    .await;
}

#[tokio::test]
async fn email_change_warns_the_old_address() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let change = json!({ "email": "novo@outlook.com", "current_password": PASSWORD });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
    let link = mailbox(&db).next().await;
    mailbox(&db).assert_empty().await;
    confirm_email(&app, &link_token(&link)).await;

    let text = expect_notice(
        &db,
        "rapha@example.com",
        "O email da sua conta foi trocado",
        "/pt/account",
    )
    .await;
    assert!(text.contains("novo@outlook.com"), "{text}");
    assert!(
        text.contains("não serve mais para entrar"),
        "com senha, o email era o login: {text}"
    );
    assert!(
        text.contains("aba Senha"),
        "com senha, a saída é trocá-la: {text}"
    );
}

#[tokio::test]
async fn first_email_and_email_removal() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;

    // O primeiro email não tem a quem avisar.
    let body = json!({ "email": "q@outlook.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, body).await.0,
        StatusCode::ACCEPTED
    );
    confirm_email(&app, &link_token(&mailbox(&db).next().await)).await;
    mailbox(&db).assert_empty().await;

    let (status, _) = call(&app, Method::DELETE, "/auth/email", Some(&me), None).await;
    assert_eq!(status, StatusCode::OK);
    expect_notice(
        &db,
        "q@outlook.com",
        "Este email foi removido",
        "/pt/account/sessions",
    )
    .await;
}

#[tokio::test]
async fn reset_by_link_right_after_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = common::app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    sqlx::query!("update users set locale = 'es'")
        .execute(&db)
        .await
        .unwrap();

    // O link acabou de sair para o mesmo endereço: o aviso não espera.
    super::skip_cooldown(&db, "rapha@example.com").await;
    let body = json!({ "email": "rapha@example.com" });
    send_json(&app, Method::POST, "/auth/password/reset", None, body).await;
    let link = mailbox(&db).next().await;
    let body = json!({ "token": link_token(&link), "password": NEW_PASSWORD });
    let (status, _, _) = send_json(
        &app,
        Method::POST,
        "/auth/password/reset/confirm",
        None,
        body,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    expect_notice(
        &db,
        "rapha@example.com",
        "Tu contraseña fue cambiada",
        "/es/login/forgot",
    )
    .await;
}

#[tokio::test]
async fn daily_limit_drops_notices() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app_from(AppState {
        mail: Mail::new(mailbox(&db), 1, true),
        ..state(db.clone())
    });
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let change = json!({ "current_password": PASSWORD, "password": NEW_PASSWORD });
    let (status, body) = patch(&app, "/auth/password", &me, change).await;
    assert_eq!(status, StatusCode::OK, "a troca vale sem o aviso: {body}");
    mailbox(&db).assert_empty().await;
}

async fn confirm_email(app: &Router, token: &str) {
    let body = json!({ "token": token });
    let (status, body, _) = send_json(app, Method::POST, "/auth/email/confirm", None, body).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}
