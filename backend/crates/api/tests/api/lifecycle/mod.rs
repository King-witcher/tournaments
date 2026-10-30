//! Transições depois de publicar: abrir o check-in, iniciar e desfazer o início; a desistência e o W.O. automático em
//! `withdrawal`.

mod withdrawal;

use axum::{Router, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

const COPA: &str = "/tenants/quake-br/tournaments/copa";

/// Faz a transição `action` (`check-in/open`, `start`).
async fn transition(app: &Router, who: &User, action: &str) -> (StatusCode, Value) {
    post_empty(app, &format!("{COPA}/{action}"), who).await
}

/// Faz a transição esperando que dê certo; devolve o status novo.
async fn ok(app: &Router, who: &User, action: &str) -> Value {
    let (status, body) = transition(app, who, action).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["status"].clone()
}

async fn check_in(app: &Router, who: &User) {
    let (status, body) = post_empty(app, &format!("{COPA}/check-in"), who).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

/// `(user_id, seed)` de cada inscrito, na ordem de inscrição.
async fn seeds(db: &PgPool) -> Vec<(i64, Option<i32>)> {
    sqlx::query!(
        r#"select p.user_id, t.seed as "seed?" from participants p
           left join teams t on t.tournament_id = p.tournament_id and t.id = p.team_id
           order by p.registered_at, p.user_id"#
    )
    .fetch_all(db)
    .await
    .unwrap()
    .into_iter()
    .map(|r| (r.user_id, r.seed))
    .collect()
}

#[tokio::test]
async fn start_keeps_the_checked_in_in_seed_order() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    let duda = member(&app, &db, &ana, "Duda").await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia, &caio, &duda]).await;
    assert_eq!(ok(&app, &ana, "check-in/open").await, "check_in");
    for who in [&ana, &bia, &duda] {
        check_in(&app, who).await;
    }
    let (status, _) = seed(&app, &ana, "copa", &[&bia, &duda, &ana]).await;
    assert_eq!(status, StatusCode::OK);
    // Duda desiste depois do seeding; Caio nunca fez check-in.
    let (status, _) = delete(&app, &format!("{COPA}/check-in"), &duda).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    assert_eq!(ok(&app, &ana, "start").await, "in_progress");
    assert_eq!(
        seeds(&db).await,
        [
            (ana.id, Some(2)),
            (bia.id, Some(1)),
            (caio.id, None),
            (duda.id, None)
        ]
    );

    // Inscrições, check-ins e seeding congelam.
    assert_eq!(
        post_empty(&app, &format!("{COPA}/check-in"), &caio).await,
        (StatusCode::CONFLICT, err("tournament.check_in_closed"))
    );
    assert_eq!(
        delete(&app, &format!("{COPA}/registration"), &bia).await,
        (StatusCode::CONFLICT, err("tournament.registration_closed"))
    );
    assert_eq!(
        post_empty(&app, &format!("{COPA}/seeding/shuffle"), &ana).await,
        (StatusCode::CONFLICT, err("tournament.participants_locked"))
    );
    assert_eq!(
        transition(&app, &ana, "start").await,
        (StatusCode::CONFLICT, err("tournament.already_started"))
    );
}

#[tokio::test]
async fn start_without_seeds_draws_them() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    ok(&app, &ana, "check-in/open").await;
    check_in(&app, &ana).await;
    check_in(&app, &bia).await;
    ok(&app, &ana, "start").await;

    let mut drawn: Vec<Option<i32>> = seeds(&db).await.into_iter().map(|(_, s)| s).collect();
    drawn.sort();
    assert_eq!(drawn, [Some(1), Some(2)]);
    let seed = sqlx::query_scalar!("select seeding_seed from tournaments")
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(seed.is_some());
}

#[tokio::test]
async fn start_needs_the_check_in_and_two_eligible() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    assert_eq!(
        transition(&app, &ana, "start").await,
        (StatusCode::CONFLICT, err("tournament.not_published"))
    );
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    assert_eq!(
        transition(&app, &ana, "start").await,
        (StatusCode::CONFLICT, err("tournament.check_in_not_open"))
    );

    ok(&app, &ana, "check-in/open").await;
    check_in(&app, &bia).await;
    assert_eq!(
        transition(&app, &ana, "start").await,
        (
            StatusCode::CONFLICT,
            json!({ "error": "tournament.not_enough_eligible", "params": { "min": 2, "eligible": 1 } })
        )
    );
    let (_, seen) = get(&app, COPA, &ana).await;
    assert_eq!(seen["status"], "check_in");
}

#[tokio::test]
async fn undo_start_goes_back_to_check_in() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    assert_eq!(
        delete(&app, &format!("{COPA}/start"), &ana).await,
        (StatusCode::CONFLICT, err("tournament.not_in_progress"))
    );
    ok(&app, &ana, "check-in/open").await;
    check_in(&app, &ana).await;
    check_in(&app, &bia).await;
    ok(&app, &ana, "start").await;

    let (status, undone) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{undone}");
    assert_eq!(undone["status"], "check_in");
    // De volta ao check-in, tudo mexe de novo.
    assert_eq!(
        delete(&app, &format!("{COPA}/check-in"), &bia).await.0,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn without_check_in_registration_goes_straight_to_in_progress() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let no_check_in = with(duel("copa"), json!({ "check_in_minutes": null }));
    create_tournament(&app, &ana, no_check_in).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    assert_eq!(
        transition(&app, &ana, "check-in/open").await,
        (StatusCode::CONFLICT, err("tournament.check_in_disabled"))
    );
    assert_eq!(
        post_empty(&app, &format!("{COPA}/check-in"), &bia).await,
        (StatusCode::CONFLICT, err("tournament.check_in_closed"))
    );

    // Todos os inscritos são elegíveis.
    let (_, order) = get(&app, &format!("{COPA}/seeding"), &ana).await;
    assert_eq!(order.as_array().unwrap().len(), 2);
    assert_eq!(ok(&app, &ana, "start").await, "in_progress");
    let (_, undone) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(undone["status"], "registration");
}

#[tokio::test]
async fn there_is_no_way_back_from_check_in_to_registration_but_turning_it_off() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    starts_in(&db, "copa", 10).await;
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(seen["status"], "check_in");
    check_in(&app, &bia).await;

    // Adiar o início não fecha o check-in; desligá-lo volta para a inscrição
    // e o check-in feito fica.
    let (status, _) = patch(&app, COPA, &ana, json!({ "starts_at": FAR_START })).await;
    assert_eq!(status, StatusCode::OK);
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(seen["status"], "check_in");
    let (status, seen) = patch(&app, COPA, &ana, json!({ "check_in_minutes": null })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        (&seen["status"], &seen["checked_in"]),
        (&json!("registration"), &json!(false))
    );
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(
        (&seen["status"], &seen["checked_in"]),
        (&json!("registration"), &json!(true))
    );
}

#[tokio::test]
async fn players_do_not_drive_the_lifecycle() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    let forbidden = (StatusCode::FORBIDDEN, err("tenant.forbidden"));
    for action in ["check-in/open", "start"] {
        assert_eq!(transition(&app, &bia, action).await, forbidden, "{action}");
    }
    assert_eq!(
        delete(&app, &format!("{COPA}/start"), &bia).await,
        forbidden
    );
}
