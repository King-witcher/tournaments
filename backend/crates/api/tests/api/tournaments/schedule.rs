//! Data de início, janela de check-in e a abertura automática do check-in.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;

const COPA: &str = "/tenants/quake-br/tournaments/copa";

#[sqlx::test(migrations = "../../migrations")]
async fn publishing_needs_a_start_date(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let created =
        create_tournament(&app, &ana, with(duel("copa"), json!({ "starts_at": null }))).await;
    assert_eq!(created["starts_at"], Value::Null);
    assert_eq!(created["check_in_minutes"], 30);
    assert_eq!(created["check_in_opens_at"], Value::Null);
    assert_eq!(
        post_empty(&app, &format!("{COPA}/publish"), &ana).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.starts_at_required")
        )
    );

    // Com data (vinda em outro fuso, guardada em UTC), publica.
    let (status, updated) = patch(
        &app,
        COPA,
        &ana,
        json!({ "starts_at": "2099-01-01T17:00:00-03:00" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["starts_at"], "2099-01-01T20:00:00Z");
    assert_eq!(updated["check_in_opens_at"], "2099-01-01T19:30:00Z");
    publish(&app, &ana, "copa").await;

    // Publicado, a data não sai mais.
    assert_eq!(
        patch(&app, COPA, &ana, json!({ "starts_at": null })).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.starts_at_required")
        )
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn check_in_window_is_validated(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    assert_eq!(
        post(
            &app,
            "/tenants/quake-br/tournaments",
            &ana,
            with(duel("copa"), json!({ "check_in_minutes": 4 }))
        )
        .await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "tournament.check_in_minutes_invalid", "params": { "min": 5, "max": 1440 } })
        )
    );
    let off = create_tournament(
        &app,
        &ana,
        with(duel("copa"), json!({ "check_in_minutes": null })),
    )
    .await;
    assert_eq!(off["check_in_minutes"], Value::Null);
    assert_eq!(off["check_in_opens_at"], Value::Null);
    let (status, on) = patch(&app, COPA, &ana, json!({ "check_in_minutes": 60 })).await;
    assert_eq!(status, StatusCode::OK, "{on}");
    assert_eq!(on["check_in_opens_at"], "2099-01-01T19:00:00Z");
}

#[sqlx::test(migrations = "../../migrations")]
async fn start_date_and_window_change_only_before_the_start(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    set_status(&db, "copa", "check_in").await;
    let (status, _) = patch(
        &app,
        COPA,
        &ana,
        json!({ "starts_at": "2099-01-02T20:00:00Z", "check_in_minutes": 45 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    set_status(&db, "copa", "in_progress").await;
    let locked = (StatusCode::CONFLICT, err("tournament.schedule_locked"));
    assert_eq!(
        patch(&app, COPA, &ana, json!({ "starts_at": FAR_START })).await,
        locked
    );
    assert_eq!(
        patch(&app, COPA, &ana, json!({ "check_in_minutes": null })).await,
        locked
    );
    // Reenviar os valores atuais (o formulário inteiro) não é mudança.
    let (status, same) = patch(
        &app,
        COPA,
        &ana,
        json!({ "starts_at": "2099-01-02T20:00:00Z", "check_in_minutes": 45, "name": "Copa" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{same}");
    assert_eq!(same["name"], "Copa");
}

#[sqlx::test(migrations = "../../migrations")]
async fn check_in_opens_by_itself_when_the_window_opens(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;

    // Começa daqui a 31 minutos, check-in de 30: ainda não abriu.
    starts_in(&db, "copa", 31).await;
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(seen["status"], "registration");

    // Daqui a 29: a janela abriu, e quem lê já vê (e grava) o check-in.
    starts_in(&db, "copa", 29).await;
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &bia).await;
    assert_eq!(list[0]["status"], "check_in");
    let stored = sqlx::query_scalar!("select status from tournaments where slug = 'copa'")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(stored, "check_in");

    // Publicar com a janela já aberta já vem em check-in.
    let late = with(
        duel("tarde"),
        json!({ "starts_at": "2000-01-01T20:00:00Z" }),
    );
    create_tournament(&app, &ana, late).await;
    let (status, published) =
        post_empty(&app, "/tenants/quake-br/tournaments/tarde/publish", &ana).await;
    assert_eq!(status, StatusCode::OK, "{published}");
    assert_eq!(published["status"], "check_in");
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_check_in_registration_stays_open_until_the_start(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(
        &app,
        &ana,
        with(duel("copa"), json!({ "check_in_minutes": null })),
    )
    .await;
    publish(&app, &ana, "copa").await;
    starts_in(&db, "copa", -10).await;
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(seen["status"], "registration");
}

#[sqlx::test(migrations = "../../migrations")]
async fn turning_check_in_off_during_check_in_reopens_registration(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    starts_in(&db, "copa", 10).await;
    let (_, seen) = get(&app, COPA, &ana).await;
    assert_eq!(seen["status"], "check_in");

    let (status, off) = patch(&app, COPA, &ana, json!({ "check_in_minutes": null })).await;
    assert_eq!(status, StatusCode::OK, "{off}");
    assert_eq!(
        (&off["status"], &off["check_in_minutes"]),
        (&json!("registration"), &Value::Null)
    );
}
