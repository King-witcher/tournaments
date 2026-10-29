//! Login com username e senha, limite de tentativas, e criar, trocar e
//! remover a senha da conta logada. O cadastro fica em `signup.rs`.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use redis::Cmd;
use serde_json::{Value, json};

use super::{add_discord, send_json, session_count, set_email, signup};
use crate::common::{User, app, call, err, get, patch, post, redis, user};

const PASSWORD: &str = "senha forte 1";

async fn login(
    app: &Router,
    username: &str,
    password: &str,
) -> (StatusCode, Value, Option<String>) {
    send_json(
        app,
        Method::POST,
        "/auth/login",
        None,
        json!({ "username": username, "password": password }),
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
async fn login_ignores_username_case() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let rapha = signup(&app, &db, "Rapha", PASSWORD).await;

    let (status, body, session) = login(&app, "rAPHA", PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], rapha.id);
    assert_ne!(session.unwrap(), rapha.cookie);
    assert_eq!(session_count(&app, &rapha).await, 2);

    for (username, password) in [
        ("Rapha", "senha errada"),
        ("ninguem", PASSWORD),
        ("x", PASSWORD),
        ("Rapha", "curta"),
    ] {
        let (status, body, session) = login(&app, username, password).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{username}");
        assert_eq!(body, err("auth.invalid_credentials"));
        assert!(session.is_none());
    }
}

#[tokio::test]
async fn wrong_passwords_block_the_username() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    signup(&app, &db, "Rapha", PASSWORD).await;
    for _ in 0..5 {
        assert_eq!(
            login(&app, "rapha", "senha errada").await.0,
            StatusCode::UNAUTHORIZED
        );
    }

    // Bloqueado, nem a senha certa entra.
    let (status, body, session) = login(&app, "Rapha", PASSWORD).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        body,
        json!({ "error": "auth.too_many_attempts", "params": { "minutes": 15 } })
    );
    assert!(session.is_none());

    // Username que não existe conta igual, para não revelar quem existe.
    for _ in 0..5 {
        assert_eq!(
            login(&app, "ninguem", PASSWORD).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        login(&app, "ninguem", PASSWORD).await.0,
        StatusCode::TOO_MANY_REQUESTS
    );

    // Com os erros fora da janela, entra, e o acerto zera a contagem.
    let redis = redis(&db);
    let failures = redis.key("password_failures:rapha");
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
    assert_eq!(login(&app, "Rapha", PASSWORD).await.0, StatusCode::OK);
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
            tokio::spawn(async move { login(&app, "rapha", "senha errada").await.0 })
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
async fn discord_account_adds_username_and_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone());
    let me = user(&db, "Quaker").await;
    add_discord(&db, me.id).await;
    let (_, body) = get(&app, "/auth/logins", &me).await;
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "username": null, "email": null })
    );

    // Sem email, a senha esquecida não teria volta.
    let creds = json!({ "username": "quaker", "password": PASSWORD });
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
        json!({ "discord": true, "google": false, "username": "quaker", "email": "quaker@example.com" })
    );

    let (status, body) = post(
        &app,
        "/auth/password",
        &me,
        json!({ "username": "outro", "password": PASSWORD }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, err("auth.password_exists"));

    let other = user(&db, "Outro").await;
    set_email(&db, other.id, "outro@example.com").await;
    let (status, body) = post(
        &app,
        "/auth/password",
        &other,
        json!({ "username": "QUAKER", "password": PASSWORD }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, err("auth.username_taken"));

    // Username reservado por um cadastro que espera o link também está em uso.
    let signup = json!({ "username": "Reservado", "password": PASSWORD, "email": "r@example.com" });
    let (status, body, _) = send_json(&app, Method::POST, "/auth/signup", None, signup).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let (status, body) = post(
        &app,
        "/auth/password",
        &other,
        json!({ "username": "reservado", "password": PASSWORD }),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.username_taken"))
    );

    // Entra na mesma conta, que mantém o nome exibido.
    let (status, body, _) = login(&app, "Quaker", PASSWORD).await;
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
        cookie: login(&app, "rapha", PASSWORD).await.2.unwrap(),
    };
    signup(&app, &db, "Outro", PASSWORD).await;

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
        (
            json!({ "current_password": PASSWORD, "username": "outro" }),
            StatusCode::CONFLICT,
            "auth.username_taken",
        ),
    ] {
        let (got, body) = patch(&app, "/auth/password", &me, change).await;
        assert_eq!((got, body), (status, err(code)));
    }

    let (status, body) = patch(
        &app,
        "/auth/password",
        &me,
        json!({ "current_password": PASSWORD, "username": "Rapha2", "password": "nova senha 2" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": false, "google": false, "username": "Rapha2", "email": "rapha@example.com" })
    );
    assert_eq!(get(&app, "/me", &me).await.0, StatusCode::OK);
    assert_eq!(
        get(&app, "/me", &other_session).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, "Rapha2", PASSWORD).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, "rapha2", "nova senha 2").await.0,
        StatusCode::OK
    );

    // Só o username: senha e sessões continuam.
    let (status, body) = patch(
        &app,
        "/auth/password",
        &me,
        json!({ "current_password": "nova senha 2", "username": "rapha2" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], "rapha2");
    assert_eq!(session_count(&app, &me).await, 2);
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
    // O email fica: recupera o acesso se a senha voltar.
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "username": null, "email": "rapha@example.com" })
    );
    assert_eq!(
        login(&app, "Rapha", PASSWORD).await.0,
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
        json!({ "current_password": PASSWORD, "username": "volta" }),
    )
    .await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.no_password"))
    );
}
