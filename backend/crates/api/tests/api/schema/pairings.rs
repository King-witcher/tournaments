//! Pareamentos do suíço (`stage_pairings`): só na fase do suíço de um
//! torneio iniciado, cada time uma vez por rodada, rodadas e partidas em
//! sequência com o bye por último, nunca mudam, e saem só com o torneio ou
//! ao desfazer o início.

use super::*;
use crate::common::*;
use crate::matches::{copa, start, table};

/// Um suíço de 4 iniciado (a rodada 1 gravada, com 2 pares). Devolve o app,
/// a owner, o tenant, o torneio e os times em ordem de seed.
async fn started_swiss(db: &PgPool) -> (Router, User, i64, i64, Vec<i64>) {
    let (app, ana, _) = quake_br(db).await;
    let p = crate::matches::players(&app, db, &ana, 4).await;
    let all: Vec<&User> = p.iter().collect();
    let body = copa(table("swiss", json!({ "best_of": 1, "rounds": null })));
    start(&app, &ana, body, &all).await;
    let teams = team_ids(&app, &ana, "copa", &all).await;
    let t = sqlx::query!("select tenant_id, id from tournaments where slug = 'copa'")
        .fetch_one(db)
        .await
        .unwrap();
    (app, ana, t.tenant_id, t.id, teams)
}

/// O insert de um pareamento `(round, number, a, b)` no torneio `t`.
fn insert(tenant: i64, t: i64, rows: &[(i16, i16, i64, Option<i64>)]) -> String {
    let values: Vec<String> = rows
        .iter()
        .map(|(round, number, a, b)| {
            let b = b.map_or("null".to_owned(), |b| b.to_string());
            format!("({tenant}, {t}, 1, {round}, {number}, {a}, {b})")
        })
        .collect();
    format!(
        "insert into stage_pairings (tenant_id, tournament_id, stage, round, number, side_a, side_b)
         values {}",
        values.join(", ")
    )
}

#[tokio::test]
async fn pairings_follow_the_rounds_and_never_change() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, _, tenant, t, teams) = started_swiss(&db).await;
    let [a, b, c, d] = [teams[0], teams[1], teams[2], teams[3]];
    assert_eq!(count(&db, "select count(*) from stage_pairings").await, 2);

    assert_eq!(
        rejected(exec(&db, "update stage_pairings set created_at = now()").await),
        rule(CHECK, "stage_pairings_immutable")
    );
    // Na mesma rodada, cada time uma vez.
    assert_eq!(
        rejected(exec(&db, &insert(tenant, t, &[(1, 3, a, None)])).await),
        rule(CHECK, "stage_pairings_team_once")
    );
    assert_eq!(
        rejected(exec(&db, &insert(tenant, t, &[(2, 1, a, Some(a))])).await),
        rule(CHECK, "stage_pairings_sides")
    );
    // Rodada pulada, partida pulada, bye antes de um par, rodada só de bye.
    for rows in [
        vec![(3, 1, a, Some(b)), (3, 2, c, Some(d))],
        vec![(2, 1, a, Some(b)), (2, 3, c, Some(d))],
        vec![(2, 1, a, None), (2, 2, c, Some(d))],
        vec![(2, 1, a, None)],
    ] {
        assert_eq!(
            rejected(exec(&db, &insert(tenant, t, &rows)).await),
            rule(CHECK, "stage_pairings_sequence"),
            "{rows:?}"
        );
    }
    exec(
        &db,
        &insert(tenant, t, &[(2, 1, a, Some(c)), (2, 2, b, Some(d))]),
    )
    .await
    .unwrap();

    // Pareamento de torneio iniciado não sai avulso, nem fica num torneio
    // que volta a não ter começado.
    assert_eq!(
        rejected(exec(&db, "delete from stage_pairings where round = 2").await),
        rule(FOREIGN_KEY, "stage_pairings_started_delete")
    );
    let unstart = format!("update tournaments set status = 'registration' where id = {t}");
    assert_eq!(
        rejected(exec(&db, &unstart).await),
        rule(CHECK, "stage_pairings_started")
    );
    // Com o torneio, saem juntos.
    exec(&db, &format!("delete from tournaments where id = {t}"))
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from stage_pairings").await, 0);
}

#[tokio::test]
async fn pairings_only_in_a_started_swiss() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, tenant, t, teams) = started_swiss(&db).await;

    // Desfazer o início apaga os pareamentos.
    let (status, body) = delete(&app, &format!("{}/start", crate::matches::COPA), &ana).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(count(&db, "select count(*) from stage_pairings").await, 0);
    // Fora do início, nada entra.
    assert_eq!(
        rejected(exec(&db, &insert(tenant, t, &[(1, 1, teams[0], Some(teams[1]))])).await),
        rule(CHECK, "stage_pairings_started")
    );
    // E só na fase do suíço.
    let elimination = json!({ "format": "elimination", "kind": { "kind": "single" },
                              "series": { "default": 1, "finals": 1 } });
    sqlx::query("update tournament_stages set config = $1 where tournament_id = $2")
        .bind(&elimination)
        .bind(t)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        rejected(exec(&db, &insert(tenant, t, &[(1, 1, teams[0], Some(teams[1]))])).await),
        rule(CHECK, "stage_pairings_swiss")
    );
}
