//! Exclusão de conta e de comunidade: a forma da conta anonimizada (sem nome,
//! contato, login, push nem participação ativa, e não volta), o owner e a
//! exclusão agendada da conta, e a da comunidade (pedida pelo owner, sem
//! torneio em andamento).

use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{copa, single, start};

/// Anonimiza `who` numa transação, como a API: sai de `quake-br`, apelidos
/// vazios, e a conta marcada.
async fn anonymize(db: &PgPool, who: &User) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    for sql in [
        "update tenant_members set left_at = now() where user_id = $1",
        "update tenant_members set nickname = '' where user_id = $1",
        "update users set display_name = '', deleted_at = now() where id = $1",
    ] {
        sqlx::query(sql).bind(who.id).execute(&mut *tx).await?;
    }
    tx.commit().await
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_anonymized_account_keeps_only_the_row(db: PgPool) {
    let (_app, _ana, bia) = quake_br(&db).await;
    let bia_sql = |sql: &str| sql.replace("{bia}", &bia.id.to_string());

    // Nome vazio é só da conta excluída, e ela não tem contato.
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql("update users set display_name = '' where id = {bia}")
            )
            .await
        ),
        rule(CHECK, "users_display_name_check")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql("update users set display_name = '', email = 'bia@example.com', deleted_at = now() where id = {bia}")
            )
            .await
        ),
        rule(CHECK, "users_deleted_check")
    );
    // Nem apelido vazio, nem membro ativo, nem login.
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql("update tenant_members set nickname = '' where user_id = {bia}")
            )
            .await
        ),
        rule(CHECK, "tenant_members_nickname_deleted")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql("update users set display_name = '', deleted_at = now() where id = {bia}")
            )
            .await
        ),
        rule(CHECK, "users_deleted_members")
    );
    let discord = bia_sql(
        "insert into user_identities (user_id, provider, provider_user_id) values ({bia}, 'discord', 'd-bia')",
    );
    exec(&db, &discord).await.unwrap();
    assert_eq!(
        rejected(anonymize(&db, &bia).await),
        rule(CHECK, "users_deleted_logins")
    );
    exec(
        &db,
        &bia_sql("delete from user_identities where user_id = {bia}"),
    )
    .await
    .unwrap();
    anonymize(&db, &bia).await.unwrap();

    // Depois, não ganha login, push, participação nem nome, e não volta.
    assert_eq!(
        rejected(exec(&db, &discord).await),
        rule(CHECK, "user_identities_user_deleted")
    );
    let password = bia_sql(&format!(
        "insert into password_credentials (user_id, username, password_hash) values ({{bia}}, 'bia', '{}')",
        "$argon2id$v=19$m=19456,t=2,p=1$x$y"
    ));
    assert_eq!(
        rejected(exec(&db, &password).await),
        rule(CHECK, "password_credentials_user_deleted")
    );
    let push = bia_sql(&format!(
        "insert into push_subscriptions (user_id, endpoint, p256dh, auth, locale)
         values ({{bia}}, 'https://push.example.com/bia', '{}', '{}', 'pt')",
        "A".repeat(87),
        "B".repeat(22)
    ));
    assert_eq!(
        rejected(exec(&db, &push).await),
        rule(CHECK, "push_subscriptions_user_deleted")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql("update tenant_members set left_at = null where user_id = {bia}")
            )
            .await
        ),
        rule(CHECK, "tenant_members_user_deleted")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql("update tenant_members set nickname = 'Bia' where user_id = {bia}")
            )
            .await
        ),
        rule(CHECK, "tenant_members_nickname_deleted")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql(
                    "update users set deleted_at = null, display_name = 'Bia' where id = {bia}"
                )
            )
            .await
        ),
        rule(CHECK, "users_deleted_final")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &bia_sql("update users set deletion_due_at = now() where id = {bia}")
            )
            .await
        ),
        rule(CHECK, "users_deleted_check")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_owner_does_not_schedule_and_the_scheduled_does_not_own(db: PgPool) {
    let (_app, ana, bia) = quake_br(&db).await;
    let due = |who: &User| {
        format!(
            "update users set deletion_due_at = now() + interval '7 days' where id = {}",
            who.id
        )
    };
    assert_eq!(
        rejected(exec(&db, &due(&ana)).await),
        rule(CHECK, "users_deletion_owner")
    );
    exec(&db, &due(&bia)).await.unwrap();
    let mut tx = db.begin().await.unwrap();
    sqlx::query("update tenant_members set role = 'organizer' where user_id = $1")
        .bind(ana.id)
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        rejected(
            sqlx::query("update tenant_members set role = 'owner' where user_id = $1")
                .bind(bia.id)
                .execute(&mut *tx)
                .await
        ),
        rule(CHECK, "tenant_members_owner_deleting")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_the_owner_schedules_the_community_deletion(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    create_tournament(&app, &ana, copa(single(3, 3))).await;
    let schedule = |by: i64| {
        format!(
            "update tenants set deletion_due_at = now() + interval '7 days', deletion_requested_by = {by}"
        )
    };
    assert_eq!(
        rejected(exec(&db, "update tenants set deletion_due_at = now()").await),
        rule(CHECK, "tenants_deletion_check")
    );
    assert_eq!(
        rejected(exec(&db, &schedule(bia.id)).await),
        rule(CHECK, "tenants_deletion_by_owner")
    );
    let outsider = user(&db, "Zeca").await;
    assert_eq!(
        rejected(exec(&db, &schedule(outsider.id)).await),
        rule(FOREIGN_KEY, "tenants_deletion_requested_by_fkey")
    );
    exec(&db, &schedule(ana.id)).await.unwrap();

    // Passar a posse com a exclusão agendada deixaria o pedido sem owner.
    let mut tx = db.begin().await.unwrap();
    for (who, role) in [(ana.id, "organizer"), (bia.id, "owner")] {
        sqlx::query("update tenant_members set role = $2 where user_id = $1")
            .bind(who)
            .bind(role)
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    assert_eq!(
        rejected(tx.commit().await),
        rule(CHECK, "tenants_deletion_by_owner")
    );

    // Torneio não começa com a exclusão agendada, e a exclusão não é agendada
    // com torneio em andamento.
    assert_eq!(
        rejected(exec(&db, "update tournaments set status = 'in_progress'").await),
        rule(CHECK, "tournaments_tenant_deleting")
    );
    exec(
        &db,
        "update tenants set deletion_due_at = null, deletion_requested_by = null",
    )
    .await
    .unwrap();
    exec(&db, "delete from tournaments").await.unwrap();
    start(&app, &ana, copa(single(3, 3)), &[&ana, &bia]).await;
    assert_eq!(
        rejected(exec(&db, &schedule(ana.id)).await),
        rule(CHECK, "tenants_deletion_in_progress")
    );

    // A cascata do tenant passa com a exclusão agendada (a FK para o owner
    // sai junto).
    exec(&db, "update tournaments set status = 'finished'")
        .await
        .unwrap();
    exec(&db, &schedule(ana.id)).await.unwrap();
    exec(&db, "delete from tenants").await.unwrap();
    assert_eq!(count(&db, "select count(*) from tenant_members").await, 0);
    assert_eq!(count(&db, "select count(*) from participants").await, 0);
}
