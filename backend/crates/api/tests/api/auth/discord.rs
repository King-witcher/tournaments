//! Entrar com Discord, `/me` e sair.

use api::shared::token::hash_token;
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use sqlx::PgPool;

use super::{app, body_text, discord_user, do_login, get, location, pair, send, set_cookie};
use crate::common::WEB;

#[sqlx::test(migrations = "../../migrations")]
async fn login_without_config_is_503(db: PgPool) {
    let app = app(db, None, false);
    let res = send(&app, get("/auth/discord/login", None)).await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        body_text(res).await,
        r#"{"error":"auth.discord_not_configured"}"#
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_redirects_to_discord(db: PgPool) {
    let app = app(db, None, true);
    let res = send(&app, get("/auth/discord/login?return_to=/pt/algo", None)).await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER);

    let cookie = set_cookie(&res, "oauth_state").unwrap();
    for attr in ["HttpOnly", "SameSite=Lax", "Path=/auth", "Max-Age=600"] {
        assert!(cookie.contains(attr), "{cookie} sem {attr}");
    }
    let state = pair(&cookie)
        .strip_prefix("oauth_state=")
        .unwrap()
        .split('.')
        .next()
        .unwrap()
        .to_owned();
    assert_eq!(state.len(), 43); // 32 bytes em base64url

    let url = location(&res);
    assert!(
        url.starts_with("https://discord.com/oauth2/authorize?"),
        "{url}"
    );
    assert!(url.contains("response_type=code"));
    assert!(url.contains("client_id=client-1"));
    assert!(url.contains("scope=identify"));
    assert!(url.contains(&format!("state={state}")));
    assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A4000%2Fauth%2Fdiscord%2Fcallback"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn callback_errors_redirect_with_code(db: PgPool) {
    let ok_app = app(db.clone(), Some(discord_user(None)), true);
    let res = send(&ok_app, get("/auth/discord/login", None)).await;
    let cookie = pair(&set_cookie(&res, "oauth_state").unwrap());

    let res = send(
        &ok_app,
        get("/auth/discord/callback?code=abc&state=outro", Some(&cookie)),
    )
    .await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        location(&res),
        format!("{WEB}/?auth_error=auth.state_mismatch")
    );

    let res = send(
        &ok_app,
        get("/auth/discord/callback?code=abc&state=x", None),
    )
    .await;
    assert_eq!(
        location(&res),
        format!("{WEB}/?auth_error=auth.state_mismatch")
    );

    let res = send(
        &ok_app,
        get("/auth/discord/callback?error=access_denied", Some(&cookie)),
    )
    .await;
    assert_eq!(
        location(&res),
        format!("{WEB}/?auth_error=auth.discord_denied")
    );

    let failing = app(db.clone(), None, true);
    let (session, res) = do_login(&failing, "/").await;
    assert!(session.is_empty());
    assert_eq!(
        location(&res),
        format!("{WEB}/?auth_error=auth.discord_failed")
    );
    let n = sqlx::query_scalar!("select count(*) from users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(n, Some(0));
}

#[sqlx::test(migrations = "../../migrations")]
async fn callback_creates_user_identity_and_session(db: PgPool) {
    let app = app(db.clone(), Some(discord_user(Some("hash1"))), true);
    let (session, res) = do_login(&app, "/pt/algo").await;

    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&res), format!("{WEB}/pt/algo"));

    let cookie = set_cookie(&res, "session").unwrap();
    for attr in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=2592000"] {
        assert!(cookie.contains(attr), "{cookie} sem {attr}");
    }
    let cleared = set_cookie(&res, "oauth_state").unwrap();
    assert!(cleared.starts_with("oauth_state=;") && cleared.contains("Max-Age=0"));

    let user = sqlx::query!("select id, display_name, avatar_url from users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(user.display_name, "Quaker Global");
    assert_eq!(
        user.avatar_url.as_deref(),
        Some("https://cdn.discordapp.com/avatars/123/hash1.png")
    );
    let ident = sqlx::query!("select user_id, provider_user_id from user_identities")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(ident.user_id, user.id);
    assert_eq!(ident.provider_user_id, "123");

    // O banco guarda só o hash, nunca o token.
    let token = session.strip_prefix("session=").unwrap();
    let stored = sqlx::query!("select id, user_id from sessions")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(stored.id, hash_token(token).to_vec());
    assert_ne!(stored.id, token.as_bytes());
    assert_eq!(stored.user_id, user.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn second_login_reuses_user_and_updates_avatar(db: PgPool) {
    let first = app(db.clone(), Some(discord_user(Some("old"))), true);
    do_login(&first, "/").await;
    let second = app(db.clone(), Some(discord_user(Some("new"))), true);
    do_login(&second, "/").await;

    let users = sqlx::query!("select avatar_url from users")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(
        users[0].avatar_url.as_deref(),
        Some("https://cdn.discordapp.com/avatars/123/new.png")
    );
    let idents = sqlx::query_scalar!("select count(*) from user_identities")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(idents, Some(1));
    let sessions = sqlx::query_scalar!("select count(*) from sessions")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(sessions, Some(2));
}

#[sqlx::test(migrations = "../../migrations")]
async fn open_redirect_is_blocked(db: PgPool) {
    let app = app(db, Some(discord_user(None)), true);
    for bad in ["//evil.com", "/%5Cevil.com", "https://evil.com", "evil"] {
        let (_, res) = do_login(&app, bad).await;
        assert_eq!(location(&res), format!("{WEB}/"), "return_to={bad}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn me_requires_valid_session(db: PgPool) {
    let app = app(db.clone(), Some(discord_user(None)), true);

    let res = send(&app, get("/me", None)).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_text(res).await, r#"{"error":"auth.unauthenticated"}"#);

    let (session, _) = do_login(&app, "/").await;
    let res = send(&app, get("/me", Some(&session))).await;
    assert_eq!(res.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(json["display_name"], "Quaker Global");
    assert_eq!(json["locale"], "pt");
    assert!(json["avatar_url"].is_null());
    assert!(json["id"].is_i64());

    sqlx::query!("update sessions set expires_at = now() - interval '1 second'")
        .execute(&db)
        .await
        .unwrap();
    let res = send(&app, get("/me", Some(&session))).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn logout_removes_session(db: PgPool) {
    let app = app(db.clone(), Some(discord_user(None)), true);
    let (session, _) = do_login(&app, "/").await;

    let post = |cookie: Option<&str>| {
        let mut b = Request::post("/auth/logout");
        if let Some(c) = cookie {
            b = b.header(header::COOKIE, c);
        }
        b.body(Body::empty()).unwrap()
    };

    let res = send(&app, post(Some(&session))).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert!(set_cookie(&res, "session").unwrap().contains("Max-Age=0"));
    let n = sqlx::query_scalar!("select count(*) from sessions")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(n, Some(0));

    let res = send(&app, get("/me", Some(&session))).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = send(&app, post(None)).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
