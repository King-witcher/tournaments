//! O endereço do torneio: muda só no rascunho, pela organização, com a
//! validação da criação. A capa está em `crate::images`.

use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::common::*;
use crate::realtime::{clear, outbox, tenant, tournament};

const URI: &str = "/tenants/quake-br/tournaments/copa";

fn move_to(slug: &str) -> Value {
    json!({ "slug": slug })
}

#[tokio::test]
async fn the_organization_moves_a_draft() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;
    create_tournament(&app, &carla, duel("copa")).await;
    let (status, link) = get(&app, &format!("{URI}/invite"), &carla).await;
    assert_eq!(status, StatusCode::OK, "{link}");
    assert_eq!(
        patch(&app, URI, &bia, move_to("liga")).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );

    clear(&db).await;
    let body = json!({ "slug": "liga", "name": "Liga de Novembro" });
    let (status, moved) = patch(&app, URI, &carla, body).await;
    assert_eq!(status, StatusCode::OK, "{moved}");
    assert_eq!(moved["slug"], "liga");
    assert_eq!(moved["name"], "Liga de Novembro");

    let new = "/tenants/quake-br/tournaments/liga";
    assert_eq!(
        get(&app, URI, &carla).await,
        (StatusCode::NOT_FOUND, err("tournament.not_found"))
    );
    let (status, seen) = get(&app, new, &carla).await;
    assert_eq!(status, StatusCode::OK, "{seen}");
    assert_eq!(seen["slug"], "liga");
    // As abas abertas vão para o endereço novo; a lista recarrega por ele.
    assert_eq!(
        outbox(&db).await,
        [
            json!({ "kind": "moved", "tenant": "quake-br", "tournament": "copa", "to": "liga" }),
            tenant("quake-br"),
            tournament("liga", None),
        ]
    );

    // O link de inscrição usa o token e continua valendo; o antigo fica livre.
    let dani = user(&db, "Dani").await;
    let token = link["token"].as_str().unwrap();
    let (status, preview) = get(&app, &format!("/invites/{token}"), &dani).await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["tournament"]["slug"], "liga");
    create_tournament(&app, &carla, duel("copa")).await;
}

#[tokio::test]
async fn the_rules_are_the_ones_of_the_creation() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    create_tournament(&app, &ana, duel("ocupado")).await;
    let cases = [
        (
            "A",
            StatusCode::UNPROCESSABLE_ENTITY,
            "tournament.slug_invalid",
        ),
        (
            "new",
            StatusCode::UNPROCESSABLE_ENTITY,
            "tournament.slug_reserved",
        ),
        ("ocupado", StatusCode::CONFLICT, "tournament.slug_taken"),
    ];
    for (slug, status, code) in cases {
        // Recusado, o resto do pedido também não grava.
        let body = json!({ "slug": slug, "name": "Outro nome" });
        assert_eq!(patch(&app, URI, &ana, body).await, (status, err(code)));
    }
    let (_, seen) = get(&app, URI, &ana).await;
    assert_eq!(seen["name"], "Copa de Outubro");

    // O mesmo endereço não é troca, e o de outra comunidade não conflita.
    let (status, body) = patch(&app, URI, &ana, move_to("copa")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["slug"], "copa");
}

#[tokio::test]
async fn after_publishing_only_the_name_changes() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    for status in ["registration", "check_in", "in_progress", "finished"] {
        set_status(&db, "copa", status).await;
        assert_eq!(
            patch(&app, URI, &ana, move_to("liga")).await,
            (StatusCode::CONFLICT, err("tournament.slug_locked")),
            "{status}"
        );
        // Reenviar o mesmo, com o nome novo, passa.
        let body = json!({ "slug": "copa", "name": format!("Copa {status}") });
        let (code, saved) = patch(&app, URI, &ana, body).await;
        assert_eq!(code, StatusCode::OK, "{status}: {saved}");
        assert_eq!(saved["name"], format!("Copa {status}"));
    }
}
