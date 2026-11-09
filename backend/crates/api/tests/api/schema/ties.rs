//! Os empates do fim da fase (`stage_tie_decisions`, `stage_pending_ties`):
//! `final_tie` só nas fases de tabela, decisão só na fase com a organização
//! decidindo e de torneio iniciado, com a ordem de 1 a n e sem mudar depois,
//! marca de espera só com o torneio em andamento, e tudo sai ao desfazer o
//! início ou com o torneio.

use super::*;
use crate::common::*;
use crate::matches::{copa, start, table};

/// Todos contra todos de 3 iniciado com a organização decidindo o empate do
/// fim. Devolve o app, a owner, o tenant, o torneio e os times em ordem de
/// seed.
async fn started(db: &PgPool, final_tie: &str) -> (Router, User, i64, i64, Vec<i64>) {
    let (app, ana, _) = quake_br(db).await;
    let p = crate::matches::players(&app, db, &ana, 3).await;
    let all: Vec<&User> = p.iter().collect();
    let fields = json!({ "best_of": 1, "tiebreakers": [], "final_tie": final_tie });
    start(&app, &ana, copa(table("round_robin", fields)), &all).await;
    let teams = team_ids(&app, &ana, "copa", &all).await;
    let t = sqlx::query!("select tenant_id, id from tournaments where slug = 'copa'")
        .fetch_one(db)
        .await
        .unwrap();
    (app, ana, t.tenant_id, t.id, teams)
}

/// Uma decisão de `ana` com os times `teams` nas posições dadas.
fn decision(tenant: i64, t: i64, ana: i64, teams: &[(i16, i64)]) -> String {
    let values: Vec<String> = teams
        .iter()
        .map(|(n, team)| format!("({tenant}, {t}, (select id from d), {n}, {team})"))
        .collect();
    format!(
        "with d as (insert into stage_tie_decisions (tenant_id, tournament_id, stage, decided_by)
                    values ({tenant}, {t}, 1, {ana}) returning id)
         insert into stage_tie_decision_teams (tenant_id, tournament_id, decision_id, position, team_id)
         values {}",
        values.join(", ")
    )
}

#[tokio::test]
async fn decisions_need_the_organizer_mode_and_a_full_order() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, ana, tenant, t, teams) = started(&db, "organizer").await;
    let [a, b, c] = [teams[0], teams[1], teams[2]];

    exec(&db, &decision(tenant, t, ana.id, &[(1, c), (2, a), (3, b)]))
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, "update stage_tie_decisions set reason = 'x'").await),
        rule(CHECK, "stage_tie_decisions_immutable")
    );
    assert_eq!(
        rejected(exec(&db, "update stage_tie_decision_teams set position = 9").await),
        rule(CHECK, "stage_tie_decisions_immutable")
    );
    // Um time só, posição pulada, time repetido.
    for rows in [vec![(1, a)], vec![(1, a), (3, b)]] {
        assert_eq!(
            rejected(exec(&db, &decision(tenant, t, ana.id, &rows)).await),
            rule(CHECK, "stage_tie_decisions_order"),
            "{rows:?}"
        );
    }
    assert_eq!(
        rejected(exec(&db, &decision(tenant, t, ana.id, &[(1, a), (2, a)])).await),
        rule(UNIQUE, "stage_tie_decision_teams_team_once")
    );
    let long = format!(
        "insert into stage_tie_decisions (tenant_id, tournament_id, stage, reason)
         values ({tenant}, {t}, 1, '{}')",
        "x".repeat(281)
    );
    assert_eq!(
        rejected(exec(&db, &long).await),
        rule(CHECK, "stage_tie_decisions_reason")
    );

    // Marca de espera só com o torneio em andamento.
    let mark = format!(
        "insert into stage_pending_ties (tenant_id, tournament_id, stage) values ({tenant}, {t}, 1)"
    );
    exec(&db, &mark).await.unwrap();
    let finish = format!("update tournaments set status = 'finished' where id = {t}");
    assert_eq!(
        rejected(exec(&db, &finish).await),
        rule(CHECK, "stage_pending_ties_in_progress")
    );

    // Com o torneio, sai tudo.
    exec(&db, &format!("delete from tournaments where id = {t}"))
        .await
        .unwrap();
    for table in [
        "stage_tie_decisions",
        "stage_tie_decision_teams",
        "stage_pending_ties",
    ] {
        assert_eq!(
            count(&db, &format!("select count(*) from {table}")).await,
            0,
            "{table}"
        );
    }
}

/// Apagar a comunidade apaga os membros, e o `set null` de `decided_by` é um
/// UPDATE que a guarda de "não muda" precisa deixar passar.
#[tokio::test]
async fn deleting_the_tenant_takes_a_decision_that_still_has_its_author() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, ana, tenant, t, teams) = started(&db, "organizer").await;
    let rows = [(1, teams[1]), (2, teams[0])];
    exec(&db, &decision(tenant, t, ana.id, &rows))
        .await
        .unwrap();

    exec(&db, &format!("delete from tenants where id = {tenant}"))
        .await
        .unwrap();

    for table in ["stage_tie_decisions", "stage_tie_decision_teams"] {
        assert_eq!(
            count(&db, &format!("select count(*) from {table}")).await,
            0,
            "{table}"
        );
    }
}

#[tokio::test]
async fn a_decision_only_changes_by_losing_its_author() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, tenant, t, teams) = started(&db, "organizer").await;
    let olga = member(&app, &db, &ana, "Olga").await;
    let rows = [(1, teams[1]), (2, teams[0])];
    exec(&db, &decision(tenant, t, olga.id, &rows))
        .await
        .unwrap();

    let immutable = rule(CHECK, "stage_tie_decisions_immutable");
    for change in [
        format!("decided_by = {}", ana.id),
        "decided_by = null, reason = 'x'".to_owned(),
    ] {
        let sql = format!("update stage_tie_decisions set {change}");
        assert_eq!(rejected(exec(&db, &sql).await), immutable, "{change}");
    }

    // Quem decidiu sai da comunidade: a decisão fica, sem autor, e não ganha
    // outro.
    let leave = format!("delete from tenant_members where user_id = {}", olga.id);
    assert_eq!(exec(&db, &leave).await.unwrap(), 1);
    let orphans = "select count(*) from stage_tie_decisions where decided_by is null";
    assert_eq!(count(&db, orphans).await, 1);
    let sql = format!("update stage_tie_decisions set decided_by = {}", ana.id);
    assert_eq!(rejected(exec(&db, &sql).await), immutable);
}

#[tokio::test]
async fn nothing_waits_on_the_seed_and_undoing_the_start_clears_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, tenant, t, teams) = started(&db, "seed").await;
    let rows = [(1, teams[1]), (2, teams[0])];
    assert_eq!(
        rejected(exec(&db, &decision(tenant, t, ana.id, &rows)).await),
        rule(CHECK, "stage_tie_decisions_organizer")
    );
    let mark = format!(
        "insert into stage_pending_ties (tenant_id, tournament_id, stage) values ({tenant}, {t}, 1)"
    );
    assert_eq!(
        rejected(exec(&db, &mark).await),
        rule(CHECK, "stage_pending_ties_organizer")
    );
    // final_tie só nas fases de tabela.
    let elimination = format!(
        "update tournament_stages
         set format = 'elimination', finals_best_of = 1, bracket = 'single', tiebreakers = null
         where tournament_id = {t}"
    );
    let (status, body) = delete(&app, &format!("{}/start", crate::matches::COPA), &ana).await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");
    assert_eq!(
        rejected(exec(&db, &elimination).await),
        rule(CHECK, "tournament_stages_shape")
    );
}

#[tokio::test]
async fn undoing_the_start_clears_the_ties() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, tenant, t, teams) = started(&db, "organizer").await;
    let rows = [(1, teams[1]), (2, teams[0])];
    exec(&db, &decision(tenant, t, ana.id, &rows))
        .await
        .unwrap();
    let mark = format!(
        "insert into stage_pending_ties (tenant_id, tournament_id, stage) values ({tenant}, {t}, 1)"
    );
    exec(&db, &mark).await.unwrap();
    let (status, body) = delete(&app, &format!("{}/start", crate::matches::COPA), &ana).await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");
    for table in [
        "stage_tie_decisions",
        "stage_tie_decision_teams",
        "stage_pending_ties",
    ] {
        assert_eq!(
            count(&db, &format!("select count(*) from {table}")).await,
            0,
            "{table}"
        );
    }
    // Fora do início, nada entra.
    assert_eq!(
        rejected(exec(&db, &decision(tenant, t, ana.id, &rows)).await),
        rule(CHECK, "stage_tie_decisions_started")
    );
}
