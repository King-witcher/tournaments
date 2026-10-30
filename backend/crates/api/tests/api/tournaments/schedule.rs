//! Data de início, janela de check-in e a abertura automática do check-in.

use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::common::*;

const COPA: &str = "/tenants/quake-br/tournaments/copa";

#[tokio::test]
async fn the_start_date_is_optional_even_after_publishing() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let created =
        create_tournament(&app, &ana, with(duel("copa"), json!({ "starts_at": null }))).await;
    assert_eq!(created["starts_at"], Value::Null);
    assert_eq!(created["check_in_minutes"], 30);
    assert_eq!(created["check_in_opens_at"], Value::Null);
    // Sem data, publica: a inscrição abre e nada mais anda sozinho.
    let (status, published) = post_empty(&app, &format!("{COPA}/publish"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{published}");
    assert_eq!(published["status"], "registration");
    assert_eq!(published["starts_at"], Value::Null);
    assert_eq!(published["check_in_opens_at"], Value::Null);

    // Publicado, ganha data (vinda em outro fuso, guardada em UTC)...
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

    // ... e perde de novo.
    let (status, cleared) = patch(&app, COPA, &ana, json!({ "starts_at": null })).await;
    assert_eq!(status, StatusCode::OK, "{cleared}");
    assert_eq!(cleared["starts_at"], Value::Null);
    assert_eq!(cleared["status"], "registration");
}

#[tokio::test]
async fn without_a_date_check_in_never_opens_by_itself() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, with(duel("copa"), json!({ "starts_at": null }))).await;
    publish(&app, &ana, "copa").await;
    let (_, seen) = get(&app, COPA, &bia).await;
    assert_eq!(seen["status"], "registration");
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &bia).await;
    assert_eq!(list[0]["status"], "registration");
    assert_eq!(list[0]["starts_at"], Value::Null);

    // O organizador abre o check-in; tirar a data depois não o fecha.
    let (status, opened) = post_empty(&app, &format!("{COPA}/check-in/open"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{opened}");
    assert_eq!(opened["status"], "check_in");
    starts_in(&db, "copa", 10).await;
    let (status, cleared) = patch(&app, COPA, &ana, json!({ "starts_at": null })).await;
    assert_eq!(status, StatusCode::OK, "{cleared}");
    assert_eq!(cleared["status"], "check_in");
}

#[tokio::test]
async fn clearing_the_date_with_check_in_open_keeps_it_open() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    starts_in(&db, "copa", 10).await;
    let (_, seen) = get(&app, COPA, &ana).await;
    assert_eq!(seen["status"], "check_in");
    let (_, cleared) = patch(&app, COPA, &ana, json!({ "starts_at": null })).await;
    assert_eq!(cleared["status"], "check_in");
}

#[tokio::test]
async fn check_in_window_is_validated() {
    let (db, _lease) = crate::db::test_db().await;
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

#[tokio::test]
async fn start_date_and_window_change_only_before_the_start() {
    let (db, _lease) = crate::db::test_db().await;
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

#[tokio::test]
async fn check_in_opens_by_itself_when_the_window_opens() {
    let (db, _lease) = crate::db::test_db().await;
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
}

#[tokio::test]
async fn the_start_must_leave_the_check_in_window_ahead() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let at = |minutes: i64| (chrono::Utc::now() + chrono::Duration::minutes(minutes)).to_rfc3339();
    let create = |slug: &str, patch: Value| {
        post(
            &app,
            "/tenants/quake-br/tournaments",
            &ana,
            with(duel(slug), patch),
        )
    };
    // Passada, ou dentro da janela de 30 minutos: recusa, rascunho ou não.
    for publish in [false, true] {
        let (status, body) =
            create("copa", json!({ "starts_at": at(-5), "publish": publish })).await;
        assert_eq!(
            (status, body),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                err("tournament.starts_at_past")
            )
        );
        let (status, body) =
            create("copa", json!({ "starts_at": at(10), "publish": publish })).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({ "error": "tournament.starts_at_too_soon", "params": { "minutes": 30 } })
        );
    }
    // Sem check-in, só não pode ser passada.
    let (status, body) = create(
        "copa",
        json!({ "starts_at": at(1), "check_in_minutes": null }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (status, _) = create(
        "outra",
        json!({ "starts_at": at(-1), "check_in_minutes": null }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    // Com a antecedência da janela, passa.
    let (status, body) = create("longe", json!({ "starts_at": at(40) })).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}

#[tokio::test]
async fn editing_checks_the_start_only_when_it_changes() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    let at = |minutes: i64| (chrono::Utc::now() + chrono::Duration::minutes(minutes)).to_rfc3339();
    // A data nova tem de deixar a janela pela frente.
    assert_eq!(
        patch(&app, COPA, &ana, json!({ "starts_at": at(-5) })).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.starts_at_past")
        )
    );
    let (status, body) = patch(&app, COPA, &ana, json!({ "starts_at": at(10) })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "tournament.starts_at_too_soon");
    // Com a data já passada (o torneio atrasou), editar outra coisa continua
    // permitido, inclusive reenviar a mesma data.
    starts_in(&db, "copa", -20).await;
    let (_, seen) = get(&app, COPA, &ana).await;
    let (status, body) = patch(
        &app,
        COPA,
        &ana,
        json!({ "name": "Copa 2", "starts_at": seen["starts_at"] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["name"], "Copa 2");
}

#[tokio::test]
async fn a_draft_whose_start_fell_behind_cannot_be_published() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let uri = format!("{COPA}/publish");
    starts_in(&db, "copa", 10).await;
    let (status, body) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "tournament.starts_at_too_soon");
    starts_in(&db, "copa", -10).await;
    assert_eq!(
        post_empty(&app, &uri, &ana).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.starts_at_past")
        )
    );
    starts_in(&db, "copa", 60).await;
    let (status, published) = post_empty(&app, &uri, &ana).await;
    assert_eq!(status, StatusCode::OK, "{published}");
    assert_eq!(published["status"], "registration");
}

#[tokio::test]
async fn without_check_in_registration_stays_open_until_the_start() {
    let (db, _lease) = crate::db::test_db().await;
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

#[tokio::test]
async fn turning_check_in_off_during_check_in_reopens_registration() {
    let (db, _lease) = crate::db::test_db().await;
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
