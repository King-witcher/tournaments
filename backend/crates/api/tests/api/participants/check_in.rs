//! Check-in: o próprio inscrito e o organizador, só com o torneio em check-in.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{COPA, participants};
use crate::common::*;

/// `(apelido, check-in)` de cada inscrito.
fn check_ins(list: &Value) -> Vec<(&str, bool)> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["nickname"].as_str().unwrap(),
                p["checked_in"].as_bool().unwrap(),
            )
        })
        .collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn participants_check_in_only_during_check_in(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    let check_in = format!("{COPA}/check-in");

    let closed = (StatusCode::CONFLICT, err("tournament.check_in_closed"));
    assert_eq!(post_empty(&app, &check_in, &bia).await, closed);
    assert_eq!(delete(&app, &check_in, &bia).await, closed);

    let (status, _) = post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    assert_eq!(status, StatusCode::OK);
    for _ in 0..2 {
        assert_eq!(
            post_empty(&app, &check_in, &bia).await,
            (StatusCode::NO_CONTENT, Value::Null)
        );
    }
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(seen["checked_in"], true);
    assert_eq!(check_ins(&participants(&app, &caio).await), [("Bia", true)]);

    // Quem não está inscrito não faz check-in; desfazer é sempre sem erro.
    assert_eq!(
        post_empty(&app, &check_in, &caio).await,
        (StatusCode::CONFLICT, err("participant.not_registered"))
    );
    assert_eq!(
        delete(&app, &check_in, &caio).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        delete(&app, &check_in, &bia).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(seen["checked_in"], false);

    // Durante o check-in a inscrição está fechada, e sair também.
    assert_eq!(
        delete(&app, &format!("{COPA}/registration"), &bia).await,
        (StatusCode::CONFLICT, err("tournament.registration_closed"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn organizers_check_anyone_in(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    let of = |who: &User| format!("{COPA}/participants/{}/check-in", who.id);

    assert_eq!(
        put_empty(&app, &of(&bia), &ana).await,
        (StatusCode::CONFLICT, err("tournament.check_in_closed"))
    );
    post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;

    assert_eq!(
        put_empty(&app, &of(&bia), &ana).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    // Player não mexe no check-in dos outros.
    assert_eq!(
        put_empty(&app, &of(&ana), &bia).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        delete(&app, &of(&bia), &bia).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        put_empty(&app, &of(&caio), &ana).await,
        (StatusCode::NOT_FOUND, err("participant.not_found"))
    );
    let by = sqlx::query_scalar!(
        "select checked_in_by from participants where user_id = $1",
        bia.id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(by, Some(ana.id));

    // Inscrito pelo organizador durante o check-in entra sem check-in.
    let add = format!("{COPA}/participants");
    let (status, added) = post(&app, &add, &ana, json!({ "user_id": caio.id })).await;
    assert_eq!(status, StatusCode::CREATED, "{added}");
    assert_eq!(added["checked_in"], false);
    assert_eq!(
        check_ins(&participants(&app, &ana).await),
        [("Ana", false), ("Bia", true), ("Caio", false)]
    );

    assert_eq!(
        delete(&app, &of(&bia), &ana).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(
        check_ins(&participants(&app, &ana).await),
        [("Ana", false), ("Bia", false), ("Caio", false)]
    );
}
