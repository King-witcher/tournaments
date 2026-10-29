//! Screenshots do placar: nome do objeto com o dono, até 3 por mapa, só em
//! torneio iniciado, sem mudar de mapa nem de objeto; todo screenshot apagado,
//! por qualquer caminho, deixa o objeto na fila de exclusão; e o torneio guarda
//! quando encerrou.

use sqlx::PgPool;

use super::*;
use crate::common::*;
use crate::matches::{copa, players, single, start};

/// `quake-br` com a final `wb-1-1` iniciada; devolve o tenant e o torneio.
async fn started(db: &PgPool) -> (i64, i64) {
    let (app, ana, _) = quake_br(db).await;
    let p = players(&app, db, &ana, 2).await;
    start(&app, &ana, copa(single(3, 3)), &[&p[0], &p[1]]).await;
    sqlx::query_as("select tenant_id, id from tournaments")
        .fetch_one(db)
        .await
        .unwrap()
}

const TOKEN: &str = "abcdefghijkmnopqrstuvw";

/// O começo do nome de um screenshot da final `wb-1-1` da `copa` em `quake-br`.
const FINAL: &str = "screenshots/quake-br/copa/wb-1-1";

/// O nome de um screenshot do mapa `number`.
fn key(number: i16, token: &str, ext: &str) -> String {
    named(&format!("{FINAL}/mapa-{number}"), token, ext)
}

fn named(start: &str, token: &str, ext: &str) -> String {
    format!("{start}-{token}.{ext}")
}

fn insert(
    tenant: i64,
    tournament: i64,
    number: i16,
    object: &str,
    kind: &str,
    attached: bool,
) -> String {
    let attached_at = if attached { "now()" } else { "null" };
    format!(
        "insert into match_screenshots (tenant_id, tournament_id, stage, match_key, number, object_key,
             content_type, size, attached_at)
         values ({tenant}, {tournament}, 1, 'wb-1-1', {number}, '{object}', '{kind}', 100, {attached_at})"
    )
}

fn token(i: usize) -> String {
    // O último caractere varia (base58, sem o zero).
    format!("{}{}", &TOKEN[..21], char::from(b'1' + i as u8))
}

#[tokio::test]
async fn the_object_name_has_its_owner() {
    let (db, _lease) = crate::db::test_db().await;
    let (t, id) = started(&db).await;
    let webp = "image/webp";
    for bad in [
        named("screenshots/outra/copa/wb-1-1/mapa-1", TOKEN, "webp"),
        named("screenshots/quake-br/outro/wb-1-1/mapa-1", TOKEN, "webp"),
        named("screenshots/quake-br/copa/wb-1-2/mapa-1", TOKEN, "webp"),
        key(2, TOKEN, "webp"),
        key(1, "curto", "webp"),
        key(1, &TOKEN.replace('a', "0"), "webp"),
        format!("outro/{FINAL}/mapa-1-{TOKEN}.webp"),
    ] {
        assert_eq!(
            rejected(exec(&db, &insert(t, id, 1, &bad, webp, true)).await),
            rule(CHECK, "match_screenshots_object_key"),
            "{bad}"
        );
    }
    let png = key(1, TOKEN, "png");
    assert_eq!(
        rejected(exec(&db, &insert(t, id, 1, &png, "image/jpeg", true)).await),
        rule(CHECK, "match_screenshots_object_key")
    );
    assert_eq!(
        rejected(exec(&db, &insert(t, id, 1, &key(1, TOKEN, "jpg"), webp, true)).await),
        rule(CHECK, "match_screenshots_extension")
    );
    exec(
        &db,
        &insert(t, id, 1, &key(1, TOKEN, "jpg"), "image/jpeg", true),
    )
    .await
    .unwrap();
    // O mapa e o objeto não mudam; quem anexou pode sair.
    assert_eq!(
        rejected(exec(&db, "update match_screenshots set number = 2").await),
        rule(CHECK, "match_screenshots_immutable")
    );
    exec(&db, "update match_screenshots set uploaded_by = null")
        .await
        .unwrap();
}

#[tokio::test]
async fn three_attached_per_map() {
    let (db, _lease) = crate::db::test_db().await;
    let (t, id) = started(&db).await;
    for i in 0..3 {
        exec(
            &db,
            &insert(t, id, 1, &key(1, &token(i), "webp"), "image/webp", true),
        )
        .await
        .unwrap();
    }
    // Pendente não conta; anexar o quarto, sim.
    exec(
        &db,
        &insert(t, id, 1, &key(1, &token(3), "webp"), "image/webp", false),
    )
    .await
    .unwrap();
    assert_eq!(
        rejected(
            exec(
                &db,
                &insert(t, id, 1, &key(1, &token(4), "webp"), "image/webp", true)
            )
            .await
        ),
        rule(CHECK, "match_screenshots_limit")
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                "update match_screenshots set attached_at = now() where attached_at is null"
            )
            .await
        ),
        rule(CHECK, "match_screenshots_limit")
    );
    // Em outro mapa, de novo três.
    exec(
        &db,
        &insert(t, id, 2, &key(2, &token(5), "webp"), "image/webp", true),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn only_in_started_tournaments_and_undoing_the_start_clears_them() {
    let (db, _lease) = crate::db::test_db().await;
    let (t, id) = started(&db).await;
    exec(
        &db,
        &insert(t, id, 1, &key(1, &token(1), "webp"), "image/webp", true),
    )
    .await
    .unwrap();
    exec(&db, "update tournaments set status = 'registration'")
        .await
        .unwrap();
    assert_eq!(
        count(&db, "select count(*) from match_screenshots").await,
        0
    );
    assert_eq!(
        count(&db, "select count(*) from screenshot_deletions").await,
        1
    );
    assert_eq!(
        rejected(
            exec(
                &db,
                &insert(t, id, 1, &key(1, &token(2), "webp"), "image/webp", true)
            )
            .await
        ),
        rule(CHECK, "match_screenshots_started")
    );
}

#[tokio::test]
async fn every_deleted_screenshot_leaves_its_object_queued() {
    let (db, _lease) = crate::db::test_db().await;
    let (t, id) = started(&db).await;
    // Um mapa enviado com screenshot, e um screenshot no mapa 2, ainda não
    // enviado.
    let (a, b): (i64, i64) = sqlx::query_as("select min(id), max(id) from participants")
        .fetch_one(&db)
        .await
        .unwrap();
    let game = format!(
        "insert into match_games (tenant_id, tournament_id, stage, match_key, number, side_a,
             side_b, winner, status, reported_side, confirm_by)
         values ({t}, {id}, 1, 'wb-1-1', 1, {a}, {b}, {a}, 'pending', 'a', now() + interval '5 minutes')"
    );
    exec(&db, &game).await.unwrap();
    let first = key(1, &token(1), "webp");
    let second = key(2, &token(2), "webp");
    exec(&db, &insert(t, id, 1, &first, "image/webp", true))
        .await
        .unwrap();
    exec(&db, &insert(t, id, 2, &second, "image/webp", false))
        .await
        .unwrap();

    // Corrigir o mapa mantém o screenshot; anular (apagar a linha) leva.
    exec(&db, "update match_games set winner = side_b")
        .await
        .unwrap();
    assert_eq!(
        count(&db, "select count(*) from match_screenshots").await,
        2
    );
    exec(&db, "delete from match_games").await.unwrap();
    let left: Vec<String> = sqlx::query_scalar("select object_key from match_screenshots")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(left, std::slice::from_ref(&second));
    let queued: Vec<String> = sqlx::query_scalar("select object_key from screenshot_deletions")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(queued, std::slice::from_ref(&first));

    // A comunidade excluída leva o resto, que também entra na fila.
    exec(&db, "delete from tenants").await.unwrap();
    assert_eq!(
        count(&db, "select count(*) from match_screenshots").await,
        0
    );
    let mut queued: Vec<String> = sqlx::query_scalar("select object_key from screenshot_deletions")
        .fetch_all(&db)
        .await
        .unwrap();
    queued.sort();
    let mut both = vec![first, second];
    both.sort();
    assert_eq!(queued, both);
}

#[tokio::test]
async fn the_tournament_keeps_when_it_finished() {
    let (db, _lease) = crate::db::test_db().await;
    let (_, _) = started(&db).await;
    let finished_at = "select finished_at is not null from tournaments";
    let has = |db: PgPool| async move {
        sqlx::query_scalar::<_, bool>(finished_at)
            .fetch_one(&db)
            .await
            .unwrap()
    };
    assert!(!has(db.clone()).await);
    exec(&db, "update tournaments set status = 'finished'")
        .await
        .unwrap();
    assert!(has(db.clone()).await);
    // Mudar outra coisa não mexe; reabrir tira.
    let before: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("select finished_at from tournaments")
            .fetch_one(&db)
            .await
            .unwrap();
    exec(
        &db,
        "update tournaments set status = 'finished', name = 'Outra'",
    )
    .await
    .unwrap();
    let after: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("select finished_at from tournaments")
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(before, after);
    exec(&db, "update tournaments set status = 'in_progress'")
        .await
        .unwrap();
    assert!(!has(db.clone()).await);
    assert_eq!(
        rejected(exec(&db, "update tournaments set finished_at = now()").await),
        rule(CHECK, "tournaments_finished_at")
    );
}
