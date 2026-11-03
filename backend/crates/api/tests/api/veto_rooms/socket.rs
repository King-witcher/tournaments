//! `GET /veto-rooms/{token}/ws`: o `Origin`, o token, o aviso a cada mudança, o
//! eco de quem agiu, os limites de conexão e o fechamento quando a sala vence.
//! De ponta a ponta, com o app numa porta e o `tokio-tungstenite`.

use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use futures_util::StreamExt;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message, client::IntoClientRequest, protocol::frame::coding::CloseCode},
};
use tower::ServiceExt;

use super::*;

async fn handshake(app: &Router, token: &str, origin: Option<&str>) -> (StatusCode, Value) {
    let mut request = Request::get(format!("/veto-rooms/{token}/ws"));
    if let Some(origin) = origin {
        request = request.header(header::ORIGIN, origin);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn serve(state: api::AppState) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app_from(state)).await.unwrap();
    });
    format!("ws://{addr}")
}

async fn open(base: &str, token: &str, query: &str) -> Socket {
    let url = format!("{base}/veto-rooms/{token}/ws{query}");
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(header::ORIGIN, WEB.parse().unwrap());
    tokio_tungstenite::connect_async(request).await.unwrap().0
}

async fn next(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
            .await
            .expect("nenhuma mensagem em 5 s")
            .expect("o socket fechou")
            .unwrap();
        if let Message::Text(text) = message {
            return serde_json::from_str(&text).unwrap();
        }
    }
}

async fn silent(socket: &mut Socket) {
    let got = tokio::time::timeout(Duration::from_millis(300), socket.next()).await;
    assert!(got.is_err(), "chegou {got:?}");
}

#[tokio::test]
async fn origin_and_token_come_first() {
    let (db, _lease) = crate::db::test_db().await;
    let app = app(db);
    let links = links_of(&app, room(1, 1)).await;
    let forbidden = (StatusCode::FORBIDDEN, err("realtime.origin_forbidden"));
    assert_eq!(
        handshake(&app, &links.spectator, Some("https://evil.example")).await,
        forbidden
    );
    assert_eq!(handshake(&app, &links.spectator, None).await, forbidden);
    // Origem certa, mas sem pedir WebSocket.
    assert_eq!(
        handshake(&app, &links.spectator, Some(WEB)).await,
        (
            StatusCode::UPGRADE_REQUIRED,
            err("realtime.upgrade_required")
        )
    );
}

/// O pedido de WebSocket, para o que o app recusa antes de abrir.
async fn upgrade_status(base: &str, token: &str) -> u16 {
    let url = format!("{base}/veto-rooms/{token}/ws");
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(header::ORIGIN, WEB.parse().unwrap());
    let refused = tokio_tungstenite::connect_async(request).await.unwrap_err();
    let tokio_tungstenite::tungstenite::Error::Http(response) = refused else {
        panic!("{refused:?}");
    };
    response.status().as_u16()
}

#[tokio::test]
async fn a_token_that_does_not_exist_or_expired_does_not_open() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 1)).await;
    let base = serve(state).await;
    assert_eq!(upgrade_status(&base, &"1".repeat(22)).await, 404);
    sqlx::query("update veto_rooms set expires_at = now()")
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(upgrade_status(&base, &links.spectator).await, 404);
}

#[tokio::test]
async fn every_link_is_told_of_each_change_of_its_room_only() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 1)).await;
    let other = links_of(&app, room(1, 1)).await;
    let base = serve(state).await;
    let changed = json!({ "type": "changed" });

    let mut spectator = open(&base, &links.spectator, "").await;
    let mut manager = open(&base, &links.manage, "").await;
    let mut player = open(&base, &links.seats[1], "").await;
    let mut elsewhere = open(&base, &other.spectator, "").await;

    enter(&app, &links.seats[0], "king").await;
    assert_eq!(next(&mut spectator).await, changed);
    assert_eq!(next(&mut manager).await, changed);
    assert_eq!(next(&mut player).await, changed);
    silent(&mut elsewhere).await;

    // Entrar de novo não muda nada, e não avisa.
    enter(&app, &links.seats[0], "outro").await;
    silent(&mut spectator).await;

    // O socket não dá acesso a nada: o token continua sendo a credencial, e a
    // mudança é só um aviso.
    enter(&app, &links.seats[1], "thallis").await;
    assert_eq!(ready(&app, &links.seats[0]).await.0, StatusCode::OK);
    for _ in 0..2 {
        assert_eq!(next(&mut spectator).await, changed);
    }
    silent(&mut spectator).await;
}

#[tokio::test]
async fn the_tab_that_acted_does_not_get_its_own_change_back() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 1)).await;
    let base = serve(state).await;
    let tab = "0123456789abcdef";
    let mut mine = open(&base, &links.seats[0], &format!("?connection={tab}")).await;
    let mut other_tab = open(&base, &links.seats[0], "").await;
    let request = Request::post(format!("/veto-rooms/{}/join", links.seats[0]))
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-realtime-connection", tab)
        .body(Body::from(json!({ "name": "king" }).to_string()))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(next(&mut other_tab).await, json!({ "type": "changed" }));
    silent(&mut mine).await;
    // A mudança de outra pessoa chega.
    enter(&app, &links.seats[1], "thallis").await;
    assert_eq!(next(&mut mine).await, json!({ "type": "changed" }));
}

#[tokio::test]
async fn a_full_room_refuses_more_spectators_but_not_the_players() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 1)).await;
    let id: i64 = sqlx::query_scalar("select id from veto_rooms")
        .fetch_one(&db)
        .await
        .unwrap();
    let held: Vec<_> = (0..200)
        .map(|_| state.veto_rooms.join(id, true).unwrap())
        .collect();
    let base = serve(state.clone()).await;
    assert_eq!(upgrade_status(&base, &links.spectator).await, 503);
    // Quem joga entra; o espectador lê a sala, mesmo sem o socket.
    let mut player = open(&base, &links.seats[0], "").await;
    enter(&app, &links.seats[0], "king").await;
    assert_eq!(next(&mut player).await, json!({ "type": "changed" }));
    assert_eq!(detail(&app, &links.spectator).await["role"], "spectator");
    drop(held);
    // Saiu um, entra outro.
    let mut spectator = open(&base, &links.spectator, "").await;
    enter(&app, &links.seats[1], "thallis").await;
    assert_eq!(next(&mut spectator).await, json!({ "type": "changed" }));
}

#[tokio::test]
async fn the_socket_closes_when_the_room_expires() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let links = links_of(&app, room(1, 1)).await;
    let set_expiry = |sql: &'static str| {
        let db = db.clone();
        async move { sqlx::query(sql).execute(&db).await.unwrap() }
    };
    let base = serve(state).await;
    // Vence daqui a pouco, mas foi estendida antes: o socket continua.
    set_expiry("update veto_rooms set expires_at = now() + interval '200 milliseconds'").await;
    let mut extended = open(&base, &links.spectator, "").await;
    set_expiry("update veto_rooms set expires_at = now() + interval '1 hour'").await;
    silent(&mut extended).await;

    set_expiry("update veto_rooms set expires_at = now() + interval '200 milliseconds'").await;
    let mut socket = open(&base, &links.spectator, "").await;
    let closed = tokio::time::timeout(Duration::from_secs(5), socket.next())
        .await
        .expect("o socket não fechou")
        .expect("sem fechamento")
        .unwrap();
    let Message::Close(Some(frame)) = closed else {
        panic!("{closed:?}");
    };
    assert_eq!(frame.code, CloseCode::Library(4004));
    assert_eq!(frame.reason, "veto_room.not_found");
}
