//! Desistência, que é do time: só com o torneio em andamento, só de quem está
//! na chave, não se desfaz, e o desistente sai da inscrição se o torneio
//! volta a não ter começado. O W.O. automático só vale contra o time que
//! desistiu ou não tem mais ninguém na comunidade.

use super::*;
use crate::common::*;
use crate::matches::{copa, single, start};

/// O time de `user` desiste (no duelo, o time de um jogador dele).
fn withdraw_sql(user: &User) -> String {
    format!(
        "update teams set withdrawn_at = now()
         where id = (select team_id from participants where user_id = {})",
        user.id
    )
}

#[tokio::test]
async fn withdrawals_only_happen_in_progress_and_stay() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    let dani = member(&app, &db, &ana, "Dani").await;
    create_tournament(&app, &ana, copa(single(3, 3))).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia, &caio, &dani]).await;

    // Antes de começar, não.
    assert_eq!(
        rejected(exec(&db, &withdraw_sql(&bia)).await),
        rule(CHECK, "teams_withdrawal_in_progress")
    );
    let (status, body) = post_empty(&app, "/tenants/quake-br/tournaments/copa/start", &ana).await;
    assert_eq!(status, axum::http::StatusCode::OK, "{body}");

    // Na chave e em andamento, sim; com autor só quem marcou.
    let by_without = format!(
        "update teams set withdrawn_by = {}
         where id = (select team_id from participants where user_id = {})",
        ana.id, caio.id
    );
    assert_eq!(
        rejected(exec(&db, &by_without).await),
        rule(CHECK, "teams_withdrawn_by_check")
    );
    exec(&db, &withdraw_sql(&bia)).await.unwrap();
    // Não se desfaz, nem muda de data.
    let undo = format!(
        "update teams set withdrawn_at = null
         where id = (select team_id from participants where user_id = {})",
        bia.id
    );
    assert_eq!(
        rejected(exec(&db, &undo).await),
        rule(CHECK, "teams_withdrawal_final")
    );
    let again = format!(
        "update teams set withdrawn_at = now() - interval '1 day'
         where id = (select team_id from participants where user_id = {})",
        bia.id
    );
    assert_eq!(
        rejected(exec(&db, &again).await),
        rule(CHECK, "teams_withdrawal_final")
    );

    // O torneio só volta a não ter começado sem o desistente.
    let mut tx = db.begin().await.unwrap();
    sqlx::query("update tournaments set status = 'registration'")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        rejected(tx.commit().await),
        rule(CHECK, "teams_withdrawn_started")
    );
    // Encerrado, continua desistente.
    exec(&db, "update tournaments set status = 'finished'")
        .await
        .unwrap();
}

#[tokio::test]
async fn withdrawn_seed_is_required() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    // Caio se inscreve, mas o seeding deixa só Ana e Bia na chave.
    create_tournament(&app, &ana, duel("copa")).await;
    publish(&app, &ana, "copa").await;
    register(&app, "copa", &[&ana, &bia, &caio]).await;
    let base = "/tenants/quake-br/tournaments/copa";
    let (status, _) = post_empty(&app, &format!("{base}/check-in/open"), &ana).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    for who in [&ana, &bia] {
        let (status, _) = post_empty(&app, &format!("{base}/check-in"), who).await;
        assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
    }
    let (status, _) = post_empty(&app, &format!("{base}/start"), &ana).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(
        rejected(exec(&db, &withdraw_sql(&caio)).await),
        rule(CHECK, "teams_withdrawn_seeded")
    );
}

#[tokio::test]
async fn automatic_walkovers_need_a_withdrawn_loser() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, bia) = quake_br(&db).await;
    start(&app, &ana, copa(single(3, 3)), &[&ana, &bia]).await;
    let ids = sqlx::query!(
        "select tenant_id, tournament_id, id from teams where seed is not null order by seed"
    )
    .fetch_all(&db)
    .await
    .unwrap();
    let (tenant, tournament) = (ids[0].tenant_id, ids[0].tournament_id);
    let (a, b) = (ids[0].id, ids[1].id);
    let walkover = |winner: i64, extra: &str, values: &str| {
        format!(
            "insert into match_results
                 (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                  winner, confirmed_via, confirmed_at, position{extra})
             values ({tenant}, {tournament}, 1, 'wb-1-1', 3, {a}, {b}, 'confirmed',
                     {winner}, 'withdrawal', now(), 1{values})"
        )
    };

    // Ninguém desistiu.
    assert_eq!(
        rejected(exec(&db, &walkover(a, "", "")).await),
        rule(CHECK, "match_results_withdrawal_loser")
    );
    // O time sem ninguém na comunidade também está fora.
    let mut tx = db.begin().await.unwrap();
    let left = format!(
        "update tenant_members set left_at = now() where user_id = {}",
        bia.id
    );
    for sql in [left, walkover(a, "", "")] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx.rollback().await.unwrap();
    exec(&db, &withdraw_sql(&bia)).await.unwrap();
    // Contra quem desistiu, sim; o vencedor pode ser só quem ficou.
    assert_eq!(
        rejected(exec(&db, &walkover(b, "", "")).await),
        rule(CHECK, "match_results_withdrawal_loser")
    );
    // Sem placar e sem autor.
    assert_eq!(
        rejected(exec(&db, &walkover(a, ", a_wins, b_wins", ", 2, 0")).await),
        rule(CHECK, "match_results_withdrawal_check")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &walkover(a, ", confirmed_by", &format!(", {}", ana.id))
            )
            .await
        ),
        rule(CHECK, "match_results_withdrawal_check")
    );
    exec(&db, &walkover(a, "", "")).await.unwrap();

    // O evento: só o vencedor e sem autor.
    let event = |winner: &str, actor: &str| {
        format!(
            "insert into match_events (tenant_id, tournament_id, stage, match_key, kind, winner, actor)
             values ({tenant}, {tournament}, 1, 'wb-1-1', 'auto_walkover', {winner}, {actor})"
        )
    };
    assert_eq!(
        rejected(exec(&db, &event("null", "null")).await),
        rule(CHECK, "match_events_outcome_check")
    );
    assert_eq!(
        rejected(exec(&db, &event(&a.to_string(), &ana.id.to_string())).await),
        rule(CHECK, "match_events_auto_walkover_actor")
    );
    exec(&db, &event(&a.to_string(), "null")).await.unwrap();
}
