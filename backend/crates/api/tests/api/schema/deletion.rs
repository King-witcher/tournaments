//! O que cada exclusão apaga, deixa sem autor ou recusa.

use sqlx::PgPool;

use super::*;
use crate::common::*;

/// Linhas de cada tabela de tenant que pertencem ao tenant `tenant`.
async fn tenant_rows(db: &PgPool, tenant: i64) -> Vec<(&'static str, i64)> {
    let mut rows = Vec::new();
    for (table, sql) in [
        (
            "tenant_members",
            "select count(*) from tenant_members where tenant_id = $1",
        ),
        (
            "tenant_invites",
            "select count(*) from tenant_invites where tenant_id = $1",
        ),
        (
            "tournaments",
            "select count(*) from tournaments where tenant_id = $1",
        ),
        (
            "participants",
            "select count(*) from participants where tenant_id = $1",
        ),
        (
            "match_results",
            "select count(*) from match_results where tenant_id = $1",
        ),
        (
            "match_events",
            "select count(*) from match_events where tenant_id = $1",
        ),
    ] {
        let n = sqlx::query_scalar::<_, i64>(sql)
            .bind(tenant)
            .fetch_one(db)
            .await
            .unwrap();
        rows.push((table, n));
    }
    rows
}

/// Linhas de cada tabela de torneio que pertencem ao torneio `tournament`.
async fn tournament_rows(db: &PgPool, tournament: i64) -> Vec<(&'static str, i64)> {
    let mut rows = Vec::new();
    for (table, sql) in [
        (
            "tournament_translations",
            "select count(*) from tournament_translations where tournament_id = $1",
        ),
        (
            "tournament_stages",
            "select count(*) from tournament_stages where tournament_id = $1",
        ),
        (
            "tenant_invites",
            "select count(*) from tenant_invites where tournament_id = $1",
        ),
        (
            "participants",
            "select count(*) from participants where tournament_id = $1",
        ),
        (
            "match_results",
            "select count(*) from match_results where tournament_id = $1",
        ),
        (
            "match_events",
            "select count(*) from match_events where tournament_id = $1",
        ),
    ] {
        let n = sqlx::query_scalar::<_, i64>(sql)
            .bind(tournament)
            .fetch_one(db)
            .await
            .unwrap();
        rows.push((table, n));
    }
    rows
}

fn all_zero(rows: &[(&'static str, i64)]) -> Vec<(&'static str, i64)> {
    rows.iter().map(|&(t, _)| (t, 0)).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_tenant_deletes_everything_inside(db: PgPool) {
    let cup = finished_cup(&db).await;
    // Outro tenant, com torneio, não é tocado.
    let zeca = user(&db, "Zeca").await;
    create(&cup.app, &zeca, "outro").await;
    let (status, body) = post(&cup.app, "/tenants/outro/tournaments", &zeca, duel("liga")).await;
    assert_eq!(status, axum::http::StatusCode::CREATED, "{body}");
    let other = sqlx::query_scalar!("select id from tenants where slug = 'outro'")
        .fetch_one(&db)
        .await
        .unwrap();
    let other_before = tenant_rows(&db, other).await;

    let before = tenant_rows(&db, cup.tenant).await;
    assert!(before.iter().all(|&(_, n)| n > 0), "{before:?}");
    let inside = tournament_rows(&db, cup.tournament).await;
    assert!(inside.iter().all(|&(_, n)| n > 0), "{inside:?}");

    exec(&db, "delete from tenants where slug = 'quake-br'")
        .await
        .unwrap();

    assert_eq!(tenant_rows(&db, cup.tenant).await, all_zero(&before));
    assert_eq!(
        tournament_rows(&db, cup.tournament).await,
        all_zero(&inside)
    );
    assert_eq!(count(&db, "select count(*) from match_games").await, 0);
    assert_eq!(tenant_rows(&db, other).await, other_before);
    // Os usuários continuam: só a participação no tenant saiu.
    assert_eq!(count(&db, "select count(*) from users").await, 7);
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_tournament_deletes_everything_inside(db: PgPool) {
    let cup = finished_cup(&db).await;
    // Outro torneio do tenant, com inscrito.
    create_tournament(&cup.app, &cup.ana, duel("outra")).await;
    publish(&cup.app, &cup.ana, "outra").await;
    register(&cup.app, "outra", &[&cup.bia]).await;
    let other = sqlx::query_scalar!("select id from tournaments where slug = 'outra'")
        .fetch_one(&db)
        .await
        .unwrap();
    let other_before = tournament_rows(&db, other).await;
    let inside = tournament_rows(&db, cup.tournament).await;
    let members = count(&db, "select count(*) from tenant_members").await;

    // Encerrado, com resultados, mapas e histórico.
    exec(&db, "delete from tournaments where slug = 'copa'")
        .await
        .unwrap();

    assert_eq!(
        tournament_rows(&db, cup.tournament).await,
        all_zero(&inside)
    );
    assert_eq!(count(&db, "select count(*) from match_games").await, 0);
    assert_eq!(tournament_rows(&db, other).await, other_before);
    // Membros e convites do tenant ficam.
    assert_eq!(
        count(&db, "select count(*) from tenant_members").await,
        members
    );
    assert_eq!(
        count(
            &db,
            "select count(*) from tenant_invites where tournament_id is null"
        )
        .await,
        4
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn started_participants_and_stages_only_leave_with_the_tournament(db: PgPool) {
    let cup = finished_cup(&db).await;
    let p1 = cup.p1.id;
    let started_delete = rule(FOREIGN_KEY, "participants_started_delete");
    let frozen = rule(CHECK, "participants_started_frozen");

    // Nem o inscrito nem o membro saem (a chave perderia um lado).
    let delete_p1 = format!("delete from participants where user_id = {p1}");
    assert_eq!(rejected(exec(&db, &delete_p1).await), started_delete);
    let leave = format!("delete from tenant_members where user_id = {p1}");
    assert_eq!(rejected(exec(&db, &leave).await), started_delete);

    // Ninguém entra; seed e check-in não mudam.
    let insert = format!(
        "insert into participants (tenant_id, tournament_id, user_id) values ({}, {}, {})",
        cup.tenant, cup.tournament, cup.bia.id
    );
    assert_eq!(rejected(exec(&db, &insert).await), frozen);
    for change in [
        "seed = 3",
        "seed = null",
        "checked_in_at = null, checked_in_by = null",
    ] {
        let sql = format!("update participants set {change} where user_id = {p1}");
        assert_eq!(rejected(exec(&db, &sql).await), frozen, "{change}");
    }
    // Inscrito não troca de membro nem de torneio, em status nenhum.
    let sql = format!(
        "update participants set user_id = {} where user_id = {p1}",
        cup.bia.id
    );
    assert_eq!(
        rejected(exec(&db, &sql).await),
        rule(CHECK, "participants_identity_immutable")
    );
    // A autoria pode sair.
    let sql = format!("update participants set added_by = null where user_id = {p1}");
    assert_eq!(exec(&db, &sql).await.unwrap(), 1);

    // A fase também fica.
    assert_eq!(
        rejected(exec(&db, "delete from tournament_stages").await),
        rule(FOREIGN_KEY, "tournament_stages_started_delete")
    );
    let stage_frozen = rule(CHECK, "tournament_stages_started_frozen");
    let sql = "update tournament_stages set config = config || '{\"series\": {\"default\": 1, \"finals\": 1}}'";
    assert_eq!(rejected(exec(&db, sql).await), stage_frozen);
    let sql = format!(
        "insert into tournament_stages (tournament_id, position, config)
         select tournament_id, 2, config from tournament_stages where tournament_id = {}",
        cup.tournament
    );
    assert_eq!(rejected(exec(&db, &sql).await), stage_frozen);

    // Antes de começar, sai normalmente (e sair do tenant tira da inscrição).
    create_tournament(&cup.app, &cup.ana, duel("outra")).await;
    publish(&cup.app, &cup.ana, "outra").await;
    register(&cup.app, "outra", &[&cup.bia, &cup.ze]).await;
    let sql = format!("delete from participants where user_id = {}", cup.bia.id);
    assert_eq!(exec(&db, &sql).await.unwrap(), 1);
    let sql = format!("delete from tenant_members where user_id = {}", cup.ze.id);
    assert_eq!(exec(&db, &sql).await.unwrap(), 1);
    assert_eq!(
        count(
            &db,
            "select count(*) from participants p join tournaments t on t.id = p.tournament_id
             where t.slug = 'outra'"
        )
        .await,
        0
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn owners_and_started_participants_cannot_be_deleted(db: PgPool) {
    let cup = finished_cup(&db).await;
    let delete_user = |id: i64| format!("delete from users where id = {id}");

    assert_eq!(
        rejected(exec(&db, &delete_user(cup.ana.id)).await),
        rule(FOREIGN_KEY, "users_owner_delete")
    );
    for p in [&cup.p1, &cup.p2] {
        assert_eq!(
            rejected(exec(&db, &delete_user(p.id)).await),
            rule(FOREIGN_KEY, "users_participant_delete")
        );
    }
    // Owner de tenant sem torneio também não.
    let zeca = user(&db, "Zeca").await;
    create(&cup.app, &zeca, "outro").await;
    assert_eq!(
        rejected(exec(&db, &delete_user(zeca.id)).await).1,
        "users_owner_delete"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_an_author_keeps_what_they_did(db: PgPool) {
    let cup = finished_cup(&db).await;
    let olga = cup.olga.id;
    exec(&db, &format!("delete from users where id = {olga}"))
        .await
        .unwrap();

    // Inscrição e check-in que ela marcou ficam, sem autor.
    let p1 = sqlx::query!(
        "select added_by, checked_in_at, checked_in_by from participants where user_id = $1",
        cup.p1.id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!((p1.added_by, p1.checked_in_by), (None, None));
    assert!(p1.checked_in_at.is_some());
    // O resultado e o histórico também.
    let result = sqlx::query!("select status, reported_by, confirmed_by from match_results")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(
        (
            result.status.as_str(),
            result.reported_by,
            result.confirmed_by
        ),
        ("confirmed", None, None)
    );
    let actors = sqlx::query!("select kind, actor from match_events order by id")
        .fetch_all(&db)
        .await
        .unwrap()
        .into_iter()
        .map(|e| (e.kind, e.actor))
        .collect::<Vec<_>>();
    assert_eq!(
        actors,
        [
            ("reported".to_owned(), Some(cup.p1.id)),
            ("contested".to_owned(), Some(cup.p2.id)),
            ("resolved".to_owned(), None),
        ]
    );
    // Os convites dela saem; quem entrou ou se inscreveu por eles fica.
    let invites = format!("select count(*) from tenant_invites where created_by = {olga}");
    assert_eq!(count(&db, &invites).await, 0);
    let ze = sqlx::query_scalar!(
        "select invite_id from tenant_members where user_id = $1",
        cup.ze.id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(ze, None);
    let p2 = sqlx::query_scalar!(
        "select invite_id from participants where user_id = $1",
        cup.p2.id
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!(p2, None);
    for table in ["sessions", "tenant_members"] {
        let sql = format!("select count(*) from {table} where user_id = {olga}");
        assert_eq!(count(&db, &sql).await, 0, "{table}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_user_takes_logins_sessions_and_registrations(db: PgPool) {
    let cup = finished_cup(&db).await;
    let bia = cup.bia.id;
    sqlx::query(
        "insert into user_identities (user_id, provider, provider_user_id, password_hash)
         values ($1, 'password', 'Bia_1', '$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA'),
                ($1, 'discord', '987654321', null)",
    )
    .bind(bia)
    .execute(&db)
    .await
    .unwrap();
    exec(
        &db,
        "insert into password_failures (username) values ('bia_1'), ('bia_1'), ('outra')",
    )
    .await
    .unwrap();
    // Inscrita num torneio que não começou.
    create_tournament(&cup.app, &cup.ana, duel("outra")).await;
    publish(&cup.app, &cup.ana, "outra").await;
    register(&cup.app, "outra", &[&cup.bia]).await;

    exec(&db, &format!("delete from users where id = {bia}"))
        .await
        .unwrap();

    for table in [
        "sessions",
        "user_identities",
        "tenant_members",
        "participants",
    ] {
        let sql = format!("select count(*) from {table} where user_id = {bia}");
        assert_eq!(count(&db, &sql).await, 0, "{table}");
    }
    // As tentativas do username dela saem; as dos outros, não.
    let failures = sqlx::query_scalar!("select username from password_failures")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(failures, ["outra"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn password_failures_leave_with_the_username(db: PgPool) {
    let me = user(&db, "Rapha").await;
    sqlx::query(
        "insert into user_identities (user_id, provider, provider_user_id, password_hash)
         values ($1, 'password', 'Rapha', '$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA')",
    )
    .bind(me.id)
    .execute(&db)
    .await
    .unwrap();
    let failures = "select count(*) from password_failures";
    exec(
        &db,
        "insert into password_failures (username) values ('rapha')",
    )
    .await
    .unwrap();

    // Trocar só a senha (ou a caixa do username) mantém.
    exec(&db, "update user_identities set provider_user_id = 'RAPHA'")
        .await
        .unwrap();
    assert_eq!(count(&db, failures).await, 1);
    // Trocar o username apaga as do antigo.
    exec(
        &db,
        "update user_identities set provider_user_id = 'rapha2'",
    )
    .await
    .unwrap();
    assert_eq!(count(&db, failures).await, 0);
    // Remover o login também.
    exec(
        &db,
        "insert into password_failures (username) values ('rapha2')",
    )
    .await
    .unwrap();
    exec(&db, "delete from user_identities").await.unwrap();
    assert_eq!(count(&db, failures).await, 0);
}
