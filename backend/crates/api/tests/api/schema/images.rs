//! O logo, a capa e a fila do que sai do bucket público.

use super::*;

const TOKEN: &str = "abcdefghijkmnopqrstuvw";

async fn queued(db: &PgPool) -> Vec<String> {
    sqlx::query_scalar("select object_key from image_deletions order by object_key")
        .fetch_all(db)
        .await
        .unwrap()
}

#[tokio::test]
async fn the_object_has_the_form_but_the_folder_may_be_an_old_address() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let bad = format!("update tenants set logo_key = 'logos/Outra/{TOKEN}.webp'");
    assert_eq!(
        rejected(exec(&db, &bad).await),
        rule(CHECK, "tenants_logo_key")
    );
    let bad = format!("update tournaments set cover_key = 'covers/quake-br/{TOKEN}.webp'");
    assert_eq!(
        rejected(exec(&db, &bad).await),
        rule(CHECK, "tournaments_cover_key")
    );
    // Objetos não se movem: a pasta é a do endereço de quando foi enviado.
    let old = format!("update tenants set logo_key = 'logos/outra/{TOKEN}.webp'");
    exec(&db, &old).await.unwrap();
    let old = format!("update tournaments set cover_key = 'covers/outra/outro/{TOKEN}.webp'");
    exec(&db, &old).await.unwrap();
    exec(&db, "update tenants set slug = 'nova'").await.unwrap();
    exec(&db, "update tournaments set slug = 'liga'")
        .await
        .unwrap();
}

#[tokio::test]
async fn deleting_the_tenant_queues_every_image() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let logo = format!("logos/quake-br/{TOKEN}.webp");
    let cover = format!("covers/quake-br/copa/{TOKEN}.jpg");
    let pending = "logos/quake-br/zyxwvutsrqpnmkjihgfedc.webp".to_owned();
    exec(&db, &format!("update tenants set logo_key = '{logo}'"))
        .await
        .unwrap();
    exec(
        &db,
        &format!("update tournaments set cover_key = '{cover}'"),
    )
    .await
    .unwrap();
    exec(
        &db,
        &format!(
            "insert into image_uploads (tenant_id, object_key, content_type, size, upload_expires_at)
             select id, '{pending}', 'image/webp', 1000, now() + interval '10 minutes' from tenants"
        ),
    )
    .await
    .unwrap();
    assert!(queued(&db).await.is_empty());

    exec(&db, "delete from tenants").await.unwrap();
    let mut all = vec![cover, logo, pending];
    all.sort();
    assert_eq!(queued(&db).await, all);
}
