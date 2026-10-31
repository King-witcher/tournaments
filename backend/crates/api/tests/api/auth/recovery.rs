//! Recuperação de senha pelo email da conta.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};

use super::{send_json, session_count, set_email, signup, skip_cooldown};
use crate::common::{User, app, err, get, link_token, mailbox, patch, put, user};

const PASSWORD: &str = "senha forte 1";
const NEW_PASSWORD: &str = "senha nova 22";

async fn request(app: &Router, email: &str) -> StatusCode {
    let body = json!({ "email": email });
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

async fn login(app: &Router, email: &str, password: &str) -> StatusCode {
    let body = json!({ "email": email, "password": password });
    send_json(app, Method::POST, "/auth/login", None, body)
        .await
        .0
}

#[tokio::test]
async fn link_sets_a_new_password_and_ends_every_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    assert_eq!(
        login(&app, "rapha@example.com", PASSWORD).await,
        StatusCode::OK
    );
    assert_eq!(session_count(&app, &me).await, 2);

    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(
        request(&app, " Rapha@Example.COM ").await,
        StatusCode::ACCEPTED
    );
    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "rapha@example.com");
    assert!(email.text.contains("/pt/login/reset#"), "{}", email.text);
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
        login(&app, "rapha@example.com", PASSWORD).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, "rapha@example.com", NEW_PASSWORD).await,
        StatusCode::OK
    );

    let (status, body, _) = reset(&app, &token, "outra senha 3").await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
}

#[tokio::test]
async fn email_finds_the_account_in_any_spelling() {
    let (db, _lease) = crate::db::test_db().await;
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

#[tokio::test]
async fn same_answer_when_no_link_can_go() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    // Conta só com Discord, mas com email: não há senha para recuperar.
    let discord = user(&db, "Discord").await;
    set_email(&db, discord.id, "discord@outlook.com").await;

    for email in [
        "ninguem@outlook.com",
        "discord@outlook.com",
        "temp@mailinator.com",
    ] {
        assert_eq!(request(&app, email).await, StatusCode::ACCEPTED, "{email}");
    }
    mailbox(&db).assert_empty().await;

    // Fora do formato não há conta, e dizer isso não revela nada.
    let body = json!({ "email": "Rapha" });
    let (status, body, _) = send_json(&app, Method::POST, "/auth/password/reset", None, body).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, err("email.invalid"))
    );
}

async fn confirm_email(app: &Router, token: &str) -> StatusCode {
    let body = json!({ "token": token });
    send_json(app, Method::POST, "/auth/email/confirm", None, body)
        .await
        .0
}

#[tokio::test]
async fn a_new_email_ends_the_reset_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(
        request(&app, "rapha@example.com").await,
        StatusCode::ACCEPTED
    );
    let old_link = link_token(&mailbox(&db).next().await);

    // O link foi para o email de antes, que pode ser de quem perdeu o acesso.
    let change = json!({ "email": "novo@outlook.com", "current_password": PASSWORD });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
    let pending = link_token(&mailbox(&db).next().await);
    assert_eq!(confirm_email(&app, &pending).await, StatusCode::NO_CONTENT);
    let (status, body, _) = reset(&app, &old_link, NEW_PASSWORD).await;
    assert_eq!((status, body), (StatusCode::GONE, err("auth.link_expired")));
}

#[tokio::test]
async fn a_new_password_ends_the_pending_links() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let change_email = |email: &str, current: &str| {
        put(
            &app,
            "/auth/email",
            &me,
            json!({ "email": email, "current_password": current }),
        )
    };

    // Trocada na conta: a troca de email e a recuperação pedidas antes caem.
    assert_eq!(
        change_email("novo@outlook.com", PASSWORD).await.0,
        StatusCode::ACCEPTED
    );
    let pending = link_token(&mailbox(&db).next().await);
    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(
        request(&app, "rapha@example.com").await,
        StatusCode::ACCEPTED
    );
    let reset_link = link_token(&mailbox(&db).next().await);
    let body = json!({ "current_password": PASSWORD, "password": NEW_PASSWORD });
    let (status, body) = patch(&app, "/auth/password", &me, body).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    mailbox(&db).next().await; // o aviso da senha trocada
    assert_eq!(confirm_email(&app, &pending).await, StatusCode::GONE);
    let (status, _, _) = reset(&app, &reset_link, "outra senha 3").await;
    assert_eq!(status, StatusCode::GONE);

    // Trocada pelo link: a troca de email pedida antes cai.
    assert_eq!(
        change_email("outro@outlook.com", NEW_PASSWORD).await.0,
        StatusCode::ACCEPTED
    );
    let pending = link_token(&mailbox(&db).next().await);
    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(
        request(&app, "rapha@example.com").await,
        StatusCode::ACCEPTED
    );
    let reset_link = link_token(&mailbox(&db).next().await);
    let (status, _, session) = reset(&app, &reset_link, "outra senha 3").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(confirm_email(&app, &pending).await, StatusCode::GONE);
    let fresh = User {
        id: me.id,
        cookie: session.unwrap(),
    };
    let (_, logins) = get(&app, "/auth/logins", &fresh).await;
    assert_eq!(logins["email"], "rapha@example.com");
}

#[tokio::test]
async fn a_new_request_replaces_the_link_and_limits_stay_quiet() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    // O email do cadastro acabou de sair: o pedido fica sem envio, calado.
    assert_eq!(
        request(&app, "rapha@example.com").await,
        StatusCode::ACCEPTED
    );
    mailbox(&db).assert_empty().await;

    skip_cooldown(&db, "rapha@example.com").await;
    assert_eq!(
        request(&app, "rapha@example.com").await,
        StatusCode::ACCEPTED
    );
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

#[tokio::test]
async fn failed_send_stays_silent_and_leaves_no_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, "rapha@example.com").await;

    mailbox(&db).set_failing(true);
    // A resposta não muda: a recuperação não diz quem tem conta.
    assert_eq!(
        request(&app, "rapha@example.com").await,
        StatusCode::ACCEPTED
    );
    let redis = crate::common::redis(&db);
    let link = redis.key(format_args!("password_reset_user:{}", me.id));
    let cooldown = redis.key(format_args!("email_cooldown:rapha@example.com"));
    // O envio é em segundo plano: espera ele falhar e desfazer o link.
    let mut gone = false;
    for _ in 0..50 {
        let (link, cooldown): (bool, bool) = redis
            .query_pipe(redis::pipe().exists(&link).exists(&cooldown))
            .await
            .unwrap();
        if !link && !cooldown {
            gone = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(gone, "o link que não chegou e a espera de 60 s saem");
}
