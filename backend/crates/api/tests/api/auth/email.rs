//! Pôr, trocar ou tirar o email da conta logada; pôr e trocar, confirmados
//! pelo link.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};

use super::{add_discord, send_json, set_email, signup, skip_cooldown};
use crate::common::{User, app, call, delete, err, get, link_token, mailbox, put, user};

const PASSWORD: &str = "senha forte 1";

async fn confirm(app: &Router, token: &str) -> (StatusCode, Value) {
    let body = json!({ "token": token });
    let (status, body, _) = send_json(app, Method::POST, "/auth/email/confirm", None, body).await;
    (status, body)
}

async fn email_of(app: &Router, who: &User) -> Value {
    get(app, "/auth/logins", who).await.1["email"].clone()
}

#[tokio::test]
async fn new_email_counts_only_after_the_link() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let change = json!({ "email": "Rapha.Q@Gmail.com", "current_password": PASSWORD });
    let (status, body) = put(&app, "/auth/email", &me, change).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert_eq!(body, json!({ "email": "raphaq@gmail.com" }));
    assert_eq!(
        email_of(&app, &me).await,
        "rapha@example.com",
        "até o link, fica o de antes"
    );

    let email = mailbox(&db).next().await;
    assert_eq!(email.to, "raphaq@gmail.com");
    assert!(email.text.contains("/pt/email/confirm#"), "{}", email.text);
    // O link não pede login: pode abrir em outro aparelho.
    assert_eq!(
        confirm(&app, &link_token(&email)).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(email_of(&app, &me).await, "raphaq@gmail.com");
    assert_eq!(
        confirm(&app, &link_token(&email)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
}

#[tokio::test]
async fn password_accounts_confirm_with_the_current_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    for change in [
        json!({ "email": "novo@outlook.com" }),
        json!({ "email": "novo@outlook.com", "current_password": "errada!!" }),
    ] {
        let (status, body) = put(&app, "/auth/email", &me, change).await;
        assert_eq!(
            (status, body),
            (StatusCode::FORBIDDEN, err("auth.wrong_password"))
        );
    }

    for (address, status, code) in [
        (
            "Rapha+x@example.com",
            StatusCode::CONFLICT,
            "auth.email_unchanged",
        ),
        (
            "x@mailinator.com",
            StatusCode::UNPROCESSABLE_ENTITY,
            "email.disposable",
        ),
    ] {
        let change = json!({ "email": address, "current_password": PASSWORD });
        let (got, body) = put(&app, "/auth/email", &me, change).await;
        assert_eq!((got, body), (status, err(code)), "{address}");
    }
    mailbox(&db).assert_empty().await;
}

#[tokio::test]
async fn discord_account_sets_email_without_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    assert_eq!(email_of(&app, &me).await, Value::Null);

    let (status, _) = put(
        &app,
        "/auth/email",
        &me,
        json!({ "email": "q@outlook.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    confirm(&app, &link_token(&mailbox(&db).next().await)).await;
    assert_eq!(email_of(&app, &me).await, "q@outlook.com");

    // Outra conta que confirmou o mesmo email antes leva a vez.
    let other = user(&db, "Outra").await;
    let (status, _) = put(
        &app,
        "/auth/email",
        &other,
        json!({ "email": "o@outlook.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let pending = mailbox(&db).next().await;
    set_email(&db, me.id, "o@outlook.com").await;
    assert_eq!(
        confirm(&app, &link_token(&pending)).await,
        (StatusCode::CONFLICT, err("auth.email_taken"))
    );

    let (status, _) = put(
        &app,
        "/auth/email",
        &me,
        json!({ "email": "z@outlook.com" }),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let unauthenticated = send_json(
        &app,
        Method::PUT,
        "/auth/email",
        None,
        json!({ "email": "z@outlook.com" }),
    )
    .await;
    assert_eq!(unauthenticated.0, StatusCode::UNAUTHORIZED);
}

/// A conta sem senha troca o email só com a sessão: a resposta não pode dizer
/// quem tem conta.
#[tokio::test]
async fn email_of_another_account_answers_the_same() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let other = signup(&app, &db, "Rapha", PASSWORD).await;
    skip_cooldown(&db, "rapha@example.com").await;
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;

    let change = json!({ "email": "Rapha@Example.com" });
    let (status, body) = put(&app, "/auth/email", &me, change.clone()).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::ACCEPTED,
            json!({ "email": "rapha@example.com" })
        )
    );
    // O dono fica sabendo, e não sai link.
    let notice = mailbox(&db).next().await;
    assert_eq!(notice.to, "rapha@example.com");
    assert!(notice.text.contains("já é da sua conta"), "{}", notice.text);
    assert!(
        !notice.text.contains("/email/confirm"),
        "sem link: {}",
        notice.text
    );

    // Pedir de novo esbarra na mesma espera de um email livre.
    let (status, body) = put(&app, "/auth/email", &me, change).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "email.too_soon");
    assert_eq!(email_of(&app, &me).await, Value::Null);
    assert_eq!(email_of(&app, &other).await, "rapha@example.com");
}

#[tokio::test]
async fn only_accounts_without_password_remove_the_email() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    // Com senha, o email é o que a recupera.
    let rapha = signup(&app, &db, "Rapha", PASSWORD).await;
    assert_eq!(
        delete(&app, "/auth/email", &rapha).await,
        (StatusCode::CONFLICT, err("auth.password_needs_email"))
    );
    assert_eq!(email_of(&app, &rapha).await, "rapha@example.com");

    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    assert_eq!(
        delete(&app, "/auth/email", &me).await,
        (StatusCode::CONFLICT, err("auth.no_email"))
    );

    // Com um link de troca esperando, tirar o email derruba o link.
    set_email(&db, me.id, "q@outlook.com").await;
    let change = json!({ "email": "novo@outlook.com" });
    assert_eq!(
        put(&app, "/auth/email", &me, change).await.0,
        StatusCode::ACCEPTED
    );
    let pending = mailbox(&db).next().await;
    let (status, body) = delete(&app, "/auth/email", &me).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "password": false, "email": null })
    );
    assert_eq!(
        confirm(&app, &link_token(&pending)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
    assert_eq!(email_of(&app, &me).await, Value::Null);

    let unauthenticated = call(&app, Method::DELETE, "/auth/email", None, None).await;
    assert_eq!(unauthenticated.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn failed_send_leaves_no_link_and_allows_a_retry_right_away() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let change = json!({ "email": "novo@outlook.com", "current_password": PASSWORD });

    mailbox(&db).set_failing(true);
    assert_eq!(
        put(&app, "/auth/email", &me, change.clone()).await,
        (StatusCode::BAD_GATEWAY, err("email.send_failed"))
    );
    let redis = crate::common::redis(&db);
    let link = redis.key(format_args!("email_change_user:{}", me.id));
    let kept: bool = redis.query(&redis::Cmd::exists(link)).await.unwrap();
    assert!(!kept, "o link que não chegou não fica valendo");

    mailbox(&db).set_failing(false);
    let (status, body) = put(&app, "/auth/email", &me, change).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let email = mailbox(&db).next().await;
    assert_eq!(
        confirm(&app, &link_token(&email)).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(email_of(&app, &me).await, "novo@outlook.com");
}
