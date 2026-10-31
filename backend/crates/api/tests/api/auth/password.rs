//! Login com email e senha, limite de tentativas, e criar, trocar e remover a
//! senha da conta logada. O cadastro fica em `signup.rs`.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use redis::Cmd;
use serde_json::{Value, json};

use super::{add_discord, send_json, session_count, set_email, signup};
use crate::common::{User, app, call, err, get, patch, post, redis, user};

const PASSWORD: &str = "senha forte 1";

async fn login(app: &Router, email: &str, password: &str) -> (StatusCode, Value, Option<String>) {
    send_json(
        app,
        Method::POST,
        "/auth/login",
        None,
        json!({ "email": email, "password": password }),
    )
    .await
}

async fn remove(app: &Router, uri: &str, who: &User, current: &str) -> (StatusCode, Value) {
    call(
        app,
        Method::DELETE,
        uri,
        Some(who),
        Some(json!({ "current_password": current })),
    )
    .await
}

#[tokio::test]
async fn login_takes_the_email_in_any_form() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let rapha = signup(&app, &db, "Rapha", PASSWORD).await;

    // Escrito de outro jeito, dá na forma canônica.
    let (status, body, session) = login(&app, " RAPHA+torneio@Example.com ", PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], rapha.id);
    assert_ne!(session.unwrap(), rapha.cookie);
    assert_eq!(session_count(&app, &rapha).await, 2);

    for (email, password) in [
        ("rapha@example.com", "senha errada"),
        ("ninguem@example.com", PASSWORD),
        ("rapha@example.com", "curta"),
    ] {
        let (status, body, session) = login(&app, email, password).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{email}");
        assert_eq!(body, err("auth.invalid_credentials"));
        assert!(session.is_none());
    }

    // Um nome no lugar do email (o username de antes) não é login.
    let (status, body, _) = login(&app, "Rapha", PASSWORD).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, err("email.invalid"))
    );
}

#[tokio::test]
async fn wrong_passwords_block_the_email() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    for _ in 0..5 {
        assert_eq!(
            login(&app, "rapha@example.com", "senha errada").await.0,
            StatusCode::UNAUTHORIZED
        );
    }

    // Bloqueado, nem a senha certa entra, nem com o email escrito de outro jeito.
    let (status, body, session) = login(&app, "Rapha+x@example.com", PASSWORD).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        body,
        json!({ "error": "auth.too_many_attempts", "params": { "minutes": 15 } })
    );
    assert!(session.is_none());

    // Email sem conta conta igual, para não revelar quem tem conta.
    for _ in 0..5 {
        assert_eq!(
            login(&app, "ninguem@example.com", PASSWORD).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        login(&app, "ninguem@example.com", PASSWORD).await.0,
        StatusCode::TOO_MANY_REQUESTS
    );

    // Com os erros fora da janela, entra, e o acerto zera a contagem.
    let redis = redis(&db);
    let failures = redis.key("password_failures:rapha@example.com");
    let recent: Vec<(String, f64)> = redis
        .query(&Cmd::zrange_withscores(&failures, 0, -1))
        .await
        .unwrap();
    assert_eq!(recent.len(), 5);
    for (failure, at) in recent {
        let fifteen_minutes = 15.0 * 60_000.0;
        let _: i64 = redis
            .query(&Cmd::zadd(&failures, failure, at - fifteen_minutes))
            .await
            .unwrap();
    }
    assert_eq!(
        login(&app, "rapha@example.com", PASSWORD).await.0,
        StatusCode::OK
    );
    let left: bool = redis.query(&Cmd::exists(&failures)).await.unwrap();
    assert!(!left);
}

/// A tentativa conta antes da conferência: pedidos ao mesmo tempo não
/// conferem mais senhas que o limite.
#[tokio::test]
async fn simultaneous_attempts_stay_within_the_limit() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    let attempts: Vec<_> = (0..12)
        .map(|_| {
            let app = app.clone();
            tokio::spawn(async move { login(&app, "rapha@example.com", "senha errada").await.0 })
        })
        .collect();
    let mut checked = 0;
    for attempt in attempts {
        match attempt.await.unwrap() {
            StatusCode::UNAUTHORIZED => checked += 1,
            status => assert_eq!(status, StatusCode::TOO_MANY_REQUESTS),
        }
    }
    assert_eq!(checked, 5);
}

#[tokio::test]
async fn discord_account_adds_a_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    let (_, body) = get(&app, "/auth/logins", &me).await;
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "password": false, "email": null })
    );

    // Sem email, não há login nem volta para a senha esquecida.
    let creds = json!({ "password": PASSWORD });
    let (status, body) = post(&app, "/auth/password", &me, creds.clone()).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.email_required"))
    );
    set_email(&db, me.id, "quaker@example.com").await;
    let (status, body) = post(&app, "/auth/password", &me, creds.clone()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "password": true, "email": "quaker@example.com" })
    );

    let (status, body) = post(&app, "/auth/password", &me, creds.clone()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, err("auth.password_exists"));

    // Entra na mesma conta pelo email, e ela mantém o nome exibido.
    let (status, body, _) = login(&app, "quaker@example.com", PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], me.id);
    assert_eq!(body["display_name"], "Quaker");

    let (status, _) = call(&app, Method::POST, "/auth/password", None, Some(creds)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn change_password_needs_current_and_ends_other_sessions() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let other_session = User {
        id: me.id,
        cookie: login(&app, "rapha@example.com", PASSWORD).await.2.unwrap(),
    };

    for (change, status, code) in [
        (
            json!({ "current_password": "errada!!", "password": "nova senha 2" }),
            StatusCode::FORBIDDEN,
            "auth.wrong_password",
        ),
        (
            json!({ "current_password": PASSWORD, "password": "curta" }),
            StatusCode::UNPROCESSABLE_ENTITY,
            "auth.password_invalid",
        ),
    ] {
        let (got, body) = patch(&app, "/auth/password", &me, change).await;
        assert_eq!((got, body), (status, err(code)));
    }

    let (status, body) = patch(
        &app,
        "/auth/password",
        &me,
        json!({ "current_password": PASSWORD, "password": "nova senha 2" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": false, "google": false, "password": true, "email": "rapha@example.com" })
    );
    assert_eq!(get(&app, "/me", &me).await.0, StatusCode::OK);
    assert_eq!(
        get(&app, "/me", &other_session).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, "rapha@example.com", PASSWORD).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, "rapha@example.com", "nova senha 2").await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn password_is_removed_only_with_discord_left() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let (status, body) = remove(&app, "/auth/password", &me, PASSWORD).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.last_login"))
    );

    add_discord(&db, me.id).await;
    let (status, body) = remove(&app, "/auth/password", &me, "errada!!").await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("auth.wrong_password"))
    );
    let (status, body) = remove(&app, "/auth/password", &me, PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // O email fica: é o login se a senha voltar.
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "password": false, "email": "rapha@example.com" })
    );
    assert_eq!(
        login(&app, "rapha@example.com", PASSWORD).await.0,
        StatusCode::UNAUTHORIZED
    );

    let (status, body) = remove(&app, "/auth/password", &me, PASSWORD).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.no_password"))
    );
    let (status, body) = patch(
        &app,
        "/auth/password",
        &me,
        json!({ "current_password": PASSWORD, "password": "nova senha 3" }),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.no_password"))
    );
}
