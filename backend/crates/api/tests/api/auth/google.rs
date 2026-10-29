//! Entrar com Google, ligar e desligar, e a conta que fica com mais de um
//! provedor.

use axum::http::{Method, StatusCode};
use serde_json::json;

use super::{
    body_text, discord_user, do_login, get, google_login, google_user, location, oauth_round_trip,
    providers_app, send, set_cookie, set_email, signup, start_oauth,
};
use crate::common::{self, User, WEB, call, err, mailbox};

const PASSWORD: &str = "senha forte 1";
const LINK: &str = "/auth/google/link?return_to=/pt/account/logins";

#[tokio::test]
async fn login_without_config_is_503() {
    let (db, _lease) = crate::db::test_db().await;
    let app = providers_app(db, None, None, false);
    let res = send(&app, get("/auth/google/login", None)).await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        body_text(res).await,
        r#"{"error":"auth.google_not_configured"}"#
    );
}

#[tokio::test]
async fn login_redirects_to_google_asking_only_id_and_name() {
    let (db, _lease) = crate::db::test_db().await;
    let app = providers_app(db, None, None, true);
    let res = send(&app, get("/auth/google/login?return_to=/pt/algo", None)).await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    let cookie = set_cookie(&res, "oauth_state").unwrap();
    assert!(cookie.contains(".google.login."), "{cookie}");

    let url = location(&res);
    assert!(
        url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"),
        "{url}"
    );
    assert!(url.contains("response_type=code"));
    assert!(url.contains("client_id=google-client"));
    // Nada de email: o da conta daqui só entra confirmado pelo link.
    assert!(url.contains("scope=openid+profile&"), "{url}");
    assert!(!url.contains("email"), "{url}");
    assert!(url.contains("prompt=select_account"));
    assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A4000%2Fauth%2Fgoogle%2Fcallback"));
}

#[tokio::test]
async fn callback_creates_account_with_the_google_name() {
    let (db, _lease) = crate::db::test_db().await;
    let app = providers_app(db.clone(), None, Some(google_user("Ana Souza")), true);
    let me = google_login(&app).await;

    let user = sqlx::query!("select display_name, avatar_url, email from users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(user.display_name, "Ana Souza");
    assert_eq!(user.avatar_url, None, "a foto do Google não vem");
    assert_eq!(user.email, None);
    let ident = sqlx::query!("select provider, provider_user_id from user_identities")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        (ident.provider.as_str(), ident.provider_user_id.as_str()),
        ("google", "g-1")
    );
    let (_, body) = common::get(&app, "/auth/logins", &me).await;
    assert_eq!(
        body,
        json!({ "discord": false, "google": true, "username": null, "email": null })
    );

    // Entrar de novo é a mesma conta, e o nome trocado aqui fica.
    sqlx::query!(
        "update users set display_name = 'Aninha' where id = $1",
        me.id
    )
    .execute(&db)
    .await
    .unwrap();
    let again = providers_app(db.clone(), None, Some(google_user("Ana S.")), true);
    let second = google_login(&again).await;
    assert_eq!(second.id, me.id);
    let name = sqlx::query_scalar!("select display_name from users where id = $1", me.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(name, "Aninha");
}

#[tokio::test]
async fn callback_errors_redirect_with_google_codes() {
    let (db, _lease) = crate::db::test_db().await;
    let failing = providers_app(db.clone(), None, None, true);
    let (session, res) = oauth_round_trip(&failing, "/auth/google/login", None).await;
    assert!(session.is_empty());
    assert_eq!(
        location(&res),
        format!("{WEB}/?auth_error=auth.google_failed")
    );

    let (cookie, _) = start_oauth(&failing, "/auth/google/login", None).await;
    let res = send(
        &failing,
        get("/auth/google/callback?error=access_denied", Some(&cookie)),
    )
    .await;
    assert_eq!(
        location(&res),
        format!("{WEB}/?auth_error=auth.google_denied")
    );

    // O state aberto para o Discord não serve no callback do Google.
    let app = providers_app(db.clone(), None, Some(google_user("Ana")), true);
    let (cookie, state) = start_oauth(&app, "/auth/discord/login", None).await;
    let res = send(
        &app,
        get(
            &format!("/auth/google/callback?code=abc&state={state}"),
            Some(&cookie),
        ),
    )
    .await;
    assert_eq!(
        location(&res),
        format!("{WEB}/?auth_error=auth.state_mismatch")
    );
    let users = sqlx::query_scalar!("select count(*) from users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(users, Some(0));
}

#[tokio::test]
async fn discord_account_links_google_and_drops_discord() {
    let (db, _lease) = crate::db::test_db().await;
    let app = providers_app(
        db.clone(),
        Some(discord_user(Some("av"))),
        Some(google_user("Ana")),
        true,
    );
    let (session, _) = do_login(&app, "/").await;
    let id = sqlx::query_scalar!("select id from users")
        .fetch_one(&db)
        .await
        .unwrap();
    let me = User {
        id,
        cookie: session,
    };
    set_email(&db, id, "quaker@example.com").await;

    let (session, res) = oauth_round_trip(&app, LINK, Some(&me.cookie)).await;
    assert!(session.is_empty(), "ligar não cria sessão");
    assert_eq!(location(&res), format!("{WEB}/pt/account/logins"));
    let notice = mailbox(&db).next().await;
    assert!(
        notice.subject.contains("conta Google"),
        "{}",
        notice.subject
    );
    assert!(
        notice.text.contains("Ana"),
        "o nome no Google: {}",
        notice.text
    );
    let (_, body) = common::get(&app, "/auth/logins", &me).await;
    assert_eq!(
        body,
        json!({ "discord": true, "google": true, "username": null, "email": "quaker@example.com" })
    );
    let avatar = sqlx::query_scalar!("select avatar_url from users where id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(
        avatar.is_some(),
        "ligar o Google não mexe no avatar do Discord"
    );
    assert_eq!(
        google_login(&app).await.id,
        id,
        "o Google entra na mesma conta"
    );

    // Já ligado: nem vai para o Google.
    let res = send(&app, get(LINK, Some(&me.cookie))).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
    assert_eq!(
        body_text(res).await,
        r#"{"error":"auth.google_already_linked"}"#
    );

    // Sem senha, desligar não pede nada: sobra o outro provedor.
    let unlink = |path: &'static str| call(&app, Method::DELETE, path, Some(&me), Some(json!({})));
    let (status, body) = unlink("/auth/discord").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": false, "google": true, "username": null, "email": "quaker@example.com" })
    );
    // O aviso diz por onde a conta continua entrando.
    let notice = mailbox(&db).next().await;
    assert!(notice.text.contains("só pelo Google"), "{}", notice.text);
    let avatar = sqlx::query_scalar!("select avatar_url from users where id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(avatar, None, "o avatar vinha do Discord");

    let (status, body) = unlink("/auth/google").await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.last_login"))
    );
    let (status, body) = unlink("/auth/discord").await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.discord_not_linked"))
    );
}

#[tokio::test]
async fn google_of_another_account_is_refused() {
    let (db, _lease) = crate::db::test_db().await;
    let app = providers_app(db.clone(), None, Some(google_user("Ana")), true);
    google_login(&app).await;
    let me = signup(&app, &db, "Rapha", PASSWORD).await;

    let (_, res) = oauth_round_trip(&app, LINK, Some(&me.cookie)).await;
    assert_eq!(
        location(&res),
        format!("{WEB}/pt/account/logins?auth_error=auth.google_taken")
    );
    let (_, body) = common::get(&app, "/auth/logins", &me).await;
    assert_eq!(body["google"], false);
}

#[tokio::test]
async fn password_account_with_google() {
    let (db, _lease) = crate::db::test_db().await;
    let app = providers_app(db.clone(), None, Some(google_user("Ana")), true);
    let me = signup(&app, &db, "Rapha", PASSWORD).await;
    oauth_round_trip(&app, LINK, Some(&me.cookie)).await;

    // Com senha, desligar pede a atual, e sem ela é como errada.
    let unlink =
        |body: serde_json::Value| call(&app, Method::DELETE, "/auth/google", Some(&me), Some(body));
    for body in [json!({}), json!({ "current_password": "errada!!" })] {
        let (status, res) = unlink(body).await;
        assert_eq!(
            (status, res),
            (StatusCode::FORBIDDEN, err("auth.wrong_password"))
        );
    }

    // O Google basta para tirar a senha.
    let (status, body) = call(
        &app,
        Method::DELETE,
        "/auth/password",
        Some(&me),
        Some(json!({ "current_password": PASSWORD })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "discord": false, "google": true, "username": null, "email": "rapha@example.com" })
    );
    let (status, body) = unlink(json!({})).await;
    assert_eq!(
        (status, body),
        (StatusCode::CONFLICT, err("auth.last_login"))
    );
}
