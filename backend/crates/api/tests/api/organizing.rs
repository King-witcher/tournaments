//! `GET /me/organizing`: o que espera a organização em todas as comunidades
//! que quem vê organiza (mapas contestados, hora de iniciar), e o que avisa a
//! home da organização pelo canal `me` quando isso muda.

use axum::{
    Router,
    http::{Method, StatusCode},
};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::matches::{
    COPA, confirm, contest, copa, game, players, report, set_game, single, start,
};
use crate::realtime::{clear, ids, me, outbox};

async fn organizing(app: &Router, who: &User) -> Value {
    let (status, body) = get(app, "/me/organizing", who).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

fn empty() -> Value {
    json!({ "contested": [], "start_due": [] })
}

/// Se o outbox tem a mudança de "Seus torneios" exatamente para `users`.
async fn has_me(db: &PgPool, users: &[&User]) -> bool {
    outbox(db).await.contains(&me(&ids(users)))
}

/// Um duelo de `quake-br` com check-in, publicado, com `who` inscritos e só
/// então o início a `minutes` minutos de agora (negativo: já passou).
async fn published(app: &Router, db: &PgPool, ana: &User, slug: &str, who: &[&User], minutes: i32) {
    create_tournament(app, ana, duel(slug)).await;
    publish(app, ana, slug).await;
    register(app, slug, who).await;
    starts_in(db, slug, minutes).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn requires_login(db: PgPool) {
    let app = app(db);
    let (status, body) = call(&app, Method::GET, "/me/organizing", None, None).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, err("auth.unauthenticated"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn contested_maps_wait_for_the_organization(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    assert_eq!(organizing(&app, &ana).await, empty());

    let (status, body) = report(&app, &p[1], "wb-1-1", 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    // Pendente não espera a organização: confirma sozinho.
    assert_eq!(organizing(&app, &ana).await, empty());
    let (status, body) = contest(&app, &p[0], "wb-1-1", 1, "foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let found = organizing(&app, &olga).await;
    let item = &found["contested"][0];
    assert_eq!(
        item["tournament"],
        json!({
            "tenant_slug": "quake-br",
            "tenant_name": "Tenant quake-br",
            "slug": "copa",
            "name": "Copa de Outubro",
            "translations": []
        })
    );
    assert_eq!(
        (&item["key"], &item["single"], &item["game"]),
        (&json!("wb-1-1"), &json!(true), &json!(1))
    );
    assert_eq!(
        (&item["map"], &item["contested_by"]),
        (&json!("Awoken"), &json!("P1"))
    );
    assert!(item["contested_at"].is_string());
    assert_eq!(found["start_due"], json!([]));
    assert_eq!(organizing(&app, &ana).await, found);
    // Quem joga não organiza: nada para ele, nem para quem contestou.
    for who in [&bia, &p[0], &p[1]] {
        assert_eq!(organizing(&app, who).await, empty());
    }

    // Resolvido, sai da lista.
    let (status, body) = confirm(&app, &olga, "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(organizing(&app, &ana).await, empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn start_due_with_the_check_ins(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    // Começa daqui a uma hora: não espera nada ainda.
    published(&app, &db, &ana, "depois", &[], 60).await;
    // Rascunho com a data passada não conta.
    create_tournament(&app, &ana, duel("rascunho")).await;
    starts_in(&db, "rascunho", -5).await;
    // Sem check-in, com a hora passada: todos os inscritos entram.
    create_tournament(&app, &ana, copa(json!({ "slug": "livre" }))).await;
    publish(&app, &ana, "livre").await;
    register(&app, "livre", &[&bia]).await;
    starts_in(&db, "livre", -30).await;
    // Com check-in, a hora passou há 5 minutos e um dos dois fez.
    published(&app, &db, &ana, "copa", &[&bia, &caio], -5).await;
    let (status, _) = post_empty(&app, &format!("{COPA}/check-in"), &bia).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let found = organizing(&app, &ana).await;
    assert_eq!(found["contested"], json!([]));
    let due = found["start_due"].as_array().unwrap();
    let summary: Vec<_> = due
        .iter()
        .map(|t| {
            (
                t["tournament"]["slug"].as_str().unwrap().to_owned(),
                t["status"].as_str().unwrap().to_owned(),
                t["has_check_in"].as_bool().unwrap(),
                t["registered"].as_i64().unwrap(),
                t["checked_in"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("livre".to_owned(), "registration".to_owned(), false, 1, 0),
            ("copa".to_owned(), "check_in".to_owned(), true, 2, 1),
        ]
    );
    assert!(due[0]["starts_at"].is_string());
    assert_eq!(organizing(&app, &bia).await, empty());

    // Iniciado, sai da lista.
    let (status, _) = post_empty(&app, &format!("{COPA}/check-in"), &caio).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let found = organizing(&app, &ana).await;
    assert_eq!(found["start_due"].as_array().unwrap().len(), 1);
    assert_eq!(found["start_due"][0]["tournament"]["slug"], "livre");
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_communities_organized_now(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    published(&app, &db, &ana, "copa", &[], -5).await;
    // Outra comunidade, de Bia, em que Ana e Olga são players.
    create(&app, &bia, "argentina").await;
    let token = invite(&app, &bia, "argentina").await;
    join(&app, &ana, &token).await;
    join(&app, &olga, &token).await;
    let liga =
        json!({ "slug": "liga", "check_in_minutes": null, "starts_at": "2020-01-01T20:00:00Z" });
    let (status, body) = post(
        &app,
        "/tenants/argentina/tournaments",
        &bia,
        with(duel("liga"), liga),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let (status, body) =
        post_empty(&app, "/tenants/argentina/tournaments/liga/publish", &bia).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let slugs = |found: &Value| -> Vec<String> {
        let due = found["start_due"].as_array().unwrap();
        due.iter()
            .map(|t| {
                let t = &t["tournament"];
                let tenant = t["tenant_slug"].as_str().unwrap();
                format!("{tenant}/{}", t["slug"].as_str().unwrap())
            })
            .collect()
    };
    assert_eq!(slugs(&organizing(&app, &ana).await), ["quake-br/copa"]);
    assert_eq!(slugs(&organizing(&app, &olga).await), ["quake-br/copa"]);
    assert_eq!(slugs(&organizing(&app, &bia).await), ["argentina/liga"]);

    // Quem sai da comunidade deixa de organizá-la.
    let (status, _) = delete(&app, "/tenants/quake-br/me", &olga).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(organizing(&app, &olga).await, empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn contests_reach_the_organization_home(db: PgPool) {
    let (app, ana, _) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let staff = [&ana, &olga];

    // Enviar não muda o que espera a organização.
    clear(&db).await;
    let (status, body) = report(&app, &p[1], "wb-1-1", 1, game("b", "awoken")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(!has_me(&db, &staff).await);

    // Contestar, sim (também para o organizer que não recebe o aviso).
    clear(&db).await;
    let (status, body) = contest(&app, &p[0], "wb-1-1", 1, "foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(has_me(&db, &staff).await);

    // Resolver com outro resultado (correção do contestado), também.
    clear(&db).await;
    let (status, body) = set_game(&app, &olga, "wb-1-1", 1, game("a", "awoken"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(has_me(&db, &staff).await);

    // Corrigir um mapa confirmado não muda nada para a organização.
    clear(&db).await;
    let (status, body) = set_game(&app, &olga, "wb-1-1", 1, game("b", "awoken"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(!has_me(&db, &staff).await);

    // Desfazer o início apaga os contestados.
    let (status, body) = report(&app, &p[1], "wb-1-1", 2, game("b", "exile")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = contest(&app, &p[0], "wb-1-1", 2, "não").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    clear(&db).await;
    let (status, body) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(has_me(&db, &staff).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_start_reaches_the_organization_home(db: PgPool) {
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let bia = member(&app, &db, &ana, "Bia").await;
    let caio = member(&app, &db, &ana, "Caio").await;
    let staff = [&ana, &olga];
    published(&app, &db, &ana, "copa", &[&bia, &caio], 10).await;

    // Antes da hora, o check-in não muda a home da organização.
    clear(&db).await;
    let (status, _) = post_empty(&app, &format!("{COPA}/check-in"), &bia).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(!has_me(&db, &staff).await);

    // A hora chega: o agendador avisa e a home recarrega.
    starts_in(&db, "copa", -1).await;
    clear(&db).await;
    let tick = api::realtime::tick(&state, Utc::now())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(tick.reminded, 1);
    assert!(has_me(&db, &staff).await);

    // Depois da hora, cada check-in muda a contagem.
    clear(&db).await;
    let (status, _) = post_empty(&app, &format!("{COPA}/check-in"), &caio).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(has_me(&db, &staff).await);

    // Iniciar tira o torneio da lista.
    clear(&db).await;
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(has_me(&db, &staff).await);

    // Desfeito, volta (a hora já passou).
    clear(&db).await;
    let (status, body) = delete(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(has_me(&db, &staff).await);

    // Adiar também tira.
    clear(&db).await;
    let uri = "/tenants/quake-br/tournaments/copa";
    let (status, body) = patch(&app, uri, &ana, json!({ "starts_at": FAR_START })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(has_me(&db, &staff).await);
    assert_eq!(organizing(&app, &ana).await, empty());

    // Iniciar antes da hora não muda a lista: não manda nada à organização.
    clear(&db).await;
    let (status, body) = post_empty(&app, &format!("{COPA}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(!has_me(&db, &staff).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_owner_reloads_the_home(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    clear(&db).await;
    let body = json!({ "user_id": bia.id, "confirm_slug": "quake-br" });
    let (status, body) = post(&app, "/tenants/quake-br/transfer-ownership", &ana, body).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(has_me(&db, &[&bia]).await);
}
