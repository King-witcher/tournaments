//! Trocar o endereço da comunidade: quem troca, a validação da criação, o
//! antigo livre na hora e o que acompanha a troca na mesma transação (o sininho,
//! a fila do Discord e os avisos ainda no outbox). As imagens estão em
//! `crate::images`, os screenshots em `crate::screenshots::uploads` e a aba
//! aberta em `crate::realtime::socket`.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::discord::{TOKEN, WEBHOOK};
use crate::realtime::{clear, me, outbox, tenant};

const URI: &str = "/tenants/quake-br";

fn move_to(slug: &str) -> Value {
    json!({ "slug": slug })
}

#[tokio::test]
async fn the_owner_moves_it_and_the_old_address_is_free_at_once() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;
    let link = invite(&app, &ana, "quake-br").await;

    for who in [&bia, &carla] {
        assert_eq!(
            patch(&app, URI, who, move_to("quake-brasil")).await,
            (StatusCode::FORBIDDEN, err("tenant.forbidden"))
        );
    }

    let (status, body) = patch(&app, URI, &ana, move_to("quake-brasil")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["slug"], "quake-brasil");
    assert_eq!(body["role"], "owner");

    // O antigo não leva a nada, nem responde diferente de quem é de fora.
    assert_eq!(
        get(&app, URI, &bia).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    let (status, seen) = get(&app, "/tenants/quake-brasil", &bia).await;
    assert_eq!(status, StatusCode::OK, "{seen}");
    assert_eq!(seen["slug"], "quake-brasil");
    let (_, mine) = get(&app, "/tenants", &bia).await;
    assert_eq!(mine[0]["slug"], "quake-brasil");

    // O link de convite usa o token e continua valendo; a prévia diz o novo.
    let (status, preview) = call_preview(&app, &link).await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(preview["tenant_slug"], "quake-brasil");
    let dani = user(&db, "Dani").await;
    let joined = join(&app, &dani, &link).await;
    assert_eq!(joined["tenant"]["slug"], "quake-brasil", "{joined}");

    // Outra comunidade pega o antigo, sem nenhum vínculo com a primeira.
    let eva = user(&db, "Eva").await;
    create(&app, &eva, "quake-br").await;
    assert_eq!(
        get(&app, URI, &bia).await,
        (StatusCode::NOT_FOUND, err("tenant.not_found"))
    );
    let (_, theirs) = get(&app, URI, &eva).await;
    assert_eq!(theirs["role"], "owner");
    assert_eq!(theirs["name"], "Tenant quake-br");
}

async fn call_preview(app: &axum::Router, token: &str) -> (StatusCode, Value) {
    call(
        app,
        axum::http::Method::GET,
        &format!("/invites/{token}"),
        None,
        None,
    )
    .await
}

#[tokio::test]
async fn the_rules_are_the_ones_of_the_creation() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let other = user(&db, "Eva").await;
    create(&app, &other, "ocupado").await;
    let cases = [
        ("A", StatusCode::UNPROCESSABLE_ENTITY, "tenant.slug_invalid"),
        (
            "fim-",
            StatusCode::UNPROCESSABLE_ENTITY,
            "tenant.slug_invalid",
        ),
        (
            "invite",
            StatusCode::UNPROCESSABLE_ENTITY,
            "tenant.slug_reserved",
        ),
        ("ocupado", StatusCode::CONFLICT, "tenant.slug_taken"),
    ];
    for (slug, status, code) in cases {
        // Recusado, o resto do pedido também não grava.
        let body = json!({ "slug": slug, "name": "Outro nome" });
        assert_eq!(patch(&app, URI, &ana, body).await, (status, err(code)));
    }
    let (status, seen) = get(&app, URI, &bia).await;
    assert_eq!(status, StatusCode::OK, "{seen}");
    assert_eq!(seen["name"], "Tenant quake-br");

    // O mesmo endereço não é troca: ninguém é levado a lugar nenhum.
    clear(&db).await;
    let (status, body) = patch(&app, URI, &ana, move_to("quake-br")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(outbox(&db).await, [tenant("quake-br")]);

    // Só leitura com a exclusão agendada.
    let confirm = json!({ "confirm_slug": "quake-br" });
    let (status, body) = post(&app, &format!("{URI}/deletion"), &ana, confirm).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        patch(&app, URI, &ana, move_to("quake-brasil")).await,
        (StatusCode::CONFLICT, err("tenant.deletion_scheduled"))
    );
}

#[tokio::test]
async fn the_open_tabs_and_the_homes_of_the_members_are_told() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    clear(&db).await;
    let (status, body) = patch(&app, URI, &ana, move_to("quake-brasil")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let mut members = vec![ana.id, bia.id];
    members.sort_unstable();
    assert_eq!(
        outbox(&db).await,
        [
            json!({ "kind": "moved", "tenant": "quake-br", "tournament": null, "to": "quake-brasil" }),
            me(&members),
            tenant("quake-brasil"),
        ]
    );
}

async fn store_notice(db: &PgPool, who: &User, path: &str) -> i64 {
    sqlx::query_scalar(
        "insert into notifications (user_id, notice)
         values ($1, jsonb_build_object('code', 'tournament.start_due',
                                        'params', jsonb_build_object('tournament', 'Copa'),
                                        'path', $2::text))
         returning id",
    )
    .bind(who.id)
    .bind(path)
    .fetch_one(db)
    .await
    .unwrap()
}

async fn notice_path(db: &PgPool, id: i64) -> String {
    sqlx::query_scalar("select notice ->> 'path' from notifications where id = $1")
        .bind(id)
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn the_bell_keeps_leading_to_the_community() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let series = store_notice(&db, &bia, "/quake-br/copa/matches/wb-1-1").await;
    let settings = store_notice(&db, &ana, "/quake-br/settings").await;
    let home = store_notice(&db, &bia, "/quake-br").await;
    // Começa igual, mas é de outra comunidade.
    let lookalike = store_notice(&db, &bia, "/quake-br-2/copa").await;
    sqlx::query("update notifications set read_at = now() where id = $1")
        .bind(settings)
        .execute(&db)
        .await
        .unwrap();

    let (status, body) = patch(&app, URI, &ana, move_to("quake-brasil")).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    assert_eq!(
        notice_path(&db, series).await,
        "/quake-brasil/copa/matches/wb-1-1"
    );
    assert_eq!(notice_path(&db, settings).await, "/quake-brasil/settings");
    assert_eq!(notice_path(&db, home).await, "/quake-brasil");
    assert_eq!(notice_path(&db, lookalike).await, "/quake-br-2/copa");
    // O que a pessoa vê, com o que ela já viu como estava.
    let (_, list) = get(&app, "/me/notifications", &ana).await;
    assert_eq!(
        list["notifications"][0]["notice"]["path"],
        "/quake-brasil/settings"
    );
    assert_eq!(list["notifications"][0]["read"], true);
    let (_, list) = get(&app, "/me/notifications", &bia).await;
    assert_eq!(list["notifications"][1]["notice"]["path"], "/quake-brasil");
    assert_eq!(list["notifications"][1]["read"], false);

    // A comunidade que pega o endereço antigo não recebe nenhum deles.
    let eva = user(&db, "Eva").await;
    create(&app, &eva, "quake-br").await;
    let (_, list) = get(&app, "/me/notifications", &eva).await;
    assert_eq!(list["notifications"], json!([]));
}

#[tokio::test]
async fn the_discord_queue_and_the_pending_notices_follow() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    sqlx::query(
        "insert into discord_webhooks (tenant_id, webhook_id, token, events, mention_players, locale)
         select id, $1, $2, '{match.ready}', false, 'pt' from tenants",
    )
    .bind(WEBHOOK)
    .bind(TOKEN)
    .execute(&db)
    .await
    .unwrap();
    sqlx::query(
        "insert into discord_messages (tenant_id, code, params, path, merge_key)
         select id, 'match.ready', '{}', '/quake-br/copa/matches/wb-1-1',
                'match.ready:/quake-br/copa/matches/wb-1-1' from tenants",
    )
    .execute(&db)
    .await
    .unwrap();
    sqlx::query(
        "insert into discord_messages (tenant_id, code, params, path)
         select id, 'tournament.started', '{}', '/quake-br/copa' from tenants",
    )
    .execute(&db)
    .await
    .unwrap();
    clear(&db).await;
    let notice = json!({
        "kind": "notice",
        "users": [bia.id],
        "notice": {
            "code": "tournament.start_due",
            "params": { "tournament": "Copa" },
            "path": "/quake-br/copa"
        }
    });
    let announce = json!({
        "kind": "announce",
        "announce": {
            "code": "tournament.started",
            "params": { "tournament": "Copa" },
            "path": "/quake-br/copa"
        }
    });
    for event in [&notice, &announce] {
        sqlx::query("insert into outbox (event) values ($1)")
            .bind(event)
            .execute(&db)
            .await
            .unwrap();
    }

    let (status, body) = patch(&app, URI, &ana, move_to("quake-brasil")).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let queue: Vec<(String, Option<String>)> =
        sqlx::query_as("select path, merge_key from discord_messages order by id")
            .fetch_all(&db)
            .await
            .unwrap();
    assert_eq!(
        queue,
        [
            (
                "/quake-brasil/copa/matches/wb-1-1".to_owned(),
                Some("match.ready:/quake-brasil/copa/matches/wb-1-1".to_owned())
            ),
            ("/quake-brasil/copa".to_owned(), None),
        ]
    );
    let events = outbox(&db).await;
    assert_eq!(events[0]["notice"]["path"], "/quake-brasil/copa");
    assert_eq!(events[0]["notice"]["code"], "tournament.start_due");
    assert_eq!(events[1]["announce"]["path"], "/quake-brasil/copa");
}
