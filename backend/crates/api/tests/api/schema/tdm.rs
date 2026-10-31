//! TDM: modo e tamanho do time andam juntos e congelam no início; o time não
//! passa do tamanho; o nome é único no torneio; trocar o modo refaz os
//! times; depois do início só o substituto entra (sem contar nas vagas); a
//! escolha simultânea de campeões é de um jogador do time do lado.

use super::*;
use crate::teams::{formed, started, tdm};

async fn ids(db: &PgPool) -> (i64, i64) {
    sqlx::query_as("select tenant_id, id from tournaments")
        .fetch_one(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn mode_and_team_size_go_together() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    for (sql, name) in [
        (
            "update tournaments set mode = 'tdm'",
            "tournaments_mode_team_size",
        ),
        (
            "update tournaments set team_size = 2",
            "tournaments_mode_team_size",
        ),
        (
            "update tournaments set mode = 'ffa', team_size = 2",
            "tournaments_mode_check",
        ),
        (
            "update tournaments set mode = 'tdm', team_size = 5",
            "tournaments_team_size_check",
        ),
        (
            "update tournaments set team_pick_seconds = 10",
            "tournaments_team_pick_seconds_check",
        ),
        (
            r#"update tournaments set champion_rules = '{"bans_per_series": 1, "mirror": true, "same_team": 1, "no_repeat": null}'"#,
            "tournaments_champion_rules_shape",
        ),
    ] {
        assert_eq!(rejected(exec(&db, sql).await), rule(CHECK, name), "{sql}");
    }
}

#[tokio::test]
async fn switching_the_mode_redoes_the_teams_in_the_database() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia]).await;
    exec(&db, "update tournaments set mode = 'tdm', team_size = 2")
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from teams").await, 0);
    assert_eq!(
        count(
            &db,
            "select count(*) from participants where team_id is null"
        )
        .await,
        2
    );
    exec(&db, "update tournaments set mode = 'duel', team_size = 1")
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from teams").await, 2);
}

#[tokio::test]
async fn a_team_holds_at_most_its_size_and_a_unique_name() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = crate::matches::players(&app, &db, &ana, 5).await;
    let teams = formed(
        &app,
        &ana,
        tdm(2, json!({})),
        2,
        &[&p[0], &p[1], &p[2], &p[3]],
    )
    .await;
    register(&app, "copa", &[&p[4]]).await;
    let third = format!(
        "update participants set team_id = {} where user_id = {}",
        teams[0], p[4].id
    );
    assert_eq!(
        rejected(exec(&db, &third).await),
        rule(CHECK, "teams_tdm_size")
    );
    exec(
        &db,
        &format!("update teams set name = 'Os Bons' where id = {}", teams[0]),
    )
    .await
    .unwrap();
    let same = format!("update teams set name = 'OS BONS' where id = {}", teams[1]);
    assert_eq!(
        rejected(exec(&db, &same).await),
        rule(UNIQUE, "teams_name_key")
    );

    // O capitão que cancela passa o time adiante (sem levar o time junto).
    exec(
        &db,
        &format!("delete from participants where user_id = {}", p[0].id),
    )
    .await
    .unwrap();
    let sql = format!("select captain from teams where id = {}", teams[0]);
    let captain: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(captain, p[1].id);
}

#[tokio::test]
async fn after_the_start_only_a_substitute_comes_in() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let p = crate::matches::players(&app, &db, &ana, 4).await;
    let body = tdm(2, json!({ "max_participants": 4 }));
    let teams = started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    let (tenant, tournament) = ids(&db).await;

    assert_eq!(
        rejected(exec(&db, "update tournaments set team_size = 3").await),
        rule(CHECK, "tournaments_mode_started_frozen")
    );
    assert_eq!(
        rejected(exec(&db, "update tournaments set team_pick_seconds = 90").await),
        rule(CHECK, "tournaments_veto_started_frozen")
    );
    // Sem time, não entra.
    let loose = format!(
        "insert into participants (tenant_id, tournament_id, user_id)
         values ({tenant}, {tournament}, {})",
        bia.id
    );
    assert_eq!(
        rejected(exec(&db, &loose).await),
        rule(CHECK, "participants_started_frozen")
    );
    // No time, entra, mesmo com as vagas cheias (o time tem um a mais no
    // commit: sai alguém na mesma transação).
    let mut tx = db.begin().await.unwrap();
    for sql in [
        format!(
            "insert into participants (tenant_id, tournament_id, user_id, team_id)
             values ({tenant}, {tournament}, {}, {})",
            bia.id, teams[0]
        ),
        format!(
            "update participants set team_id = null where user_id = {}",
            p[1].id
        ),
    ] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    assert_eq!(count(&db, "select count(*) from participants").await, 5);
}

#[tokio::test]
async fn a_team_pick_is_of_a_player_of_that_side() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let p = crate::matches::players(&app, &db, &ana, 4).await;
    let body = tdm(
        2,
        json!({ "veto": { "map_mode": "off", "champion_phase": true } }),
    );
    let teams = started(&app, &ana, body, 2, &[&p[0], &p[1], &p[2], &p[3]]).await;
    let uri = format!("{COPA}/matches/wb-1-1/veto/ready");
    post_empty(&app, &uri, &p[0]).await;
    let (status, body) = post_empty(&app, &uri, &p[2]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (tenant, tournament) = ids(&db).await;
    let anarki: i64 = sqlx::query_scalar("select id from champions where slug = 'anarki'")
        .fetch_one(&db)
        .await
        .unwrap();
    let event = |side: &str, player: &str, kind: &str, pick_game: &str, gave_way: bool| {
        format!(
            "insert into match_veto_events (tenant_id, tournament_id, stage, match_key, side_a,
                 side_b, kind, game, side, champion_id, player_id, pick_game, gave_way)
             values ({tenant}, {tournament}, 1, 'wb-1-1', {}, {}, '{kind}', 1, '{side}', {anarki},
                     {player}, {pick_game}, {gave_way})",
            teams[0], teams[1]
        )
    };
    let pick = |side: &str, player: i64, kind: &str| {
        let pick_game = if kind == "pick" { "1" } else { "null" };
        event(side, &player.to_string(), kind, pick_game, false)
    };
    // P3 é do lado b.
    assert_eq!(
        rejected(exec(&db, &pick("a", p[2].id, "pick")).await),
        rule(CHECK, "match_veto_events_player_team")
    );
    // Só pick de campeão leva jogador.
    assert_eq!(
        rejected(exec(&db, &pick("a", p[1].id, "ban")).await),
        rule(CHECK, "match_veto_events_player")
    );
    // O pick de campeão diz a partida a que se destina, a em andamento ou
    // uma seguinte; e só o da janela simultânea diz se cedeu.
    for (pick_game, gave_way, player) in [("null", false, "null"), ("0", false, "null")] {
        assert_eq!(
            rejected(exec(&db, &event("a", player, "pick", pick_game, gave_way)).await),
            rule(CHECK, "match_veto_events_pick_game")
        );
    }
    assert_eq!(
        rejected(exec(&db, &event("a", "null", "pick", "2", true)).await),
        rule(CHECK, "match_veto_events_gave_way")
    );
    exec(&db, &pick("a", p[1].id, "pick")).await.unwrap();
}
