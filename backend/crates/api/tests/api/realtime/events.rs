//! O que cada ação grava no outbox, na transação dela: a mudança de
//! comunidade, de torneio (com a partida) e de "Seus torneios", o acesso
//! perdido e as sessões encerradas.

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use api::auth::create_session;

use super::*;
use crate::common::*;
use crate::matches::{COPA, copa, game, players, report, single, start};

#[sqlx::test(migrations = "../../migrations")]
async fn tournaments_and_registrations(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    clear(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    // Rascunho sem inscritos: ninguém em "Seus torneios".
    assert_eq!(
        outbox(&db).await,
        [tenant("quake-br"), tournament("copa", None)]
    );

    publish(&app, &ana, "copa").await;
    clear(&db).await;
    register(&app, "copa", &[&bia]).await;
    assert_eq!(
        outbox(&db).await,
        [tenant("quake-br"), tournament("copa", None), me(&[bia.id])]
    );

    // Quem sai dos inscritos também recebe a mudança de "Seus torneios".
    clear(&db).await;
    let (status, _) = delete(&app, &format!("{COPA}/registration"), &bia).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        outbox(&db).await,
        [tenant("quake-br"), tournament("copa", None), me(&[bia.id])]
    );

    // Uma ação recusada não deixa nada (a transação volta).
    clear(&db).await;
    let (status, _) = post_empty(&app, &format!("{COPA}/publish"), &ana).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(outbox(&db).await, Vec::<serde_json::Value>::new());
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_same_event_goes_once_per_transaction(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let uri = format!("{COPA}/participants");
    let (status, _) = post(&app, &uri, &ana, json!({ "user_id": bia.id })).await;
    assert_eq!(status, StatusCode::CREATED);
    // Publicar com a janela de check-in já aberta: o torneio muda duas vezes
    // na mesma transação (publicado e check-in aberto), e cada evento entra
    // uma vez, com o aviso aos inscritos.
    starts_in(&db, "copa", 10).await;
    clear(&db).await;
    publish(&app, &ana, "copa").await;
    let events = outbox(&db).await;
    assert_eq!(
        events[..3],
        [tenant("quake-br"), tournament("copa", None), me(&[bia.id])]
    );
    assert_eq!(events.len(), 4);
    assert_eq!(notices(&events)[0].0, "check_in.opened");
}

#[sqlx::test(migrations = "../../migrations")]
async fn matches_say_which_match_changed(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    start(
        &app,
        &ana,
        copa(single(3, 3)),
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    clear(&db).await;
    let (status, body) = report(&app, &p[0], "wb-1-1", 1, game("a", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let events = outbox(&db).await;
    let changes: Vec<_> = events
        .iter()
        .filter(|e| e["kind"] != "notice")
        .cloned()
        .collect();
    assert_eq!(
        changes,
        [
            tournament("copa", Some("wb-1-1")),
            me(&ids(&[&p[0], &p[3]]))
        ]
    );
    // Mais o aviso ao adversário (ver `notices`).
    assert_eq!(events.len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn members_and_invites(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    clear(&db).await;
    invite(&app, &ana, "quake-br").await;
    assert_eq!(outbox(&db).await, [tenant("quake-br")]);

    clear(&db).await;
    let (status, _) = patch(
        &app,
        "/tenants/quake-br/me",
        &bia,
        json!({ "nickname": "B" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(outbox(&db).await, [tenant("quake-br")]);

    // Removida com inscrição aberta: sai do torneio e perde o acesso.
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    clear(&db).await;
    let uri = format!("/tenants/quake-br/members/{}", bia.id);
    let (status, _) = delete(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        outbox(&db).await,
        [
            tenant("quake-br"),
            tournament("copa", None),
            me(&[bia.id]),
            json!({ "kind": "access_lost", "user": bia.id, "tenant": "quake-br" }),
        ]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn ended_sessions(db: PgPool) {
    let app = app(db.clone());
    let ana = user(&db, "Ana").await;
    let other = create_session(&redis(&db), ana.id, None).await.unwrap();
    let id = |token: &str| token_id(token);

    let (status, _) = delete(&app, "/auth/sessions", &ana).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        outbox(&db).await,
        [json!({ "kind": "sessions_ended", "user": ana.id, "sessions": [id(&other)] })]
    );

    clear(&db).await;
    let (status, _) = call(
        &app,
        axum::http::Method::POST,
        "/auth/logout",
        Some(&ana),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let mine = ana.cookie.trim_start_matches("session=");
    assert_eq!(
        outbox(&db).await,
        [json!({ "kind": "sessions_ended", "user": ana.id, "sessions": [id(mine)] })]
    );
}

/// O id público da sessão do token (16 caracteres do SHA-256 em hexadecimal).
fn token_id(token: &str) -> String {
    let hash = api::shared::token::hash_token(token);
    api::shared::token::hex(&hash)[..16].to_owned()
}
