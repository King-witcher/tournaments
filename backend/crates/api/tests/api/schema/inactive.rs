//! Membro inativo (saiu ou foi removido): a linha fica, sem acesso, sem
//! convite aberto e sem inscrição em torneio que não começou; o owner não
//! fica inativo.

use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{copa, single, start};

fn leave_sql(user: &User) -> String {
    format!(
        "update tenant_members set left_at = now() where user_id = {}",
        user.id
    )
}

#[sqlx::test(migrations = "../../migrations")]
async fn inactive_members_keep_their_row_and_their_rules(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let olga = organizer(&app, &db, &ana, "Olga").await;
    let revoke_all = "update tenant_invites set revoked_at = now()";

    // O owner transfere a posse antes de sair.
    exec(&db, revoke_all).await.unwrap();
    assert_eq!(
        rejected(exec(&db, &leave_sql(&ana)).await),
        rule(CHECK, "tenant_members_owner_active")
    );
    // Quem removeu: só de quem saiu, e nunca ele mesmo.
    let removed_by = |who: &User, by: &User, left: bool| {
        format!(
            "update tenant_members set removed_by = {}, left_at = {} where user_id = {}",
            by.id,
            if left { "now()" } else { "null" },
            who.id
        )
    };
    assert_eq!(
        rejected(exec(&db, &removed_by(&bia, &ana, false)).await),
        rule(CHECK, "tenant_members_removed_by_check")
    );
    assert_eq!(
        rejected(exec(&db, &removed_by(&bia, &bia, true)).await),
        rule(CHECK, "tenant_members_removed_by_check")
    );
    let outsider = user(&db, "Zeca").await;
    assert_eq!(
        rejected(exec(&db, &removed_by(&bia, &outsider, true)).await),
        rule(FOREIGN_KEY, "tenant_members_removed_by_fkey")
    );

    // Não sai com convite aberto (a API revoga antes).
    invite(&app, &olga, "quake-br").await;
    assert_eq!(
        rejected(exec(&db, &leave_sql(&olga)).await),
        rule(CHECK, "tenant_members_left_invites_open")
    );
    exec(&db, revoke_all).await.unwrap();
    exec(&db, &removed_by(&olga, &ana, true)).await.unwrap();
    // Inativo não abre convite, nem reabre um revogado.
    let tenant = sqlx::query_scalar!("select id from tenants")
        .fetch_one(&db)
        .await
        .unwrap();
    let new_invite = format!(
        "insert into tenant_invites (tenant_id, token_hash, created_by, expires_at)
         values ({tenant}, sha256('olga'), {}, now() + interval '1 day')",
        olga.id
    );
    assert_eq!(
        rejected(exec(&db, &new_invite).await),
        rule(CHECK, "tenant_invites_creator_active")
    );
    assert_eq!(
        rejected(exec(&db, "update tenant_invites set revoked_at = null").await),
        rule(CHECK, "tenant_invites_creator_active")
    );

    // Apagar quem removeu deixa a remoção sem autor.
    exec(&db, &removed_by(&bia, &olga, true)).await.unwrap();
    exec(&db, &format!("delete from users where id = {}", olga.id))
        .await
        .unwrap();
    let sql = format!(
        "select count(*) from tenant_members where user_id = {} and removed_by is null and left_at is not null",
        bia.id
    );
    assert_eq!(count(&db, &sql).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn inactive_members_are_only_registered_in_started_tournaments(db: PgPool) {
    let (app, ana, bia) = quake_br(&db).await;
    let caio = member(&app, &db, &ana, "Caio").await;
    create_tournament(&app, &ana, duel("aberta")).await;
    publish(&app, &ana, "aberta").await;
    register(&app, "aberta", &[&bia]).await;

    // Inscrita em torneio que não começou, não sai (a API tira antes).
    assert_eq!(
        rejected(exec(&db, &leave_sql(&bia)).await),
        rule(CHECK, "tenant_members_left_registered")
    );
    exec(
        &db,
        &format!("delete from participants where user_id = {}", bia.id),
    )
    .await
    .unwrap();
    exec(&db, &leave_sql(&bia)).await.unwrap();
    // Inativa, não entra.
    let insert = format!(
        "insert into participants (tenant_id, tournament_id, user_id)
         select tenant_id, id, {} from tournaments where slug = 'aberta'",
        bia.id
    );
    assert_eq!(
        rejected(exec(&db, &insert).await),
        rule(CHECK, "participants_member_active")
    );

    // No torneio iniciado, a inscrição de quem sai fica; se ele volta a não
    // ter começado, ela sai antes do commit.
    start(&app, &ana, copa(single(3, 3)), &[&ana, &caio]).await;
    exec(&db, &leave_sql(&caio)).await.unwrap();
    let mut tx = db.begin().await.unwrap();
    sqlx::query("update tournaments set status = 'registration' where slug = 'copa'")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        rejected(tx.commit().await),
        rule(CHECK, "participants_member_active")
    );
    let mut tx = db.begin().await.unwrap();
    sqlx::query("delete from match_results")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("update tournaments set status = 'registration' where slug = 'copa'")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("delete from participants where user_id = $1")
        .bind(caio.id)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}
