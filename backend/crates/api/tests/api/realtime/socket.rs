//! `GET /ws`: sessão e `Origin` antes do upgrade, assinaturas com o acesso
//! da leitura, o que chega a cada conexão e o fechamento quando a sessão
//! acaba. De ponta a ponta com o app numa porta e o `tokio-tungstenite`.

use std::time::Duration;

use api::realtime;
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message, client::IntoClientRequest, protocol::frame::coding::CloseCode},
};
use tower::ServiceExt;

use crate::common::*;
use crate::matches::COPA;

async fn handshake(db: &PgPool, cookie: Option<&str>, origin: Option<&str>) -> (StatusCode, Value) {
    let mut request = Request::get("/ws");
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    if let Some(origin) = origin {
        request = request.header(header::ORIGIN, origin);
    }
    let response = app(db.clone())
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn session_and_origin_come_first() {
    let (db, _lease) = crate::db::test_db().await;
    let ana = user(&db, "Ana").await;
    assert_eq!(
        handshake(&db, None, Some(WEB)).await,
        (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
    );
    let forbidden = (StatusCode::FORBIDDEN, err("realtime.origin_forbidden"));
    assert_eq!(
        handshake(&db, Some(&ana.cookie), Some("https://evil.example")).await,
        forbidden
    );
    assert_eq!(handshake(&db, Some(&ana.cookie), None).await, forbidden);
    // Sessão e origem certas, mas sem pedir WebSocket.
    assert_eq!(
        handshake(&db, Some(&ana.cookie), Some(WEB)).await,
        (
            StatusCode::UPGRADE_REQUIRED,
            err("realtime.upgrade_required")
        )
    );
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Sobe o app numa porta livre e devolve o endereço.
async fn serve(state: api::AppState) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app_from(state)).await.unwrap();
    });
    format!("ws://{addr}/ws")
}

/// Conecta e devolve o socket e a primeira mensagem, a das notificações não
/// lidas.
async fn open(url: &str, who: &User) -> (Socket, Value) {
    let mut request = url.into_client_request().unwrap();
    let headers = request.headers_mut();
    headers.insert(header::COOKIE, who.cookie.parse().unwrap());
    headers.insert(header::ORIGIN, WEB.parse().unwrap());
    let mut socket = tokio_tungstenite::connect_async(request).await.unwrap().0;
    let unread = next(&mut socket).await;
    (socket, unread)
}

/// Conecta sem notificações não lidas.
async fn connect(url: &str, who: &User) -> Socket {
    let (socket, unread) = open(url, who).await;
    assert_eq!(unread, json!({ "type": "unread", "count": 0 }));
    socket
}

/// A próxima mensagem de texto, em JSON (falha se não chega em 5 s).
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

/// Nada chega em meio segundo.
async fn silent(socket: &mut Socket) {
    let got = tokio::time::timeout(Duration::from_millis(500), socket.next()).await;
    assert!(got.is_err(), "chegou {got:?}");
}

async fn send(socket: &mut Socket, message: Value) -> Value {
    socket
        .send(Message::Text(message.to_string().into()))
        .await
        .unwrap();
    next(socket).await
}

fn subscribed(tenant: &str, tournament: Option<&str>) -> Value {
    json!({ "type": "subscribed", "tenant": tenant, "tournament": tournament })
}

#[tokio::test]
async fn subscriptions_follow_read_access() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let (ana, bia) = (user(&db, "Ana").await, user(&db, "Bia").await);
    create(&app, &ana, "quake-br").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    // Inscrição fechada não esconde: quem não está inscrito assina. O rascunho
    // some para o jogador.
    let invite_only = with(duel("fechada"), json!({ "admission": "invite_only" }));
    create_tournament(&app, &ana, invite_only).await;
    publish(&app, &ana, "fechada").await;
    create_tournament(&app, &ana, duel("rascunho")).await;
    // Torneio de outra comunidade.
    let cris = user(&db, "Cris").await;
    create(&app, &cris, "quake-ar").await;
    let (status, _) = post(&app, "/tenants/quake-ar/tournaments", &cris, duel("copa")).await;
    assert_eq!(status, StatusCode::CREATED);

    let url = serve(state).await;
    let mut socket = connect(&url, &bia).await;
    let subscribe = |tenant: &str, tournament: Option<&str>| json!({ "type": "subscribe", "tenant": tenant, "tournament": tournament });
    let error = |code: &str, tenant: &str, tournament: Option<&str>| json!({ "type": "error", "code": code, "tenant": tenant, "tournament": tournament });
    assert_eq!(
        send(&mut socket, subscribe("quake-ar", Some("copa"))).await,
        error("tenant.not_found", "quake-ar", Some("copa"))
    );
    assert_eq!(
        send(&mut socket, subscribe("quake-br", Some("rascunho"))).await,
        error("tournament.not_found", "quake-br", Some("rascunho"))
    );
    assert_eq!(
        send(&mut socket, subscribe("quake-br", Some("fechada"))).await,
        subscribed("quake-br", Some("fechada"))
    );
    assert_eq!(
        send(&mut socket, json!({ "type": "dance" })).await,
        json!({ "type": "error", "code": "request.invalid_body", "tenant": null, "tournament": null })
    );
    assert_eq!(
        send(
            &mut socket,
            json!({ "type": "unsubscribe", "tenant": "quake-br", "tournament": "fechada" })
        )
        .await,
        json!({ "type": "unsubscribed", "tenant": "quake-br", "tournament": "fechada" })
    );
}

#[tokio::test]
async fn changes_reach_the_subscribers_until_access_is_lost() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let (ana, bia) = (user(&db, "Ana").await, user(&db, "Bia").await);
    create(&app, &ana, "quake-br").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    realtime::dispatch_once(&state).await.unwrap();

    let url = serve(state.clone()).await;
    let mut socket = connect(&url, &bia).await;
    let topic = json!({ "type": "subscribe", "tenant": "quake-br", "tournament": "copa" });
    assert_eq!(
        send(&mut socket, topic).await,
        subscribed("quake-br", Some("copa"))
    );
    realtime::dispatch_once(&state).await.unwrap();
    silent(&mut socket).await;

    register(&app, "copa", &[&bia]).await;
    realtime::dispatch_once(&state).await.unwrap();
    assert_eq!(
        next(&mut socket).await,
        json!({ "type": "changed", "scope": "tenant", "tenant": "quake-br" })
    );
    assert_eq!(
        next(&mut socket).await,
        json!({ "type": "changed", "scope": "tournament", "tenant": "quake-br", "tournament": "copa", "match": null })
    );
    assert_eq!(
        next(&mut socket).await,
        json!({ "type": "changed", "scope": "me" })
    );

    // Removida: o aviso de acesso perdido, e a comunidade não chega mais.
    let (status, _) = delete(&app, &format!("/tenants/quake-br/members/{}", bia.id), &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    realtime::dispatch_once(&state).await.unwrap();
    let mut got = vec![
        next(&mut socket).await,
        next(&mut socket).await,
        next(&mut socket).await,
    ];
    got.push(next(&mut socket).await);
    assert!(
        got.contains(&json!({ "type": "access_lost", "tenant": "quake-br" })),
        "{got:?}"
    );
    let (status, _) = patch(&app, COPA, &ana, json!({ "name": "Copa Nova" })).await;
    assert_eq!(status, StatusCode::OK);
    realtime::dispatch_once(&state).await.unwrap();
    silent(&mut socket).await;
}

#[tokio::test]
async fn notices_and_ended_sessions() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let (ana, bia) = (user(&db, "Ana").await, user(&db, "Bia").await);
    create(&app, &ana, "quake-br").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    realtime::dispatch_once(&state).await.unwrap();

    let url = serve(state.clone()).await;
    let mut socket = connect(&url, &bia).await;
    let mut other_tab = connect(&url, &bia).await;
    // Sem assinar nada: "Seus torneios" e o aviso chegam igual.
    let (status, _) = post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    realtime::dispatch_once(&state).await.unwrap();
    assert_eq!(
        next(&mut socket).await,
        json!({ "type": "changed", "scope": "me" })
    );
    assert_eq!(
        next(&mut socket).await,
        json!({
            "type": "notify", "code": "check_in.opened",
            "params": { "tournament": "Copa de Outubro" }, "path": "/quake-br/copa"
        })
    );
    for _ in 0..2 {
        next(&mut other_tab).await;
    }

    // O aviso ficou no sininho: a aba que abre agora já conta, e ver numa aba
    // leva a contagem às outras.
    let (mut new_tab, unread) = open(&url, &bia).await;
    assert_eq!(unread, json!({ "type": "unread", "count": 1 }));
    let (_, list) = get(&app, "/me/notifications", &bia).await;
    let up_to = list[0]["id"].clone();
    let (status, _) = post(
        &app,
        "/me/notifications/read",
        &bia,
        json!({ "up_to": up_to }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    realtime::dispatch_once(&state).await.unwrap();
    for tab in [&mut socket, &mut other_tab, &mut new_tab] {
        assert_eq!(next(tab).await, json!({ "type": "unread", "count": 0 }));
    }
    drop(new_tab);

    // Sair fecha os sockets desta sessão (as duas abas são do mesmo cookie).
    let (status, _) = call(
        &app,
        axum::http::Method::POST,
        "/auth/logout",
        Some(&bia),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    realtime::dispatch_once(&state).await.unwrap();
    for tab in [&mut socket, &mut other_tab] {
        let closed = loop {
            match tokio::time::timeout(Duration::from_secs(5), tab.next())
                .await
                .unwrap()
            {
                Some(Ok(Message::Close(frame))) => break frame,
                Some(Ok(_)) => continue,
                other => panic!("esperava o fechamento: {other:?}"),
            }
        };
        assert_eq!(closed.unwrap().code, CloseCode::Library(4001));
    }
}
