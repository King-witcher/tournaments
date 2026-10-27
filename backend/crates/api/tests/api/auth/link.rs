//! Ligar o Discord a uma conta e desligar.

use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;

use super::{
    app, body_text, discord_user, do_login, get, location, oauth_round_trip, send, signup,
    start_oauth,
};
use crate::common::{self, User, WEB, call, err, post};

const PASSWORD: &str = "senha forte 1";
const LINK: &str = "/auth/discord/link?return_to=/pt/account";

#[sqlx::test(migrations = "../../migrations")]
async fn password_account_links_discord(db: PgPool) {
    let app = app(db.clone(), Some(discord_user(Some("av"))), true);
    let me = signup(&app, "Rapha", PASSWORD).await;

    let (session, res) = oauth_round_trip(&app, LINK, Some(&me.cookie)).await;
    assert!(session.is_empty(), "ligar não cria sessão");
    assert_eq!(location(&res), format!("{WEB}/pt/account"));

    let (_, body) = common::get(&app, "/auth/logins", &me).await;
    assert_eq!(body, json!({ "discord": true, "username": "Rapha" }));
    let avatar = sqlx::query_scalar!("select avatar_url from users where id = $1", me.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        avatar.as_deref(),
        Some("https://cdn.discordapp.com/avatars/123/av.png")
    );

    // Agora o Discord entra na mesma conta.
    let (session, _) = do_login(&app, "/").await;
    let (_, body) = common::get(
        &app,
        "/me",
        &User {
            id: me.id,
            cookie: session,
        },
    )
    .await;
    assert_eq!(body["id"], me.id);

    // Já ligado: nem vai para o Discord.
    let res = send(&app, get(LINK, Some(&me.cookie))).await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
    assert_eq!(
        body_text(res).await,
        r#"{"error":"auth.discord_already_linked"}"#
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn link_requires_session(db: PgPool) {
    let app = app(db.clone(), Some(discord_user(None)), true);
    let res = send(&app, get(LINK, None)).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // A sessão também precisa estar no callback.
    let me = signup(&app, "Rapha", PASSWORD).await;
    let (state_cookie, state) = start_oauth(&app, LINK, Some(&me.cookie)).await;
    let res = send(
        &app,
        get(
            &format!("/auth/discord/callback?code=abc&state={state}"),
            Some(&state_cookie),
        ),
    )
    .await;
    assert_eq!(
        location(&res),
        format!("{WEB}/pt/account?auth_error=auth.unauthenticated")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn discord_of_another_account_is_refused(db: PgPool) {
    let app = app(db.clone(), Some(discord_user(None)), true);
    do_login(&app, "/").await;
    let me = signup(&app, "Rapha", PASSWORD).await;

    let (_, res) = oauth_round_trip(
        &app,
        "/auth/discord/link?return_to=/es/account",
        Some(&me.cookie),
    )
    .await;
    assert_eq!(
        location(&res),
        format!("{WEB}/es/account?auth_error=auth.discord_taken")
    );
    let (_, body) = common::get(&app, "/auth/logins", &me).await;
    assert_eq!(body["discord"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn link_errors_go_back_to_return_to(db: PgPool) {
    let failing = app(db.clone(), None, true);
    let me = signup(&failing, "Rapha", PASSWORD).await;
    let (_, res) = oauth_round_trip(&failing, LINK, Some(&me.cookie)).await;
    assert_eq!(
        location(&res),
        format!("{WEB}/pt/account?auth_error=auth.discord_failed")
    );

    let (state_cookie, _) = start_oauth(&failing, LINK, Some(&me.cookie)).await;
    let res = send(
        &failing,
        get(
            "/auth/discord/callback?error=access_denied",
            Some(&format!("{state_cookie}; {}", me.cookie)),
        ),
    )
    .await;
    assert_eq!(
        location(&res),
        format!("{WEB}/pt/account?auth_error=auth.discord_denied")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn unlink_needs_password_login(db: PgPool) {
    let app = app(db.clone(), Some(discord_user(Some("av"))), true);
    let (session, _) = do_login(&app, "/").await;
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

    let (status, _) = post(
        &app,
        "/auth/password",
        &me,
        json!({ "username": "quaker", "password": PASSWORD }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = unlink("errada!!").await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, err("auth.wrong_password"))
    );
    let (status, body) = unlink(PASSWORD).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({ "discord": false, "username": "quaker" }));
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
    let (session, _) = do_login(&app, "/").await;
    let (_, body) = common::get(
        &app,
        "/me",
        &User {
            id,
            cookie: session,
        },
    )
    .await;
    assert_ne!(body["id"], id);
}
