//! As fases ligadas no banco (`tournament_stages`, `stage_teams`): corte e
//! check-in só da fase 2 em diante, a trava por fase depois do início do
//! torneio, as fases em sequência com a eliminação só na última, a fase que
//! só começa com a anterior terminada, e os times de cada fase (presença com
//! o check-in aberto, seeds de 1 a K gravados antes do início e congelados
//! depois, só quem jogou a anterior), que saem com a fase ou com o torneio.

use super::*;
use crate::common::*;
use crate::matches::{copa, start};

/// Todos contra todos de 4 em md1 iniciado, com a disputa de posições depois.
/// Devolve o tenant, o torneio e os times em ordem de seed.
async fn started(db: &PgPool) -> (Router, User, i64, i64, Vec<i64>) {
    let (app, ana, _) = quake_br(db).await;
    let p = crate::matches::players(&app, db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    let stages = json!({ "stages": [
        { "format": "round_robin", "best_of": 1 },
        { "format": "placement", "best_of": 1, "check_in": true }
    ] });
    start(&app, &ana, copa(stages), &all).await;
    let teams = team_ids(&app, &ana, "copa", &all).await;
    let t = sqlx::query!("select tenant_id, id from tournaments where slug = 'copa'")
        .fetch_one(db)
        .await
        .unwrap();
    (app, ana, t.tenant_id, t.id, teams)
}

fn stage_team(tenant: i64, t: i64, stage: i16, team: i64, seed: Option<i16>) -> String {
    let seed = seed.map_or("null".to_owned(), |s| s.to_string());
    let checked_in = if seed == "null" { "now()" } else { "null" };
    format!(
        "insert into stage_teams (tenant_id, tournament_id, stage, team_id, checked_in_at, seed)
         values ({tenant}, {t}, {stage}, {team}, {checked_in}, {seed})"
    )
}

#[tokio::test]
async fn the_stage_columns_follow_the_position() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, _, _, _, _) = started(&db).await;
    let set = |sql: &str| format!("update tournament_stages set {sql} where position = 2");
    assert_eq!(
        rejected(
            exec(
                &db,
                "update tournament_stages set advance = 4 where position = 1"
            )
            .await
        ),
        rule(CHECK, "tournament_stages_started_frozen")
    );
    assert_eq!(
        rejected(exec(&db, &set("advance = 1")).await),
        rule(CHECK, "tournament_stages_advance")
    );
    assert_eq!(
        rejected(exec(&db, &set("check_in = false, check_in_opened_at = now()")).await),
        rule(CHECK, "tournament_stages_check_in_opened")
    );
    assert_eq!(
        rejected(exec(&db, &set("finished_at = now()")).await),
        rule(CHECK, "tournament_stages_finished_started")
    );
    assert_eq!(
        rejected(exec(&db, &set("position = 5")).await),
        rule(CHECK, "tournament_stages_position_max")
    );
    // A fase que não começou muda e sai depois do início; fase nova não entra.
    exec(&db, &set("best_of = 3, advance = 2")).await.unwrap();
    assert_eq!(
        rejected(
            exec(
                &db,
                "insert into tournament_stages (tournament_id, position, format, best_of)
                 select tournament_id, 3, 'placement', 1 from tournament_stages where position = 2"
            )
            .await
        ),
        rule(CHECK, "tournament_stages_started_frozen")
    );
    exec(&db, "delete from tournament_stages where position = 2")
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, "delete from tournament_stages where position = 1").await),
        rule(FOREIGN_KEY, "tournament_stages_started_delete")
    );
}

#[tokio::test]
async fn stages_go_in_sequence_with_elimination_last() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    let stages = json!({ "stages": [
        { "format": "round_robin", "best_of": 1 },
        { "format": "placement", "best_of": 1 }
    ] });
    create_tournament(&app, &ana, copa(stages)).await;
    // Rascunho: tudo muda, mas no commit as fases são 1..n e a eliminação fica
    // por último.
    assert_eq!(
        rejected(
            exec(
                &db,
                "update tournament_stages set position = 3 where position = 2"
            )
            .await
        ),
        rule(CHECK, "tournament_stages_sequence")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                "update tournament_stages
                 set format = 'elimination', bracket = 'single', finals_best_of = 1,
                     tiebreakers = null, final_tie = null
                 where position = 1"
            )
            .await
        ),
        rule(CHECK, "tournament_stages_elimination_last")
    );
}

#[tokio::test]
async fn a_stage_starts_after_the_previous_one_with_its_seeds() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, _, tenant, t, teams) = started(&db).await;
    // A fase 1 não terminou: a 2 não começa, nem com os seeds.
    // (A atualização do CTE, sem quem a leia, roda depois do insert.)
    let start_with_seeds = format!(
        "with u as (update tournament_stages set started_at = now() where position = 2)
         insert into stage_teams (tenant_id, tournament_id, stage, team_id, seed)
         values ({tenant}, {t}, 2, {a}, 1), ({tenant}, {t}, 2, {b}, 2)",
        a = teams[0],
        b = teams[1]
    );
    assert_eq!(
        rejected(exec(&db, &start_with_seeds).await),
        rule(CHECK, "tournament_stages_previous_finished")
    );
    exec(
        &db,
        "update tournament_stages set finished_at = now() where position = 1",
    )
    .await
    .unwrap();
    // Presença só com o check-in aberto; só quem jogou a anterior.
    assert_eq!(
        rejected(exec(&db, &stage_team(tenant, t, 2, teams[0], None)).await),
        rule(CHECK, "stage_teams_check_in_closed")
    );
    exec(
        &db,
        "update tournament_stages set check_in_opened_at = now() where position = 2",
    )
    .await
    .unwrap();
    exec(&db, &stage_team(tenant, t, 2, teams[0], None))
        .await
        .unwrap();
    assert_eq!(
        rejected(
            exec(
                &db,
                &format!(
                    "insert into stage_teams (tenant_id, tournament_id, stage, team_id, seed)
                     values ({tenant}, {t}, 2, {}, null)",
                    teams[1]
                )
            )
            .await
        ),
        rule(CHECK, "stage_teams_present")
    );
    // Começada sem seeds, ou com um só: recusada no commit.
    assert_eq!(
        rejected(
            exec(
                &db,
                "update tournament_stages set started_at = now() where position = 2"
            )
            .await
        ),
        rule(CHECK, "stage_teams_seeds")
    );
    // Os seeds antes do início, 1..K, e o início junto.
    let mut tx = db.begin().await.unwrap();
    for sql in [
        format!(
            "update stage_teams set seed = 1 where team_id = {}",
            teams[0]
        ),
        stage_team(tenant, t, 2, teams[1], Some(2)),
        "update tournament_stages set started_at = now() where position = 2".to_owned(),
    ] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    // Começada: os seeds e as presenças não mudam, e ninguém sai.
    assert_eq!(
        rejected(exec(&db, "update stage_teams set seed = 3 where seed = 2").await),
        rule(CHECK, "stage_teams_seed_frozen")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                "update stage_teams set checked_in_at = null where seed = 1"
            )
            .await
        ),
        rule(CHECK, "stage_teams_check_in_closed")
    );
    assert_eq!(
        rejected(exec(&db, "delete from stage_teams where seed = 2").await),
        rule(FOREIGN_KEY, "stage_teams_started_delete")
    );
    assert_eq!(count(&db, "select count(*) from stage_teams").await, 2);
    // O torneio apagado leva tudo.
    exec(&db, &format!("delete from tournaments where id = {t}"))
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from stage_teams").await, 0);
}
