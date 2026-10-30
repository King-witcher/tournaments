//! O envio: do aviso no outbox à mensagem no canal, com o texto de cada
//! aviso, as menções, o ritmo do Discord e o webhook que deixa de existir.

use std::time::Duration;

use api::{discord, realtime};
use axum::http::StatusCode;
use chrono::{DateTime, TimeDelta, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{COPA, copa, play, players, single, start};

/// O despachante passa o outbox para a fila e a fila sai, em `now`.
async fn pump(s: &Setup, now: DateTime<Utc>) -> usize {
    realtime::dispatch_once(&s.state).await.unwrap();
    // As mensagens nascem com o relógio do banco, depois do `now` do teste.
    discord::deliver_due(&s.state, now.max(Utc::now()))
        .await
        .unwrap()
}

async fn queued(db: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from discord_messages")
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_ready_match_mentions_the_players_who_linked_discord() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.ready"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    link_discord(&db, &p[0], "111222333").await;
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;

    // Um aviso por lado, uma mensagem só, que nomeia quem não vinculou.
    assert_eq!(pump(&s, Utc::now()).await, 1);
    let hits = s.fake.hits();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].path, format!("/api/webhooks/{WEBHOOK}/{TOKEN}"));
    assert_eq!(
        hits[0].body,
        json!({
            "content": "SÃ©rie pronta em **Copa de Outubro** (wb-1-1): <@111222333> vs P2\nhttp://localhost:4001/pt/quake-br/copa/matches/wb-1-1",
            "allowed_mentions": { "parse": [], "users": ["111222333"] }
        })
    );
    assert_eq!(queued(&db).await, 0);
}

#[tokio::test]
async fn mentions_can_be_turned_off() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.ready"]).await;
    let off = json!({ "events": ["match.ready"], "locale": "pt", "mention_players": false });
    put(&s.app, URI, &s.ana, off).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    link_discord(&db, &p[0], "111222333").await;
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;

    pump(&s, Utc::now()).await;
    let hit = &s.fake.hits()[0];
    assert!(
        hit.body["content"]
            .as_str()
            .unwrap()
            .contains(": P1 vs P2\n")
    );
    assert_eq!(
        hit.body["allowed_mentions"],
        json!({ "parse": [], "users": [] })
    );
}

#[tokio::test]
async fn a_notice_that_is_off_is_not_sent() {
    let (db, _lease) = crate::db::test_db().await;
    // Só o check-in está ligado; o início e as partidas prontas saem do outbox
    // sem virar mensagem.
    let s = configured(&db, &["check_in.opened"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    assert_eq!(pump(&s, Utc::now()).await, 0);
    assert!(s.fake.hits().is_empty());
    assert_eq!(queued(&db).await, 0);
}

#[tokio::test]
async fn a_community_without_webhook_queues_nothing() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.ready"]).await;
    delete(&s.app, URI, &s.ana).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    assert_eq!(pump(&s, Utc::now()).await, 0);
    assert!(s.fake.hits().is_empty());
}

#[tokio::test]
async fn check_in_and_start() {
    let (db, _lease) = crate::db::test_db().await;
    let all = [
        "check_in.opened",
        "tournament.started",
        "tournament.finished",
        "match.series_decided",
    ];
    let s = configured(&db, &all).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    create_tournament(&s.app, &s.ana, duel("copa")).await;
    publish(&s.app, &s.ana, "copa").await;
    register(&s.app, "copa", &[&p[0], &p[1]]).await;
    let (status, body) = post_empty(&s.app, &format!("{COPA}/check-in/open"), &s.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    pump(&s, Utc::now()).await;
    assert_eq!(
        s.fake.texts(),
        [
            "O check-in de **Copa de Outubro** abriu. Faça o seu para jogar.\nhttp://localhost:4001/pt/quake-br/copa"
        ]
    );

    // O check-in de cada um e o início. A chave de duelo é dupla com reset;
    // basta jogar a série até a final sair.
    for who in &p {
        let uri = format!("{COPA}/check-in");
        assert_eq!(
            post_empty(&s.app, &uri, who).await.0,
            StatusCode::NO_CONTENT
        );
    }
    let (status, body) = post_empty(&s.app, &format!("{COPA}/start"), &s.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    pump(&s, Utc::now()).await;
    assert_eq!(
        s.fake.texts()[1],
        "**Copa de Outubro** começou. A chave está aqui: http://localhost:4001/pt/quake-br/copa"
    );
}

#[tokio::test]
async fn the_series_and_the_podium_of_a_finished_tournament() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["tournament.finished", "match.series_decided"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    start(&s.app, &s.ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    // P2 vence por 2 a 1.
    play(&s.app, &p[1], &p[0], "wb-1-1", 1, 2).await;
    pump(&s, Utc::now()).await;
    assert_eq!(
        s.fake.texts(),
        [
            "**Copa de Outubro**, sÃ©rie wb-1-1: P2 venceu P1 (2-1).\nhttp://localhost:4001/pt/quake-br/copa/matches/wb-1-1",
            "**Copa de Outubro** terminou. Pódio:\n1. P2\n2. P1\nhttp://localhost:4001/pt/quake-br/copa",
        ]
    );
}

#[tokio::test]
async fn the_series_result_is_off_by_default() {
    let (db, _lease) = crate::db::test_db().await;
    let defaults = ["tournament.finished", "tournament.started"];
    let s = configured(&db, &defaults).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    play(&s.app, &p[0], &p[1], "wb-1-1", 1, 0).await;
    pump(&s, Utc::now()).await;
    let texts = s.fake.texts();
    assert_eq!(texts.len(), 2, "{texts:?}");
    assert!(texts.iter().all(|t| !t.contains("venceu")));
}

#[tokio::test]
async fn the_organization_is_mentioned_on_a_contested_map() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.game_contested"]).await;
    let olga = organizer(&s.app, &db, &s.ana, "Olga").await;
    link_discord(&db, &olga, "999").await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    start(&s.app, &s.ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    let (status, body) = crate::matches::report(
        &s.app,
        &p[1],
        "wb-1-1",
        1,
        crate::matches::game("b", "awoken"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = crate::matches::contest(&s.app, &p[0], "wb-1-1", 1, "foi meu").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    pump(&s, Utc::now()).await;
    let hit = &s.fake.hits()[0];
    assert_eq!(
        hit.body["content"],
        "<@999> P1 contestou a partida 1 da sÃ©rie wb-1-1 de **Copa de Outubro**. A organização precisa resolver.\nhttp://localhost:4001/pt/quake-br/copa/matches/wb-1-1"
    );
    assert_eq!(hit.body["allowed_mentions"]["users"], json!(["999"]));
}

#[tokio::test]
async fn the_text_follows_the_language_and_the_translated_name() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["tournament.started"]).await;
    let es = json!({ "events": ["tournament.started"], "locale": "es", "mention_players": true });
    put(&s.app, URI, &s.ana, es).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    let translated =
        json!({ "locale": "es", "name": "Copa *de* Octubre", "description": "", "rules": "" });
    let body = with(copa(single(1, 1)), json!({ "translations": [translated] }));
    start(&s.app, &s.ana, body, &[&p[0], &p[1]]).await;
    pump(&s, Utc::now()).await;
    // O nome da tradução, sem deixar a formatação do organizador valer.
    assert_eq!(
        s.fake.texts(),
        [
            "**Copa \\*de\\* Octubre** empezó. La llave está aquí: http://localhost:4001/es/quake-br/copa"
        ]
    );
}

#[tokio::test]
async fn a_rate_limit_holds_the_webhook_and_the_message_goes_later() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["tournament.started"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    s.fake.reply(&[Reply::TooMany(5.0)]);
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    let now = Utc::now();

    // 429: a mensagem fica, e o webhook espera.
    assert_eq!(pump(&s, now).await, 1);
    assert_eq!(s.fake.hits().len(), 1);
    assert_eq!(queued(&db).await, 1);
    // Antes do prazo, nem tenta.
    assert_eq!(
        discord::deliver_due(&s.state, now + TimeDelta::seconds(4))
            .await
            .unwrap(),
        0
    );
    assert_eq!(s.fake.hits().len(), 1);
    // Passado o prazo, sai. O 429 não gastou uma tentativa.
    let later = now + TimeDelta::seconds(6);
    assert_eq!(discord::deliver_due(&s.state, later).await.unwrap(), 1);
    assert_eq!(s.fake.hits().len(), 2);
    assert_eq!(queued(&db).await, 0);
}

#[tokio::test]
async fn a_spent_window_makes_the_next_message_wait() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["check_in.opened", "tournament.started"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    create_tournament(&s.app, &s.ana, duel("copa")).await;
    publish(&s.app, &s.ana, "copa").await;
    register(&s.app, "copa", &[&p[0], &p[1]]).await;
    post_empty(&s.app, &format!("{COPA}/check-in/open"), &s.ana).await;
    for who in &p {
        post_empty(&s.app, &format!("{COPA}/check-in"), who).await;
    }
    post_empty(&s.app, &format!("{COPA}/start"), &s.ana).await;
    s.fake.reply(&[Reply::Spent(2.0)]);
    let now = Utc::now();

    // As duas mensagens estão na fila; a primeira esgota a janela.
    assert_eq!(pump(&s, now).await, 1);
    assert_eq!(s.fake.hits().len(), 1);
    assert_eq!(queued(&db).await, 1);
    assert_eq!(
        discord::deliver_due(&s.state, now + TimeDelta::seconds(3))
            .await
            .unwrap(),
        1
    );
    assert_eq!(s.fake.hits().len(), 2);
}

#[tokio::test]
async fn a_webhook_held_back_does_not_block_other_communities() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["tournament.started"]).await;
    // A segunda comunidade, com o webhook em espera.
    let bia = user(&db, "Bia").await;
    create(&s.app, &bia, "outra").await;
    let other = with(config(&["tournament.started"]), json!({}));
    let uri = "/tenants/outra/discord";
    assert_eq!(put(&s.app, uri, &bia, other).await.0, StatusCode::OK);
    sqlx::query(
        "update discord_webhooks set retry_at = now() + interval '1 hour' \
         where tenant_id = (select id from tenants where slug = 'outra')",
    )
    .execute(&db)
    .await
    .unwrap();
    sqlx::query(
        "insert into discord_messages (tenant_id, code, params, path) \
         select id, 'tournament.started', '{\"tournament\": \"Taça\"}', '/outra/taca' \
         from tenants where slug = 'outra'",
    )
    .execute(&db)
    .await
    .unwrap();

    let p = players(&s.app, &db, &s.ana, 2).await;
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    assert_eq!(pump(&s, Utc::now()).await, 1);
    assert_eq!(s.fake.hits().len(), 1);
    assert_eq!(queued(&db).await, 1);
}

#[tokio::test]
async fn a_deleted_webhook_breaks_the_configuration_and_stops_the_attempts() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["tournament.started", "tournament.finished"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    s.fake.reply(&[Reply::Status(404)]);
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    assert_eq!(pump(&s, Utc::now()).await, 1);

    let (_, body) = get(&s.app, URI, &s.ana).await;
    assert_eq!(body["webhook"]["broken"], true);
    assert_eq!(queued(&db).await, 0);

    // Novos avisos não entram na fila nem chegam ao Discord.
    play(&s.app, &p[0], &p[1], "wb-1-1", 1, 0).await;
    assert_eq!(pump(&s, Utc::now()).await, 0);
    assert_eq!(s.fake.hits().len(), 1);
    assert_eq!(queued(&db).await, 0);

    // Outra URL volta a funcionar.
    put(&s.app, URI, &s.ana, config(&["tournament.started"])).await;
    let (_, body) = get(&s.app, URI, &s.ana).await;
    assert_eq!(body["webhook"]["broken"], false);
}

#[tokio::test]
async fn other_failures_are_retried_a_few_times_and_then_dropped() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["tournament.started"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    s.fake.reply(&[Reply::Status(500); 5]);
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    realtime::dispatch_once(&s.state).await.unwrap();
    let mut now = Utc::now();
    for attempt in 1..=5 {
        assert_eq!(
            discord::deliver_due(&s.state, now).await.unwrap(),
            1,
            "{attempt}"
        );
        // Entre uma tentativa e outra, espera (o dobro a cada vez).
        let waiting = discord::deliver_due(&s.state, now + TimeDelta::seconds(5)).await;
        assert_eq!(waiting.unwrap(), 0);
        now += TimeDelta::minutes(10);
    }
    assert_eq!(s.fake.hits().len(), 5);
    assert_eq!(queued(&db).await, 0);
    // Nada disso quebrou a configuração.
    assert_eq!(get(&s.app, URI, &s.ana).await.1["webhook"]["broken"], false);
}

#[tokio::test]
async fn old_messages_are_dropped_unsent() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["tournament.started"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    start(&s.app, &s.ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    realtime::dispatch_once(&s.state).await.unwrap();
    let now = Utc::now() + TimeDelta::from_std(Duration::from_secs(2 * 3600)).unwrap();
    assert_eq!(discord::deliver_due(&s.state, now).await.unwrap(), 0);
    assert!(s.fake.hits().is_empty());
    assert_eq!(queued(&db).await, 0);
}

#[tokio::test]
async fn the_address_never_shows_up_in_the_api_surface() {
    let (db, _lease) = crate::db::test_db().await;
    // Nem em erro: a credencial só existe no banco e na requisição ao Discord.
    let s = configured(&db, &["tournament.started"]).await;
    let (_, openapi) = call(&s.app, axum::http::Method::GET, "/openapi.json", None, None).await;
    assert!(!openapi.to_string().contains(TOKEN));
    let bad = with(
        config(&[]),
        json!({ "url": format!("https://x.com/api/webhooks/1/{TOKEN}") }),
    );
    let (_, body) = put(&s.app, URI, &s.ana, bad).await;
    assert!(!body.to_string().contains(TOKEN));
}

#[tokio::test]
async fn the_veto_notices_reach_the_channel() {
    let (db, _lease) = crate::db::test_db().await;
    let s = configured(&db, &["match.opponent_ready", "match.veto_started"]).await;
    let p = players(&s.app, &db, &s.ana, 2).await;
    link_discord(&db, &p[1], "444555666").await;
    let veto = json!({ "veto": { "map_mode": "veto", "champion_phase": false } });
    start(
        &s.app,
        &s.ana,
        copa(with(single(1, 1), veto)),
        &[&p[0], &p[1]],
    )
    .await;
    assert_eq!(pump(&s, Utc::now()).await, 0);

    // P1 fica pronto: o canal chama quem falta.
    let ready = format!("{COPA}/matches/wb-1-1/veto/ready");
    assert_eq!(post_empty(&s.app, &ready, &p[0]).await.0, StatusCode::OK);
    assert_eq!(pump(&s, Utc::now()).await, 1);
    // P2 também: o veto começa, uma mensagem para os dois lados.
    assert_eq!(post_empty(&s.app, &ready, &p[1]).await.0, StatusCode::OK);
    assert_eq!(pump(&s, Utc::now()).await, 1);
    let link = "http://localhost:4001/pt/quake-br/copa/matches/wb-1-1";
    assert_eq!(
        s.fake.texts(),
        [
            format!(
                "<@444555666>: P1 está pronto para o veto da sÃ©rie wb-1-1 de **Copa de Outubro** e espera você.\n{link}"
            ),
            format!(
                "O veto da sÃ©rie wb-1-1 de **Copa de Outubro** começou: P1 vs <@444555666>\n{link}"
            ),
        ]
    );
}
