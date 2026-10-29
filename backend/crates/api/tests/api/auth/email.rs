//! Pôr, trocar ou tirar o email da conta logada; pôr e trocar, confirmados
//! pelo link.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{add_discord, send_json, set_email, signup};
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

#[sqlx::test(migrations = "../../migrations")]
async fn new_email_counts_only_after_the_link(db: PgPool) {
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

#[sqlx::test(migrations = "../../migrations")]
async fn password_accounts_confirm_with_the_current_password(db: PgPool) {
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

    let other = user(&db, "Outra").await;
    set_email(&db, other.id, "outra@outlook.com").await;
    for (address, status, code) in [
        (
            "Rapha+x@example.com",
            StatusCode::CONFLICT,
            "auth.email_unchanged",
        ),
        (
            "outra@outlook.com",
            StatusCode::CONFLICT,
            "auth.email_taken",
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

#[sqlx::test(migrations = "../../migrations")]
async fn discord_account_sets_email_without_password(db: PgPool) {
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

#[sqlx::test(migrations = "../../migrations")]
async fn only_accounts_without_password_remove_the_email(db: PgPool) {
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
        json!({ "discord": true, "google": false, "username": null, "email": null })
    );
    assert_eq!(
        confirm(&app, &link_token(&pending)).await,
        (StatusCode::GONE, err("auth.link_expired"))
    );
    assert_eq!(email_of(&app, &me).await, Value::Null);

    let unauthenticated = call(&app, Method::DELETE, "/auth/email", None, None).await;
    assert_eq!(unauthenticated.0, StatusCode::UNAUTHORIZED);
}
