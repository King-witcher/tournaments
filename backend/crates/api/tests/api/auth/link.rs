//! Ligar o Discord a uma conta e desligar.

use axum::http::{Method, StatusCode};
use serde_json::json;

use super::{
    OTHER_VERIFIER, VERIFIER, app, authorize, discord_user, do_login, exchange, link_provider,
    send_json, set_email, signup,
};
use crate::common::{self, User, call, err, post};

const PASSWORD: &str = "senha forte 1";

#[tokio::test]
async fn password_account_links_discord() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(Some("av"))), true);
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let (status, body) = link_provider(&app, "discord", &me, Some(PASSWORD)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": true, "google": false, "password": true, "email": "rapha@example.com" })
    );
    let avatar = sqlx::query_scalar!("select avatar_url from users where id = $1", me.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        avatar.as_deref(),
        Some("https://cdn.discordapp.com/avatars/123/av.png")
    );

    // Agora o Discord entra na mesma conta.
    let (_, body) = do_login(&app).await;
    assert_eq!(body["id"], me.id);

    // Já ligado: nem vai para o Discord.
    let (status, body) = authorize(&app, "discord", Some(&me.cookie), None).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.discord_already_linked"))
    );
}

#[tokio::test]
async fn link_requires_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    let unauthenticated = (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"));
    let (status, body) = authorize(&app, "discord", None, None).await;
    assert_eq!((status, body), unauthenticated);

    // A sessão também precisa estar na troca do code.
    let (status, body, _) = exchange(&app, "discord", "link", None, VERIFIER).await;
    assert_eq!((status, body), unauthenticated);
}

#[tokio::test]
async fn discord_of_another_account_is_refused() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    do_login(&app).await;
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let (status, body) = link_provider(&app, "discord", &me, Some(PASSWORD)).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.discord_taken"))
    );
    let (_, body) = common::get(&app, "/auth/logins", &me).await;
    assert_eq!(body["discord"], false);
}

#[tokio::test]
async fn link_errors_link_nothing() {
    let (db, _lease) = crate::db::test_db().await;
    let failed = (StatusCode::BAD_GATEWAY, err("auth.discord_failed"));
    let failing = app(db.clone(), None, true);
    let me = signup(&failing, &db, "Rapha", PASSWORD).await;
    let (status, body) = link_provider(&failing, "discord", &me, Some(PASSWORD)).await;
    assert_eq!((status, body), failed);

    // O code de outra aba não liga, mesmo com a sessão: o verifier não é o
    // do fluxo aberto, e nem vai ao Discord.
    let app = app(db.clone(), Some(discord_user(None)), true);
    let (status, _) = authorize(&app, "discord", Some(&me.cookie), Some(PASSWORD)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body, _) =
        exchange(&app, "discord", "link", Some(&me.cookie), OTHER_VERIFIER).await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("auth.state_mismatch"))
    );
    let (_, body) = common::get(&app, "/auth/logins", &me).await;
    assert_eq!(body["discord"], false);
}

#[tokio::test]
async fn link_needs_the_password_and_a_flow_of_this_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    let wrong = (StatusCode::FORBIDDEN, err("auth.wrong_password"));
    let mismatch = (StatusCode::FORBIDDEN, err("auth.state_mismatch"));

    // Com senha na conta, a sessão sozinha não abre o fluxo de ligar.
    let (status, body) = authorize(&app, "discord", Some(&me.cookie), None).await;
    assert_eq!((status, body), wrong);
    let (status, body) = authorize(&app, "discord", Some(&me.cookie), Some("errada!!")).await;
    assert_eq!((status, body), wrong);
    // E sem o fluxo aberto, o code não liga.
    let (status, body, _) = exchange(&app, "discord", "link", Some(&me.cookie), VERIFIER).await;
    assert_eq!((status, body), mismatch);

    // O fluxo é da sessão que o abriu: outra sessão da conta não o usa.
    let (status, _) = authorize(&app, "discord", Some(&me.cookie), Some(PASSWORD)).await;
    assert_eq!(status, StatusCode::OK);
    let credentials = json!({ "email": "rapha@example.com", "password": PASSWORD });
    let (_, _, other) = send_json(&app, Method::POST, "/auth/login", None, credentials).await;
    let (status, body, _) =
        exchange(&app, "discord", "link", Some(&other.unwrap()), VERIFIER).await;
    assert_eq!((status, body), mismatch);
    let (status, body, _) = exchange(&app, "discord", "link", Some(&me.cookie), VERIFIER).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["discord"], true);

    // E vale uma vez.
    let (status, body, _) = exchange(&app, "discord", "link", Some(&me.cookie), VERIFIER).await;
    assert_eq!((status, body), mismatch);
}

#[tokio::test]
async fn unlink_needs_password() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(Some("av"))), true);
    let (session, _) = do_login(&app).await;
    let id = sqlx::query_scalar!("select id from users")
        .fetch_one(&db)
        .await
        .unwrap();
    let me = User {
        id,
        cookie: session,
    };
    let unlink = |current: &str| {
        call(
            &app,
            Method::DELETE,
            "/auth/discord",
            Some(&me),
            Some(json!({ "current_password": current })),
        )
    };

    let (status, body) = unlink(PASSWORD).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.last_login"))
    );

    // A senha vem com o email que a recupera.
    set_email(&db, id, "quaker@example.com").await;
    let (status, _) = post(&app, "/auth/password", &me, json!({ "password": PASSWORD })).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = unlink("errada!!").await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("auth.wrong_password"))
    );
    let (status, body) = unlink(PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": false, "google": false, "password": true, "email": "quaker@example.com" })
    );
    let avatar = sqlx::query_scalar!("select avatar_url from users where id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(avatar, None, "o avatar vinha do Discord");

    let (status, body) = unlink(PASSWORD).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.discord_not_linked"))
    );

    // Desligado, o mesmo Discord cria outra conta.
    let (_, body) = do_login(&app).await;
    assert_ne!(body["id"], id);
}
