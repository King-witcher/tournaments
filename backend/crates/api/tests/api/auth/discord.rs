//! Entrar com Discord (com PKCE), `/me` e sair.

use std::time::Duration;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use redis::Cmd;
use serde_json::json;

use super::{
    CHALLENGE, OTHER_VERIFIER, STATE, VERIFIER, app, authorize, body_text, discord_user, do_login,
    exchange, get, json_request, pair, send, send_json, session_key, set_cookie,
};
use crate::common::{err, redis};

#[tokio::test]
async fn without_config_is_503() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db, None, false);
    let (status, body) = authorize(&app, "discord", "login", None).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            err("auth.discord_not_configured")
        )
    );
    let (status, body, _) = exchange(&app, "discord", "login", None, VERIFIER).await;
    assert_eq!(
        (status, body),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            err("auth.discord_not_configured")
        )
    );
}

#[tokio::test]
async fn authorize_returns_the_discord_url_with_the_challenge() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db, None, true);
    let body = json!({ "intent": "login", "state": STATE, "code_challenge": CHALLENGE });
    let res = send(
        &app,
        json_request(Method::POST, "/auth/discord/authorize", None, body),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(
        res.headers().get(header::SET_COOKIE).is_none(),
        "a API não guarda nada do fluxo"
    );

    let body: serde_json::Value = serde_json::from_str(&body_text(res).await).unwrap();
    let url = body["url"].as_str().unwrap();
    assert!(
        url.starts_with("https://discord.com/oauth2/authorize?"),
        "{url}"
    );
    for part in [
        "response_type=code",
        "client_id=client-1",
        "scope=identify",
        &format!("state={STATE}"),
        &format!("code_challenge={CHALLENGE}"),
        "code_challenge_method=S256",
        // A página de callback do front.
        "redirect_uri=http%3A%2F%2Flocalhost%3A4001%2Fauth%2Fdiscord%2Fcallback",
    ] {
        assert!(url.contains(part), "{url} sem {part}");
    }
}

#[tokio::test]
async fn malformed_pkce_is_refused() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    let refused = (
        StatusCode::UNPROCESSABLE_ENTITY,
        err("request.invalid_body"),
    );
    for (state, challenge) in [
        ("curto", CHALLENGE),
        (STATE, &CHALLENGE[..42]),
        (STATE, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-c="),
    ] {
        let body = json!({ "intent": "login", "state": state, "code_challenge": challenge });
        let (status, body, _) =
            send_json(&app, Method::POST, "/auth/discord/authorize", None, body).await;
        assert_eq!((status, body), refused, "{state} {challenge}");
    }
    let body = json!({ "intent": "entrar", "state": STATE, "code_challenge": CHALLENGE });
    let (status, _, _) = send_json(&app, Method::POST, "/auth/discord/authorize", None, body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    for verifier in [
        &VERIFIER[..42],
        "verifier com espaço e mais de quarenta e três",
    ] {
        let (status, body, _) = exchange(&app, "discord", "login", None, verifier).await;
        assert_eq!((status, body), refused, "{verifier}");
    }
    let body = json!({ "code": "", "code_verifier": VERIFIER });
    let (status, body, _) = send_json(&app, Method::POST, "/auth/discord/login", None, body).await;
    assert_eq!((status, body), refused);
}

#[tokio::test]
async fn login_errors_create_nothing() {
    let (db, _lease) = crate::db::test_db().await;
    let failed = (StatusCode::BAD_GATEWAY, err("auth.discord_failed"));

    // O code que saiu para outra aba (outro challenge) não serve.
    let ok_app = app(db.clone(), Some(discord_user(None)), true);
    let (status, body, session) = exchange(&ok_app, "discord", "login", None, OTHER_VERIFIER).await;
    assert_eq!((status, body), failed);
    assert_eq!(session, None);

    let failing = app(db.clone(), None, true);
    let (status, body, session) = exchange(&failing, "discord", "login", None, VERIFIER).await;
    assert_eq!((status, body), failed);
    assert_eq!(session, None);
    let n = sqlx::query_scalar!("select count(*) from users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(n, Some(0));
}

#[tokio::test]
async fn login_creates_user_identity_and_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(Some("hash1"))), true);
    let body = json!({ "code": "abc", "code_verifier": VERIFIER });
    let res = send(
        &app,
        json_request(Method::POST, "/auth/discord/login", None, body),
    )
    .await;
    assert_eq!(res.status(), StatusCode::OK);
    let cookie = set_cookie(&res, "session").unwrap();
    for attr in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=2592000"] {
        assert!(cookie.contains(attr), "{cookie} sem {attr}");
    }
    let session = pair(&cookie);
    let body: serde_json::Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(body["display_name"], "Quaker Global");
    assert_eq!(body["deletion_cancelled"], false);

    let user = sqlx::query!("select id, display_name, avatar_url from users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(body["id"], user.id);
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

    // O Redis guarda a sessão pelo hash, nunca pelo token.
    let redis = redis(&db);
    let owner: Option<i64> = redis
        .query(&Cmd::hget(session_key(&redis, &session), "user_id"))
        .await
        .unwrap();
    assert_eq!(owner, Some(user.id));
    let token = session.strip_prefix("session=").unwrap();
    let by_token: bool = redis
        .query(&Cmd::exists(redis.key(format_args!("session:{token}"))))
        .await
        .unwrap();
    assert!(!by_token);
}

#[tokio::test]
async fn second_login_reuses_user_and_updates_avatar() {
    let (db, _lease) = crate::db::test_db().await;
    let first = app(db.clone(), Some(discord_user(Some("old"))), true);
    do_login(&first).await;
    let second = app(db.clone(), Some(discord_user(Some("new"))), true);
    let (session, _) = do_login(&second).await;

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
    let res = send(&second, get("/auth/sessions", Some(&session))).await;
    let sessions: serde_json::Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(sessions.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn me_requires_valid_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);

    let res = send(&app, get("/me", None)).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_text(res).await, r#"{"error":"auth.unauthenticated"}"#);

    let (session, _) = do_login(&app).await;
    let res = send(&app, get("/me", Some(&session))).await;
    assert_eq!(res.status(), StatusCode::OK);
    let json: serde_json::Value = serde_json::from_str(&body_text(res).await).unwrap();
    assert_eq!(json["display_name"], "Quaker Global");
    assert_eq!(json["locale"], "pt");
    assert!(json["avatar_url"].is_null());
    assert!(json["id"].is_i64());

    // A sessão expira sozinha no Redis, em 30 dias.
    let redis = redis(&db);
    let key = session_key(&redis, &session);
    let ttl: i64 = redis.query(&Cmd::ttl(&key)).await.unwrap();
    assert!((30 * 86_400 - 60..=30 * 86_400).contains(&ttl), "{ttl}");
    let _: bool = redis.query(&Cmd::pexpire(&key, 1)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(5)).await;
    let res = send(&app, get("/me", Some(&session))).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_removes_session() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db.clone(), Some(discord_user(None)), true);
    let (session, _) = do_login(&app).await;

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
    // Sai a sessão e sai do índice da conta.
    let redis = redis(&db);
    let user_id = sqlx::query_scalar!("select id from users")
        .fetch_one(&db)
        .await
        .unwrap();
    let left: (bool, i64) = redis
        .query_pipe(
            redis::pipe()
                .exists(session_key(&redis, &session))
                .zcard(redis.key(format_args!("user_sessions:{user_id}"))),
        )
        .await
        .unwrap();
    assert_eq!(left, (false, 0));

    let res = send(&app, get("/me", Some(&session))).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let res = send(&app, post(None)).await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
