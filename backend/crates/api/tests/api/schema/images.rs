//! O logo, as capas, a foto, as imagens das regras e a fila do que sai do
//! bucket público.

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
    let bad = format!("update tenants set cover_key = 'covers/quake-br/copa/{TOKEN}.jpg'");
    assert_eq!(
        rejected(exec(&db, &bad).await),
        rule(CHECK, "tenants_cover_key")
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

/// O upload da capa da comunidade não tem torneio, e o da capa do torneio tem;
/// o que não vira capa vai para a fila, e o que vira fica.
#[tokio::test]
async fn the_cover_upload_says_whose_it_is() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let upload = |kind: &str, tournament: &str, key: &str| {
        format!(
            "insert into image_uploads
                 (kind, tenant_id, tournament_id, object_key, content_type, size, upload_expires_at)
             select '{kind}', t.tenant_id, {tournament}, '{key}', 'image/jpeg', 1000,
                    now() + interval '10 minutes'
             from tournaments t"
        )
    };
    let community = format!("covers/quake-br/{TOKEN}.jpg");
    let of_tournament = format!("covers/quake-br/copa/{TOKEN}.jpg");
    for (kind, tournament, key, constraint) in [
        (
            "cover",
            "t.id",
            of_tournament.as_str(),
            "image_uploads_kind",
        ),
        (
            "tenant_cover",
            "t.id",
            community.as_str(),
            "image_uploads_scope",
        ),
        (
            "tournament_cover",
            "null",
            of_tournament.as_str(),
            "image_uploads_scope",
        ),
        (
            "tenant_cover",
            "null",
            of_tournament.as_str(),
            "image_uploads_object_key",
        ),
        (
            "tournament_cover",
            "t.id",
            community.as_str(),
            "image_uploads_object_key",
        ),
    ] {
        assert_eq!(
            rejected(exec(&db, &upload(kind, tournament, key)).await),
            rule(CHECK, constraint),
            "{kind} {tournament} {key}"
        );
    }

    exec(&db, &upload("tenant_cover", "null", &community))
        .await
        .unwrap();
    exec(
        &db,
        &format!("update tenants set cover_key = '{community}'"),
    )
    .await
    .unwrap();
    exec(&db, "delete from image_uploads").await.unwrap();
    assert!(queued(&db).await.is_empty());

    exec(&db, "update tenants set cover_key = null")
        .await
        .unwrap();
    assert_eq!(queued(&db).await, vec![community]);
}

/// A foto: sem pasta, só da conta não excluída; o upload dela não tem
/// comunidade e é de quem pediu.
#[tokio::test]
async fn the_photo_belongs_to_one_account() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, ana, _) = quake_br(&db).await;
    let id = ana.id;
    let bad =
        format!("update users set avatar_key = 'avatars/quake-br/{TOKEN}.jpg' where id = {id}");
    assert_eq!(
        rejected(exec(&db, &bad).await),
        rule(CHECK, "users_avatar_key")
    );
    let photo = format!("avatars/{TOKEN}.jpg");
    let set = format!("update users set avatar_key = '{photo}' where id = {id}");
    exec(&db, &set).await.unwrap();
    let deleted = format!("update users set deleted_at = now(), display_name = '' where id = {id}");
    assert_eq!(
        rejected(exec(&db, &deleted).await),
        rule(CHECK, "users_deleted_avatar")
    );

    let upload = |user: &str, tenant: &str, by: &str, key: &str| {
        let kind = if key.starts_with("avatars/") {
            "avatar"
        } else {
            "logo"
        };
        format!(
            "insert into image_uploads
                 (kind, user_id, tenant_id, uploaded_by, object_key, content_type, size,
                  upload_expires_at)
             select '{kind}', {user}, {tenant}, {by}, '{key}', 'image/jpeg', 1000,
                    now() + interval '10 minutes'
             from users u, tenants t where u.id = {id}"
        )
    };
    let pending = "avatars/zyxwvutsrqpnmkjihgfedc.jpg";
    let logo = format!("logos/quake-br/{TOKEN}.jpg");
    for (user, tenant, by, key) in [
        ("u.id", "t.id", "u.id", pending),
        ("u.id", "null", "null", pending),
        ("null", "null", "u.id", logo.as_str()),
    ] {
        assert_eq!(
            rejected(exec(&db, &upload(user, tenant, by, key)).await),
            rule(CHECK, "image_uploads_scope"),
            "{user} {tenant} {by}"
        );
    }
    let in_folder = format!("avatars/quake-br/{TOKEN}.jpg");
    assert_eq!(
        rejected(exec(&db, &upload("u.id", "null", "u.id", &in_folder)).await),
        rule(CHECK, "image_uploads_object_key")
    );
    exec(&db, &upload("u.id", "null", "u.id", pending))
        .await
        .unwrap();

    // Trocada, a de antes vai para a fila; o upload abandonado também.
    let unset = format!("update users set avatar_key = null where id = {id}");
    exec(&db, &unset).await.unwrap();
    exec(&db, "delete from image_uploads").await.unwrap();
    assert_eq!(queued(&db).await, vec![photo, pending.to_owned()]);
}

#[tokio::test]
async fn deleting_the_tenant_queues_every_image() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let logo = format!("logos/quake-br/{TOKEN}.webp");
    let cover = format!("covers/quake-br/copa/{TOKEN}.jpg");
    let community = format!("covers/quake-br/{TOKEN}.jpg");
    let pending = "logos/quake-br/zyxwvutsrqpnmkjihgfedc.webp".to_owned();
    exec(
        &db,
        &format!("update tenants set logo_key = '{logo}', cover_key = '{community}'"),
    )
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
            "insert into image_uploads
                 (kind, tenant_id, object_key, content_type, size, upload_expires_at)
             select 'logo', id, '{pending}', 'image/webp', 1000, now() + interval '10 minutes' from tenants"
        ),
    )
    .await
    .unwrap();
    assert!(queued(&db).await.is_empty());

    exec(&db, "delete from tenants").await.unwrap();
    let mut all = vec![cover, community, logo, pending];
    all.sort();
    assert_eq!(queued(&db).await, all);
}

/// A imagem das regras: da comunidade, só sai sem texto que a use, e o último
/// texto que deixa de usá-la (editado, tradução ou torneio apagados) a leva.
#[tokio::test]
async fn a_rules_image_lives_while_some_text_uses_it() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    create_tournament(&app, &ana, duel("liga")).await;
    let bad = format!(
        "insert into rules_images (tenant_id, object_key) select id, 'rules/{TOKEN}.jpg' from tenants"
    );
    assert_eq!(
        rejected(exec(&db, &bad).await),
        rule(CHECK, "rules_images_object_key")
    );
    // O upload pendente da imagem é pedido num torneio.
    let pending = format!(
        "insert into image_uploads (kind, tenant_id, object_key, content_type, size, upload_expires_at)
         select 'rules', id, 'rules/quake-br/{TOKEN}.jpg', 'image/jpeg', 1000, now() + interval '10 minutes'
         from tenants"
    );
    assert_eq!(
        rejected(exec(&db, &pending).await),
        rule(CHECK, "image_uploads_scope")
    );

    let image = format!("rules/quake-br/{TOKEN}.jpg");
    exec(
        &db,
        &format!(
            "insert into rules_images (tenant_id, object_key) select id, '{image}' from tenants"
        ),
    )
    .await
    .unwrap();
    exec(
        &db,
        "insert into tournament_translations (tournament_id, locale, name)
         select id, 'es', 'Copa' from tournaments where slug = 'copa'",
    )
    .await
    .unwrap();
    // O original e a tradução da copa e o original da liga.
    exec(
        &db,
        "insert into rules_image_refs (tenant_id, tournament_id, locale, image_id)
         select t.tenant_id, t.id, l.locale, r.id
         from tournaments t, rules_images r, (values (null), ('es')) l (locale)
         where t.slug = 'copa' or l.locale is null",
    )
    .await
    .unwrap();
    assert_eq!(
        rejected(exec(&db, "delete from rules_images").await),
        (
            FOREIGN_KEY.to_owned(),
            "rules_image_refs_tenant_id_image_id_fkey".to_owned()
        )
    );

    exec(&db, "delete from tournament_translations")
        .await
        .unwrap();
    exec(&db, "delete from rules_image_refs where locale is null and tournament_id = (select id from tournaments where slug = 'copa')")
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from rules_images").await, 1);
    assert!(queued(&db).await.is_empty());
    exec(&db, "delete from tournaments where slug = 'liga'")
        .await
        .unwrap();
    assert_eq!(count(&db, "select count(*) from rules_images").await, 0);
    assert_eq!(queued(&db).await, vec![image]);
}

/// A comunidade apagada leva as imagens das regras, usadas ou não.
#[tokio::test]
async fn deleting_the_tenant_queues_the_rules_images() {
    let (db, _lease) = crate::db::test_db().await;
    let (app, ana, _) = quake_br(&db).await;
    create_tournament(&app, &ana, duel("copa")).await;
    let used = format!("rules/quake-br/{TOKEN}.jpg");
    let unused = "rules/quake-br/zyxwvutsrqpnmkjihgfedc.jpg".to_owned();
    exec(
        &db,
        &format!(
            "insert into rules_images (tenant_id, object_key)
             select id, k from tenants, (values ('{used}'), ('{unused}')) v (k)"
        ),
    )
    .await
    .unwrap();
    exec(
        &db,
        &format!(
            "insert into rules_image_refs (tenant_id, tournament_id, image_id)
             select t.tenant_id, t.id, r.id from tournaments t, rules_images r
             where r.object_key = '{used}'"
        ),
    )
    .await
    .unwrap();
    exec(&db, "delete from tenants").await.unwrap();
    assert_eq!(count(&db, "select count(*) from rules_images").await, 0);
    let mut all = vec![used, unused];
    all.sort();
    assert_eq!(queued(&db).await, all);
}
