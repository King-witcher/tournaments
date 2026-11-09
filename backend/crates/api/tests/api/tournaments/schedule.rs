//! Data de início, só informativa, e o check-in ligado ou desligado.

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
    let (status, published) = post_empty(&app, &format!("{COPA}/publish"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{published}");
    assert_eq!(published["status"], "registration");
    assert_eq!(published["starts_at"], Value::Null);

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

    // ... e perde de novo.
    let (status, cleared) = patch(&app, COPA, &ana, json!({ "starts_at": null })).await;
    assert_eq!(status, StatusCode::OK, "{cleared}");
    assert_eq!(cleared["starts_at"], Value::Null);
    assert_eq!(cleared["status"], "registration");
}

#[tokio::test]
async fn a_past_date_is_only_information() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let past = "2020-01-01T20:00:00Z";

    // Cria e publica com a data no passado.
    for publish in [false, true] {
        let slug = format!("copa-{publish}");
        let body = with(
            duel(&slug),
            json!({ "starts_at": past, "publish": publish }),
        );
        let created = create_tournament(&app, &ana, body).await;
        assert_eq!(created["starts_at"], past);
    }
    publish(&app, &ana, "copa-false").await;

    // Põe a data no passado.
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    let (status, body) = patch(&app, COPA, &ana, json!({ "starts_at": past })).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // A hora chegou e nada mudou, nem ao ler nem na passada do agendador.
    let tick = api::realtime::tick(&state, chrono::Utc::now())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(tick, api::realtime::Tick::default());
    let (_, seen) = get(&app, COPA, &ana).await;
    assert_eq!(seen["status"], "registration");
    let (_, list) = get(&app, "/tenants/quake-br/tournaments", &ana).await;
    let statuses: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["registration"; 3]);
}

#[tokio::test]
async fn the_check_in_is_on_or_off() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    // Ligado, se a criação não diz.
    let created = create_tournament(&app, &ana, duel("liga")).await;
    assert_eq!(created["check_in"], true);

    let off = create_tournament(&app, &ana, with(duel("copa"), json!({ "check_in": false }))).await;
    assert_eq!(off["check_in"], false);
    let (status, on) = patch(&app, COPA, &ana, json!({ "check_in": true })).await;
    assert_eq!(status, StatusCode::OK, "{on}");
    assert_eq!(on["check_in"], true);
}

#[tokio::test]
async fn start_date_and_check_in_change_only_before_the_start() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    open_check_in(&app, &ana, "copa").await;
    let (status, _) = patch(
        &app,
        COPA,
        &ana,
        json!({ "starts_at": "2099-01-02T20:00:00Z" }),
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
        patch(&app, COPA, &ana, json!({ "check_in": false })).await,
        locked
    );
    // Reenviar os valores atuais (o formulário inteiro) não é mudança.
    let (status, same) = patch(
        &app,
        COPA,
        &ana,
        json!({ "starts_at": "2099-01-02T20:00:00Z", "check_in": true, "name": "Copa" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{same}");
    assert_eq!(same["name"], "Copa");
}

#[tokio::test]
async fn clearing_the_date_with_check_in_open_keeps_it_open() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    open_check_in(&app, &ana, "copa").await;
    let (_, cleared) = patch(&app, COPA, &ana, json!({ "starts_at": null })).await;
    assert_eq!(cleared["status"], "check_in");
}

#[tokio::test]
async fn turning_check_in_off_during_check_in_reopens_registration() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    open_check_in(&app, &ana, "copa").await;
    let (status, off) = patch(&app, COPA, &ana, json!({ "check_in": false })).await;
    assert_eq!(status, StatusCode::OK, "{off}");
    assert_eq!(
        (&off["status"], &off["check_in"]),
        (&json!("registration"), &json!(false))
    );
}
