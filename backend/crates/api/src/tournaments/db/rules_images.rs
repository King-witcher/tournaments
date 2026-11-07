//! As imagens das regras registradas na comunidade (`rules_images`) e quais
//! textos usam cada uma (`rules_image_refs`; `locale` nulo é o texto
//! original). O banco tira a imagem que deixa de ser usada pelo último texto e
//! põe o objeto na fila de exclusão.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::TournamentId;
use crate::{shared::Locale, tenants::TenantId};

/// Registra o objeto `key` (enviado e conferido) como imagem das regras da
/// comunidade; já registrado, nada muda.
pub async fn register_rules_image(
    conn: &mut PgConnection,
    tenant: TenantId,
    key: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into rules_images (tenant_id, object_key) values ($1, $2)
           on conflict (object_key) do nothing"#,
        tenant.get(),
        key
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Imagens registradas na comunidade que nenhum texto usa.
pub async fn unused_rules_images(
    conn: &mut PgConnection,
    tenant: TenantId,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "n!" from rules_images r
           where r.tenant_id = $1
             and not exists (select 1 from rules_image_refs f where f.image_id = r.id)"#,
        tenant.get()
    )
    .fetch_one(conn)
    .await
}

/// Apaga as imagens registradas antes de `before` que nenhum texto usa (os
/// objetos vão para a fila de exclusão). Pula a que um texto está gravando
/// agora (travada por `lock_rules_images`).
pub async fn delete_unused_rules_images(
    db: &PgPool,
    before: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        r#"delete from rules_images where id in (
               select r.id from rules_images r
               where r.created_at < $1
                 and not exists (select 1 from rules_image_refs f where f.image_id = r.id)
               for update skip locked)"#,
        before
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// Os ids das imagens registradas na comunidade com estes nomes, travados até
/// o fim da transação: o texto que deixa de usar uma delas espera a gravação
/// desta referência antes de contar quem ainda a usa.
pub async fn lock_rules_images(
    conn: &mut PgConnection,
    tenant: TenantId,
    keys: &[String],
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select id from rules_images
           where tenant_id = $1 and object_key = any($2)
           order by id for share"#,
        tenant.get(),
        keys
    )
    .fetch_all(conn)
    .await
}

/// Imagens distintas que o torneio passa a usar com `ids` no texto `locale`
/// (nulo: o original), contando as dos outros textos.
pub async fn rules_images_with(
    conn: &mut PgConnection,
    tournament: TournamentId,
    locale: Option<Locale>,
    ids: &[i64],
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "n!" from (
               select image_id from rules_image_refs
               where tournament_id = $1 and locale is distinct from $2
               union
               select unnest($3::bigint[])
           ) used"#,
        tournament.get(),
        locale.map(Locale::as_str),
        ids
    )
    .fetch_one(conn)
    .await
}

/// O texto `locale` do torneio (nulo: o original) passa a usar só `ids`. A
/// tradução já tem de estar gravada.
pub async fn set_rules_images(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    locale: Option<Locale>,
    ids: &[i64],
) -> Result<(), sqlx::Error> {
    let locale = locale.map(Locale::as_str);
    // Primeiro as novas: a que continua não fica sem texto no meio do caminho.
    if !ids.is_empty() {
        sqlx::query!(
            r#"insert into rules_image_refs (tenant_id, tournament_id, locale, image_id)
               select $1, $2, $3, unnest($4::bigint[])
               on conflict do nothing"#,
            tenant.get(),
            tournament.get(),
            locale,
            ids
        )
        .execute(&mut *conn)
        .await?;
    }
    sqlx::query!(
        r#"delete from rules_image_refs
           where tournament_id = $1 and locale is not distinct from $2 and image_id <> all($3)"#,
        tournament.get(),
        locale,
        ids
    )
    .execute(conn)
    .await?;
    Ok(())
}
