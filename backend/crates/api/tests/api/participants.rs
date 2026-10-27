use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

const COPA: &str = "/tenants/quake-br/tournaments/copa";

fn nicknames(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|p| p["nickname"].as_str().unwrap())
        .collect()
}

async fn participants(app: &Router, who: &User) -> Value {
    let (status, list) = get(app, &format!("{COPA}/participants"), who).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    list
}

#[sqlx::test(migrations = "../../migrations")]
async fn members_register_themselves_in_public_tournaments(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let registration = format!("{COPA}/registration");

    // Rascunho: player nem vê; organizer vê, mas a inscrição não abriu.
    assert_eq!(
        post_empty(&app, &registration, &bia).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );
    assert_eq!(
        post_empty(&app, &registration, &ana).await,
        (StatusCode::CONFLICT, err("tournament.registration_closed"))
    );

    publish(&app, &ana, "copa").await;
    for _ in 0..2 {
        // Inscrever de novo não muda nada.
        assert_eq!(
            post_empty(&app, &registration, &bia).await,
            (StatusCode::NO_CONTENT, Value::Null)
        );
    }
    let list = participants(&app, &ana).await;
    assert_eq!(nicknames(&list), ["Bia"]);
    assert_eq!(list[0]["user_id"], bia.id);
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(
        (&seen["registered"], &seen["participants"]),
        (&json!(true), &json!(1))
    );

    for _ in 0..2 {
        assert_eq!(
            delete(&app, &registration, &bia).await,
            (StatusCode::NO_CONTENT, Value::Null)
        );
    }
    assert_eq!(participants(&app, &bia).await, json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn registration_closes_when_full(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = user(&db, "Caio").await;
    join(&app, &caio, &invite(&app, &ana, "quake-br").await).await;
    create_tournament(
        &app,
        &ana,
        with(duel("copa"), json!({ "max_participants": 2 })),
    )
    .await;
    publish(&app, &ana, "copa").await;
    let registration = format!("{COPA}/registration");

    post_empty(&app, &registration, &ana).await;
    post_empty(&app, &registration, &bia).await;
    assert_eq!(
        post_empty(&app, &registration, &caio).await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "tournament.full", "params": { "max": 2 } })
        )
    );
    assert_eq!(
        post(
            &app,
            &format!("{COPA}/participants"),
            &ana,
            json!({ "user_id": caio.id })
        )
        .await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "tournament.full", "params": { "max": 2 } })
        )
    );
    // Quem sai libera a vaga.
    delete(&app, &registration, &bia).await;
    assert_eq!(
        post_empty(&app, &registration, &caio).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(nicknames(&participants(&app, &ana).await), ["Ana", "Caio"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn private_needs_an_invite_or_the_organizer(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let private = with(duel("copa"), json!({ "visibility": "private" }));
    create_tournament(&app, &ana, private).await;
    publish(&app, &ana, "copa").await;
    let registration = format!("{COPA}/registration");
    let add = format!("{COPA}/participants");

    assert_eq!(
        post_empty(&app, &registration, &bia).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );
    assert_eq!(
        get(&app, &add, &bia).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );
    // Organizer e owner veem, mas também não se inscrevem sozinhos.
    assert_eq!(
        post_empty(&app, &registration, &ana).await,
        (StatusCode::CONFLICT, err("tournament.private"))
    );

    let (status, added) = post(&app, &add, &ana, json!({ "user_id": bia.id })).await;
    assert_eq!(status, StatusCode::CREATED, "{added}");
    assert_eq!(added["user_id"], bia.id);
    assert_eq!(added["nickname"], "Bia");
    assert_eq!(
        post(&app, &add, &ana, json!({ "user_id": bia.id })).await,
        (StatusCode::CONFLICT, err("participant.already_registered"))
    );
    assert_eq!(nicknames(&participants(&app, &bia).await), ["Bia"]);
    // Inscrita, a própria Bia pode sair; aí deixa de ver o torneio.
    assert_eq!(
        delete(&app, &registration, &bia).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&app, COPA, &bia).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );

    // Só membro do tenant entra, e só organizer ou owner inscreve outros.
    let eve = user(&db, "Eve").await;
    assert_eq!(
        post(&app, &add, &ana, json!({ "user_id": eve.id })).await,
        (StatusCode::NOT_FOUND, err("member.not_found"))
    );
    assert_eq!(
        post(&app, &add, &bia, json!({ "user_id": bia.id })).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn organizers_change_participants_until_the_start(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let add = format!("{COPA}/participants");
    let remove_bia = format!("{COPA}/participants/{}", bia.id);
    let registration = format!("{COPA}/registration");

    // Já no rascunho.
    let (status, _) = post(&app, &add, &ana, json!({ "user_id": bia.id })).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        delete(&app, &remove_bia, &bia).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        delete(&app, &remove_bia, &ana).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(
        delete(&app, &remove_bia, &ana).await,
        (StatusCode::NOT_FOUND, err("participant.not_found"))
    );
    assert_eq!(
        delete(&app, &format!("{COPA}/participants/abc"), &ana).await,
        (StatusCode::BAD_REQUEST, err("request.invalid_path"))
    );

    publish(&app, &ana, "copa").await;
    post_empty(&app, &registration, &bia).await;
    set_status(&db, "copa", "in_progress").await;
    let locked = (StatusCode::CONFLICT, err("tournament.participants_locked"));
    assert_eq!(delete(&app, &remove_bia, &ana).await, locked);
    assert_eq!(
        post(&app, &add, &ana, json!({ "user_id": ana.id })).await,
        locked
    );
    let closed = (StatusCode::CONFLICT, err("tournament.registration_closed"));
    assert_eq!(delete(&app, &registration, &bia).await, closed);
    assert_eq!(post_empty(&app, &registration, &ana).await, closed);
    // Quem já está inscrito "se inscrever" de novo continua sem erro.
    assert_eq!(
        post_empty(&app, &registration, &bia).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(nicknames(&participants(&app, &bia).await), ["Bia"]);
}
