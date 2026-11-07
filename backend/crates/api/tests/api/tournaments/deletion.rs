//! Excluir um torneio: quem pode em cada estado, a confirmação digitada, a
//! cascata de verdade com torneio iniciado e encerrado (séries, mapas, veto,
//! draft, escalação, times, screenshots e rating), o que some junto
//! (notificações, avisos pendentes) e que ninguém é avisado.

use api::realtime;
use axum::{
    Router,
    http::{Method, StatusCode},
};
use chrono::Utc;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::*;
use crate::discord::{TOKEN, WEBHOOK};
use crate::matches::{MAPS, confirm, copa, game, play, players, report, single, start, table};
use crate::ratings::ranking;
use crate::realtime::{clear, outbox, tournament};
use crate::screenshots::{FINAL, attach_screenshot, cup, queued};
use crate::teams::{started, tdm};

const URI: &str = "/tenants/quake-br/tournaments/copa";

/// O `DELETE` com o endereço digitado.
async fn delete_typing(app: &Router, who: &User, typed: &str) -> (StatusCode, Value) {
    let body = json!({ "confirm_slug": typed });
    call(app, Method::DELETE, URI, Some(who), Some(body)).await
}

async fn tournament_id(db: &PgPool) -> i64 {
    sqlx::query_scalar("select id from tournaments where slug = 'copa'")
        .fetch_one(db)
        .await
        .unwrap()
}

/// Quantas linhas cada tabela com `tournament_id` tem do torneio `id`.
async fn rows_of(db: &PgPool, id: i64) -> Vec<(String, i64)> {
    let tables: Vec<String> = sqlx::query_scalar(
        "select table_name::text from information_schema.columns
         where table_schema = 'public' and column_name = 'tournament_id'
         order by 1",
    )
    .fetch_all(db)
    .await
    .unwrap();
    let mut rows = Vec::with_capacity(tables.len());
    for table in tables {
        let sql = format!("select count(*) from {table} where tournament_id = $1");
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .bind(id)
            .fetch_one(db)
            .await
            .unwrap();
        rows.push((table, n));
    }
    rows
}

/// As tabelas de `rows` com alguma linha.
fn filled(rows: &[(String, i64)]) -> Vec<&str> {
    rows.iter()
        .filter(|(_, n)| *n > 0)
        .map(|(table, _)| table.as_str())
        .collect()
}

async fn count(db: &PgPool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
}

#[tokio::test]
async fn before_the_start_the_organization_deletes_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;

    // Rascunho: sem o endereço digitado.
    create_tournament(&app, &carla, duel("copa")).await;
    assert_eq!(
        delete(&app, URI, &bia).await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        delete(&app, URI, &carla).await,
        (StatusCode::NO_CONTENT, Value::Null)
    );

    // Publicado: com ele, e quem joga continua de fora.
    for status in ["registration", "check_in"] {
        create_tournament(&app, &carla, duel("copa")).await;
        publish(&app, &carla, "copa").await;
        set_status(&db, "copa", status).await;
        assert_eq!(
            delete_typing(&app, &bia, "copa").await,
            (StatusCode::FORBIDDEN, err("tenant.forbidden"))
        );
        let refused = (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.confirmation_mismatch"),
        );
        assert_eq!(delete(&app, URI, &carla).await, refused, "{status}");
        assert_eq!(delete_typing(&app, &carla, "liga").await, refused);
        assert_eq!(
            delete_typing(&app, &carla, " copa ").await,
            (StatusCode::NO_CONTENT, Value::Null),
            "{status}"
        );
        assert_eq!(
            delete(&app, URI, &carla).await,
            (StatusCode::NOT_FOUND, err("tournament.not_found"))
        );
    }
    // O endereço fica livre, e o dono também exclui.
    create_tournament(&app, &carla, duel("copa")).await;
    publish(&app, &carla, "copa").await;
    assert_eq!(
        delete_typing(&app, &ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
}

#[tokio::test]
async fn after_the_start_only_the_owner_deletes_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let carla = organizer(&app, &db, &ana, "Carla").await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(1, 1)), &[&p[0], &p[1]]).await;
    assert_eq!(
        delete_typing(&app, &carla, "copa").await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        delete(&app, URI, &ana).await,
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            err("tournament.confirmation_mismatch")
        )
    );
    play(&app, &p[0], &p[1], "wb-1-1", 1, 0).await;
    let (_, seen) = get(&app, URI, &ana).await;
    assert_eq!(seen["status"], "finished");
    assert_eq!(
        delete_typing(&app, &carla, "copa").await,
        (StatusCode::FORBIDDEN, err("tenant.forbidden"))
    );
    assert_eq!(
        delete_typing(&app, &ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
}

#[tokio::test]
async fn read_only_with_the_community_being_deleted() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let confirm_slug = json!({ "confirm_slug": "quake-br" });
    post(&app, "/tenants/quake-br/deletion", &ana, confirm_slug).await;
    assert_eq!(
        delete(&app, URI, &ana).await,
        (StatusCode::CONFLICT, err("tenant.deletion_scheduled"))
    );
}

/// Um duelo em andamento, md3, com veto e draft até o primeiro mapa, mapa
/// confirmado e contando para o rating, screenshot e capa pendente. Depois de
/// excluir, não sobra nada dele, as filas levam os arquivos e o ranking fica
/// como se ele nunca tivesse existido.
#[tokio::test]
async fn a_started_duel_goes_with_everything() {
    let (db, _lease) = crate::db::test_db().await;
    let c = cup(
        &db,
        3,
        json!({ "veto": { "map_mode": "veto", "champion_phase": true, "order": "alternating" } }),
    )
    .await;
    let veto = |action: &str| format!("{URI}/matches/{FINAL}/veto/{action}");
    for who in [&c.p1, &c.p2] {
        let (status, body) = post_empty(&c.app, &veto("ready"), who).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    // O veto de mapas e o draft inteiros, até poder enviar o primeiro mapa.
    let steps = [
        (&c.p1, json!({ "map": "awoken" })),
        (&c.p2, json!({ "map": "blood-covenant" })),
        (&c.p1, json!({ "map": "corrupted-keep" })),
        (&c.p2, json!({ "map": "molten-falls" })),
        (&c.p1, json!({ "map": "ruins-of-sarnath" })),
        (&c.p2, json!({ "map": "vale-of-pnath" })),
        (&c.p2, json!({ "champion": "anarki" })),
        (&c.p1, json!({ "champion": "athena" })),
        (&c.p1, json!({ "champion": "clutch" })),
        (&c.p2, json!({ "champion": "clutch" })),
        (&c.p2, json!({ "champion": "eisen" })),
        (&c.p1, json!({ "champion": "galena" })),
        (&c.p1, json!({ "champion": "keel" })),
        (&c.p2, json!({ "champion": "nyx" })),
    ];
    for (who, step) in steps {
        let (status, body) = post(&c.app, &veto("steps"), who, step.clone()).await;
        assert_eq!(status, StatusCode::OK, "{step}: {body}");
    }
    // Um mapa confirmado (que conta para o rating) e um screenshot.
    let (status, shot) = attach_screenshot(&c.app, &c.p1, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{shot}");
    let (status, body) = report(&c.app, &c.p1, FINAL, 1, game("a", "corrupted-keep")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&c.app, &c.p2, FINAL, 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Outro torneio da comunidade, encerrado, que fica no ranking.
    let other = "/tenants/quake-br/tournaments/liga";
    create_tournament(
        &c.app,
        &c.ana,
        with(
            duel("liga"),
            with(
                single(1, 1),
                json!({ "check_in_minutes": null, "screenshots_required": false }),
            ),
        ),
    )
    .await;
    publish(&c.app, &c.ana, "liga").await;
    register(&c.app, "liga", &[&c.bia, &c.p1]).await;
    let (status, seeded) = seed(&c.app, &c.ana, "liga", &[&c.bia, &c.p1]).await;
    assert_eq!(status, StatusCode::OK, "{seeded}");
    let (status, body) = post_empty(&c.app, &format!("{other}/start"), &c.ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let result = format!("{other}/matches/wb-1-1/games/1");
    let map = json!({ "map": MAPS[0], "a_frags": 10, "b_frags": 5 });
    let (status, body) = post(&c.app, &format!("{result}/report"), &c.bia, map).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(&c.app, &format!("{result}/confirm"), &c.p1).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // O relógio do veto já viu o torneio (e guarda o que viu).
    realtime::veto_clock(&c.state, Utc::now()).await.unwrap();

    let id = tournament_id(&db).await;
    let before = rows_of(&db, id).await;
    for table in [
        "match_events",
        "match_game_players",
        "match_games",
        "match_screenshots",
        "match_veto_events",
        "participants",
        "teams",
        "tournament_maps",
        "tournament_stages",
    ] {
        assert!(filled(&before).contains(&table), "{table}: {before:?}");
    }

    let key: String = sqlx::query_scalar("select object_key from match_screenshots")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        delete_typing(&c.app, &c.ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );

    let after = rows_of(&db, id).await;
    assert_eq!(filled(&after), Vec::<&str>::new(), "{after:?}");
    // O outro torneio não perdeu nada, e o arquivo vai para a fila.
    assert_eq!(
        count(&db, "select count(*) from tournaments where slug = 'liga'").await,
        1
    );
    assert_eq!(queued(&db).await, [key]);

    // O agendador, o relógio do veto e o despachante passam sem tropeçar.
    realtime::tick(&c.state, Utc::now())
        .await
        .unwrap()
        .expect("nenhuma outra passada");
    assert_eq!(realtime::veto_clock(&c.state, Utc::now()).await.unwrap(), 0);
    realtime::dispatch_once(&c.state).await.unwrap();
    // O ranking fica só com o outro torneio.
    let rated = ranking(&c.app, &c.ana).await;
    let mut who: Vec<i64> = rated["players"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["user_id"].as_i64().unwrap())
        .collect();
    who.sort_unstable();
    let mut expected = [c.bia.id, c.p1.id];
    expected.sort_unstable();
    assert_eq!(who, expected, "{rated}");
}

/// O rating é refeito sem os mapas do torneio excluído, e também sem contar a
/// escalação deles.
#[tokio::test]
async fn the_ranking_is_recalculated_without_it() {
    let (db, _lease) = crate::db::test_db().await;
    let state = state(db.clone());
    let app = app_from(state.clone());
    let ana = user(&db, "Ana").await;
    create(&app, &ana, "quake-br").await;
    let p = players(&app, &db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    play(&app, &p[0], &p[1], "wb-1-1", 2, 0).await;
    let rated = ranking(&app, &ana).await;
    assert_eq!(rated["players"].as_array().unwrap().len(), 2);
    assert!(count(&db, "select count(*) from rating_history").await > 0);

    assert_eq!(
        delete_typing(&app, &ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    // A marca fica até o recálculo, que o agendador faz (e emite a mudança,
    // para o ranking aberto recarregar).
    assert!(count(&db, "select count(*) from rating_dirty").await > 0);
    clear(&db).await;
    let tick = realtime::tick(&state, Utc::now()).await.unwrap().unwrap();
    assert_eq!(tick.rated, 1);
    assert_eq!(count(&db, "select count(*) from rating_history").await, 0);
    assert_eq!(count(&db, "select count(*) from rating_dirty").await, 0);
    assert_eq!(count(&db, "select count(*) from ratings").await, 0);
    assert_eq!(ranking(&app, &ana).await["players"], json!([]));
}

#[tokio::test]
async fn a_finished_tdm_goes_with_the_lineups_and_the_teams() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    started(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    let (status, body) = report(&app, &p[1], "wb-1-1", 1, game("a", MAPS[0])).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = confirm(&app, &p[3], "wb-1-1", 1).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, seen) = get(&app, URI, &ana).await;
    assert_eq!(seen["status"], "finished", "{seen}");

    let id = tournament_id(&db).await;
    let before = rows_of(&db, id).await;
    for table in [
        "match_game_players",
        "match_games",
        "match_results",
        "teams",
    ] {
        assert!(filled(&before).contains(&table), "{table}: {before:?}");
    }
    assert_eq!(
        delete_typing(&app, &ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    let after = rows_of(&db, id).await;
    assert_eq!(filled(&after), Vec::<&str>::new(), "{after:?}");
    assert_eq!(ranking(&app, &ana).await["players"], json!([]));
}

#[tokio::test]
async fn a_table_goes_with_its_pairings() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 4).await;
    let body = copa(table("swiss", json!({ "best_of": 1 })));
    start(&app, &ana, body, &[&p[0], &p[1], &p[2], &p[3]]).await;
    let id = tournament_id(&db).await;
    assert!(filled(&rows_of(&db, id).await).contains(&"stage_pairings"));
    assert_eq!(
        delete_typing(&app, &ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(filled(&rows_of(&db, id).await), Vec::<&str>::new());
}

/// Quem entrou pelo link de inscrição e quem saiu da comunidade no meio
/// (desistência e W.O. automático) não seguram a exclusão, e o link morre junto.
#[tokio::test]
async fn links_and_withdrawals_do_not_hold_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = players(&app, &db, &ana, 3).await;
    create_tournament(&app, &ana, copa(single(1, 1))).await;
    publish(&app, &ana, "copa").await;
    let (status, link) = get(&app, &format!("{URI}/invite"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{link}");
    let token = link["token"].as_str().unwrap();
    let dani = user(&db, "Dani").await;
    join(&app, &dani, token).await;
    register(&app, "copa", &[&p[0], &p[1], &p[2]]).await;
    let (status, body) = seed(&app, &ana, "copa", &[&dani, &p[0], &p[1], &p[2]]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = post_empty(&app, &format!("{URI}/start"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = delete(&app, "/tenants/quake-br/me", &p[2]).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    let id = tournament_id(&db).await;
    let before = rows_of(&db, id).await;
    for table in ["match_events", "participants", "teams"] {
        assert!(filled(&before).contains(&table), "{table}: {before:?}");
    }
    assert_eq!(
        delete_typing(&app, &ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );
    assert_eq!(filled(&rows_of(&db, id).await), Vec::<&str>::new());
    let eva = user(&db, "Eva").await;
    let (status, gone) = get(&app, &format!("/invites/{token}"), &eva).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{gone}");
    // Quem saiu continua com a linha de inativo, e a comunidade segue.
    assert_eq!(
        count(
            &db,
            "select count(*) from tenant_members where left_at is not null"
        )
        .await,
        1
    );
}

/// Guarda um aviso que leva a `path`.
async fn store_notice(db: &PgPool, who: &User, path: &str, read: bool) {
    sqlx::query(
        "insert into notifications (user_id, notice, read_at)
         values ($1, jsonb_build_object('code', 'tournament.start_due',
                                        'params', jsonb_build_object('tournament', 'Copa'),
                                        'path', $2::text),
                 case when $3 then now() end)",
    )
    .bind(who.id)
    .bind(path)
    .bind(read)
    .execute(db)
    .await
    .unwrap();
}

async fn paths(db: &PgPool, who: &User) -> Vec<String> {
    sqlx::query_scalar("select notice ->> 'path' from notifications where user_id = $1 order by id")
        .bind(who.id)
        .fetch_all(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn nothing_keeps_leading_to_it_and_nobody_is_told() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    create_tournament(&app, &ana, duel("copa-2")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&bia]).await;
    // Com o resto do caminho, sem ele, de outro torneio e um já visto.
    store_notice(&db, &bia, "/quake-br/copa/matches/wb-1-1", false).await;
    store_notice(&db, &bia, "/quake-br/copa", true).await;
    store_notice(&db, &bia, "/quake-br/copa-2", false).await;
    store_notice(&db, &ana, "/quake-br/copa", true).await;
    sqlx::query(
        "insert into discord_webhooks (tenant_id, webhook_id, token, events, mention_players, locale)
         select id, $1, $2, '{match.ready}', false, 'pt' from tenants",
    )
    .bind(WEBHOOK)
    .bind(TOKEN)
    .execute(&db)
    .await
    .unwrap();
    for (path, code) in [
        ("/quake-br/copa/matches/wb-1-1", "match.ready"),
        ("/quake-br/copa-2", "tournament.started"),
    ] {
        sqlx::query(
            "insert into discord_messages (tenant_id, code, params, path)
             select id, $1, '{}', $2 from tenants",
        )
        .bind(code)
        .bind(path)
        .execute(&db)
        .await
        .unwrap();
    }
    clear(&db).await;
    let pending = json!({
        "kind": "notice",
        "users": [bia.id],
        "notice": {
            "code": "tournament.start_due",
            "params": { "tournament": "Copa" },
            "path": "/quake-br/copa"
        }
    });
    sqlx::query("insert into outbox (event) values ($1)")
        .bind(&pending)
        .execute(&db)
        .await
        .unwrap();

    assert_eq!(
        delete_typing(&app, &ana, "copa").await,
        (StatusCode::NO_CONTENT, Value::Null)
    );

    assert_eq!(paths(&db, &bia).await, ["/quake-br/copa-2"]);
    assert_eq!(paths(&db, &ana).await, Vec::<String>::new());
    let queue: Vec<String> = sqlx::query_scalar("select path from discord_messages")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(queue, ["/quake-br/copa-2"]);

    // O que sai é só "recarregue", e a contagem de quem perdeu um não visto.
    let events = outbox(&db).await;
    assert!(events.contains(&tournament("copa", None)), "{events:?}");
    assert!(
        events.contains(&json!({ "kind": "unread", "user": bia.id, "count": 1 })),
        "{events:?}"
    );
    assert!(
        events
            .iter()
            .all(|e| e["kind"] != "notice" && e["kind"] != "announce"),
        "{events:?}"
    );
    // Quem só tinha visto não muda de contagem.
    assert!(
        !events
            .iter()
            .any(|e| e["kind"] == "unread" && e["user"] == ana.id),
        "{events:?}"
    );
}
