//! Excluir a comunidade: só o owner agenda, digitando o slug, e cancela; até
//! a data ela é só leitura (escrita, convites e o que o agendador faria
//! sozinho); torneio em andamento impede; na data, o agendador apaga tudo.

use api::realtime;
use axum::{
    Router,
    http::{Method, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{COPA, copa, single, start};
use crate::realtime::{clear, outbox};

const DELETION: &str = "/tenants/quake-br/deletion";

async fn schedule(app: &Router, by: &User, slug: &str) -> (StatusCode, Value) {
    post(app, DELETION, by, json!({ "confirm_slug": slug })).await
}

fn scheduled() -> (StatusCode, Value) {
    (StatusCode::CONFLICT, err("tenant.deletion_scheduled"))
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_the_owner_schedules_and_cancels(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;

    assert_eq!(
        schedule(&app, &olga, "quake-br").await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        schedule(&app, &ana, "quake").await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tenant.confirmation_mismatch")
        )
    );
    let before = Utc::now();
    let (status, body) = schedule(&app, &ana, " quake-br ").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let due: chrono::DateTime<Utc> =
        serde_json::from_value(body["deletion_scheduled_for"].clone()).unwrap();
    assert!(due >= before + Duration::days(7) - Duration::seconds(5));
    assert_eq!(body["role"], "owner");
    // Todos os membros veem a data, na comunidade e na lista.
    let (_, seen) = get(&app, "/tenants/quake-br", &bia).await;
    assert_eq!(
        seen["deletion_scheduled_for"],
        body["deletion_scheduled_for"]
    );
    let (_, mine) = get(&app, "/tenants", &bia).await;
    assert_eq!(
        mine[0]["deletion_scheduled_for"],
        body["deletion_scheduled_for"]
    );
    // Já agendada.
    assert_eq!(schedule(&app, &ana, "quake-br").await, scheduled());

    assert_eq!(
        delete(&app, DELETION, &olga).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    let (status, body) = delete(&app, DELETION, &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["deletion_scheduled_for"], Value::Null);
    assert_eq!(
        delete(&app, DELETION, &ana).await,
        (StatusCode::CONFLICT, err("tenant.deletion_not_scheduled"))
    );
    // Volta a aceitar escritas.
    let (status, _) = patch(
        &app,
        "/tenants/quake-br/me",
        &bia,
        json!({ "nickname": "Bia QC" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_community_is_read_only_until_the_deletion(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    let token = invite(&app, &ana, "quake-br").await;
    create_tournament(&app, &ana, duel("aberta")).await;
    publish(&app, &ana, "aberta").await;
    let (status, body) = schedule(&app, &ana, "quake-br").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Escrita de qualquer domínio, de qualquer papel.
    let nickname = json!({ "nickname": "Bia QC" });
    assert_eq!(
        patch(&app, "/tenants/quake-br/me", &bia, nickname).await,
        scheduled()
    );
    assert_eq!(
        post_empty(&app, "/tenants/quake-br/invites", &ana).await,
        scheduled()
    );
    assert_eq!(
        post(&app, "/tenants/quake-br/tournaments", &ana, duel("nova")).await,
        scheduled()
    );
    let registration = "/tenants/quake-br/tournaments/aberta/registration";
    assert_eq!(post_empty(&app, registration, &bia).await, scheduled());
    let transfer = json!({ "user_id": bia.id, "confirm_slug": "quake-br" });
    assert_eq!(
        post(&app, "/tenants/quake-br/transfer-ownership", &ana, transfer).await,
        scheduled()
    );
    // O convite não vale, nem a prévia.
    let accept = format!("/invites/{token}/accept");
    let stranger = user(&db, "Zeca").await;
    assert_eq!(post_empty(&app, &accept, &stranger).await, scheduled());
    assert_eq!(
        call(&app, Method::GET, &format!("/invites/{token}"), None, None).await,
        scheduled()
    );
    // Ler continua.
    for uri in [
        "/tenants/quake-br",
        "/tenants/quake-br/members",
        "/tenants/quake-br/tournaments",
        "/tenants/quake-br/tournaments/aberta",
    ] {
        let (status, body) = get(&app, uri, &bia).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {body}");
    }
    // Sair também.
    let (status, body) = delete(&app, "/tenants/quake-br/me", &caio).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_scheduler_waits_while_it_is_read_only(db: PgPool) {
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    for slug in ["janela", "hora"] {
        create_tournament(&app, &ana, duel(slug)).await;
        publish(&app, &ana, slug).await;
    }
    // O check-in da janela abriu; o início da hora passou.
    starts_in(&db, "janela", 10).await;
    starts_in(&db, "hora", -1).await;
    sqlx::query("update tournaments set check_in_minutes = null where slug = 'hora'")
        .execute(&db)
        .await
        .unwrap();
    let (status, body) = schedule(&app, &ana, "quake-br").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!((tick.opened, tick.reminded), (0, 0));
    // Nem ao ler.
    let (_, t) = get(&app, "/tenants/quake-br/tournaments/janela", &ana).await;
    assert_eq!(t["status"], "registration");

    let (status, body) = delete(&app, DELETION, &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!((tick.opened, tick.reminded), (1, 1));
}

#[sqlx::test(migrations = "../../migrations")]
async fn not_with_a_tournament_in_progress(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    start(&app, &ana, copa(single(3, 3)), &[&ana, &bia]).await;
    assert_eq!(
        schedule(&app, &ana, "quake-br").await,
        (StatusCode::CONFLICT, err("tenant.tournament_in_progress"))
    );
    // Desfeito o início, agenda.
    let (status, body) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = schedule(&app, &ana, "quake-br").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        post_empty(&app, &format!("{COPA}/start"), &ana).await,
        scheduled()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn on_the_date_everything_inside_goes(db: PgPool) {
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    let bia = user(&db, "Bia").await;
    create(&app, &ana, "quake-br").await;
    create(&app, &ana, "outra").await;
    join(&app, &bia, &invite(&app, &ana, "quake-br").await).await;
    start(&app, &ana, copa(single(1, 1)), &[&ana, &bia]).await;
    set_status(&db, "copa", "finished").await;
    let (status, body) = schedule(&app, &ana, "quake-br").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Antes da data, nada sai.
    let later = Utc::now() + Duration::days(7) + Duration::minutes(1);
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.deleted_tenants, 0);

    clear(&db).await;
    let tick = realtime::tick(&state, later).await.unwrap().unwrap();
    assert_eq!(tick.deleted_tenants, 1);
    let left = sqlx::query_scalar!("select slug from tenants")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(left, ["outra"]);
    for table in [
        "tournaments",
        "participants",
        "tenant_invites",
        "match_results",
    ] {
        let sql = format!("select count(*) from {table}");
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(n, 0, "{table}");
    }
    // Os membros perdem o acesso na hora.
    let events = outbox(&db).await;
    for who in [&ana, &bia] {
        let lost = json!({ "kind": "access_lost", "user": who.id, "tenant": "quake-br" });
        assert!(events.contains(&lost), "{events:?}");
    }
    assert_eq!(
        get(&app, "/tenants/quake-br", &bia).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    // A outra comunidade da Ana fica.
    let (_, mine) = get(&app, "/tenants", &ana).await;
    assert_eq!(mine[0]["slug"], "outra");
}
