//! Exatamente um owner por tenant: no máximo um na hora (índice único
//! parcial), pelo menos um no commit (a transferência cabe numa transação).

use sqlx::PgPool;

use super::*;
use crate::common::*;

async fn roles(db: &PgPool) -> Vec<(String, String)> {
    sqlx::query!(
        "select u.display_name, m.role from tenant_members m join users u on u.id = m.user_id
         order by u.display_name"
    )
    .fetch_all(db)
    .await
    .unwrap()
    .into_iter()
    .map(|r| (r.display_name, r.role))
    .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|&(a, b)| (a.to_owned(), b.to_owned()))
        .collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn ownership_moves_inside_one_transaction(db: PgPool) {
    let (_app, ana, bia) = quake_br(&db).await;
    let set_role = |id: i64, role: &'static str| {
        sqlx::query("update tenant_members set role = $2 where user_id = $1")
            .bind(id)
            .bind(role)
    };

    // O owner atual sai primeiro, depois o novo entra.
    let mut tx = db.begin().await.unwrap();
    set_role(ana.id, "organizer")
        .execute(&mut *tx)
        .await
        .unwrap();
    set_role(bia.id, "owner").execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        roles(&db).await,
        pairs(&[("Ana", "organizer"), ("Bia", "owner")])
    );

    // Na ordem inversa, dois owners ao mesmo tempo: o índice recusa na hora.
    let mut tx = db.begin().await.unwrap();
    assert_eq!(
        rejected(set_role(ana.id, "owner").execute(&mut *tx).await),
        rule(UNIQUE, "tenant_members_one_owner_idx")
    );
    drop(tx);

    // Sem owner no commit, não.
    let no_owner = rule(CHECK, "tenant_members_owner_required");
    let mut tx = db.begin().await.unwrap();
    set_role(bia.id, "player").execute(&mut *tx).await.unwrap();
    assert_eq!(rejected(tx.commit().await), no_owner);
    let mut tx = db.begin().await.unwrap();
    sqlx::query("delete from tenant_members where user_id = $1")
        .bind(bia.id)
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(rejected(tx.commit().await), no_owner);
    assert_eq!(
        roles(&db).await,
        pairs(&[("Ana", "organizer"), ("Bia", "owner")])
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_tenant_is_born_with_its_owner(db: PgPool) {
    let ana = user(&db, "Ana").await;
    let new_tenant =
        "insert into tenants (slug, name) values ('quake-br', 'Quake BR') returning id";

    let mut tx = db.begin().await.unwrap();
    sqlx::query_scalar::<_, i64>(new_tenant)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        rejected(tx.commit().await),
        rule(CHECK, "tenant_members_owner_required")
    );

    let mut tx = db.begin().await.unwrap();
    let id = sqlx::query_scalar::<_, i64>(new_tenant)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "insert into tenant_members (tenant_id, user_id, role, nickname) values ($1, $2, 'owner', 'Ana')",
    )
    .bind(id)
    .bind(ana.id)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Apagar o tenant leva o owner junto, sem esbarrar na regra.
    exec(&db, "delete from tenants").await.unwrap();
    assert_eq!(count(&db, "select count(*) from tenant_members").await, 0);
}
