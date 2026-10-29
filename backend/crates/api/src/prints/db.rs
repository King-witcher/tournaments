//! SQL dos prints (`match_prints`) e da fila do que sai do bucket
//! (`print_deletions`, preenchida pelo banco a cada print apagado).

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::object::ImageType;
use crate::{shared::ApiError, tenants::TenantId, tournaments::TournamentId};

/// Um mapa da série: torneio, fase, partida e número.
pub struct GameRef<'a> {
    pub tenant: TenantId,
    pub tournament: TournamentId,
    pub stage: i16,
    pub key: &'a str,
    pub number: i16,
}

/// Um print gravado (anexado ou só pedido).
pub struct StoredPrint {
    pub id: i64,
    pub stage: i16,
    pub match_key: String,
    pub number: i16,
    pub object_key: String,
    pub content_type: ImageType,
    pub size: i32,
    pub uploaded_by: Option<i64>,
    /// `None`: o upload foi pedido e ainda não foi registrado.
    pub attached_at: Option<DateTime<Utc>>,
}

impl StoredPrint {
    /// É do mapa `number` da partida `key` da fase `stage`.
    pub fn is_of(&self, stage: i16, key: &str, number: i16) -> bool {
        (self.stage, self.match_key.as_str(), self.number) == (stage, key, number)
    }
}

/// Um print anexado, com o apelido de quem anexou.
pub struct PrintRow {
    pub id: i64,
    pub number: i16,
    pub object_key: String,
    pub content_type: ImageType,
    pub uploaded_by: Option<i64>,
    pub nickname: Option<String>,
    pub attached_at: DateTime<Utc>,
}

fn image_type(value: &str) -> Result<ImageType, ApiError> {
    ImageType::from_mime(value).ok_or_else(|| {
        tracing::error!(value, "tipo de print desconhecido no banco");
        ApiError::internal()
    })
}

/// Prints anexados ao mapa.
pub async fn attached_count(conn: &mut PgConnection, at: &GameRef<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "n!" from match_prints
           where tenant_id = $1 and tournament_id = $2 and stage = $3 and match_key = $4
             and number = $5 and attached_at is not null"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        at.number
    )
    .fetch_one(conn)
    .await
}

/// Uploads de `user` no tenant pedidos desde `since` e ainda não registrados.
pub async fn pending_since(
    conn: &mut PgConnection,
    tenant: TenantId,
    user: i64,
    since: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "n!" from match_prints
           where tenant_id = $1 and uploaded_by = $2 and attached_at is null and created_at >= $3"#,
        tenant.get(),
        user,
        since
    )
    .fetch_one(conn)
    .await
}

/// O upload pedido por `user` para o mapa: a linha que o registro completa.
pub async fn insert_upload(
    conn: &mut PgConnection,
    at: &GameRef<'_>,
    object_key: &str,
    kind: ImageType,
    size: u32,
    user: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"insert into match_prints
               (tenant_id, tournament_id, stage, match_key, number, object_key, content_type,
                size, uploaded_by)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           returning id"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        at.number,
        object_key,
        kind.mime(),
        i32::try_from(size).unwrap_or(i32::MAX),
        user
    )
    .fetch_one(conn)
    .await
}

/// O print `id` do torneio.
pub async fn find(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    id: i64,
) -> Result<Option<StoredPrint>, ApiError> {
    let Some(row) = sqlx::query!(
        r#"select id, stage, match_key, number, object_key, content_type, size, uploaded_by,
                  attached_at
           from match_prints
           where tenant_id = $1 and tournament_id = $2 and id = $3"#,
        tenant.get(),
        tournament.get(),
        id
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };
    Ok(Some(StoredPrint {
        id: row.id,
        stage: row.stage,
        match_key: row.match_key,
        number: row.number,
        object_key: row.object_key,
        content_type: image_type(&row.content_type)?,
        size: row.size,
        uploaded_by: row.uploaded_by,
        attached_at: row.attached_at,
    }))
}

/// Registra o upload `id` conferido no bucket. `None` se já não estava
/// pendente (registrado em outra aba, ou apagado pela rotina).
pub async fn attach(
    conn: &mut PgConnection,
    id: i64,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"update match_prints set attached_at = now()
           where id = $1 and attached_at is null
           returning attached_at as "attached_at!""#,
        id
    )
    .fetch_optional(conn)
    .await
}

/// Apaga o print `id` (o banco põe o objeto na fila de exclusão).
pub async fn delete(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from match_prints where id = $1", id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Os prints anexados da partida, por mapa e na ordem em que foram anexados.
pub async fn of_match(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    stage: i16,
    key: &str,
) -> Result<Vec<PrintRow>, ApiError> {
    sqlx::query!(
        r#"select p.id, p.number, p.object_key, p.content_type, p.uploaded_by,
                  m.nickname as "nickname?", p.attached_at as "attached_at!"
           from match_prints p
           left join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.uploaded_by
           where p.tenant_id = $1 and p.tournament_id = $2 and p.stage = $3 and p.match_key = $4
             and p.attached_at is not null
           order by p.number, p.attached_at, p.id"#,
        tenant.get(),
        tournament.get(),
        stage,
        key
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|r| {
        Ok(PrintRow {
            id: r.id,
            number: r.number,
            object_key: r.object_key,
            content_type: image_type(&r.content_type)?,
            uploaded_by: r.uploaded_by,
            nickname: r.nickname,
            attached_at: r.attached_at,
        })
    })
    .collect()
}

/// Quando a rotina diária passou pela última vez.
pub async fn last_sweep(db: &PgPool) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!("select swept_at from print_sweeps")
        .fetch_optional(db)
        .await
}

pub async fn mark_swept(db: &PgPool, at: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into print_sweeps (swept_at) values ($1)
           on conflict (singleton) do update set swept_at = excluded.swept_at"#,
        at
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Apaga os prints dos torneios encerrados antes de `before`. Quantos saíram.
pub async fn delete_finished_before(
    db: &PgPool,
    before: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        r#"delete from match_prints p
           using tournaments t
           where t.id = p.tournament_id and t.status = 'finished' and t.finished_at < $1"#,
        before
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// Apaga os uploads pedidos antes de `before` e nunca registrados.
pub async fn delete_pending_before(db: &PgPool, before: DateTime<Utc>) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        "delete from match_prints where attached_at is null and created_at < $1",
        before
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// Até `limit` objetos da fila de exclusão, os mais antigos primeiro.
pub async fn queued(db: &PgPool, limit: usize) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "select object_key from print_deletions order by queued_at, object_key limit $1",
        i64::try_from(limit).unwrap_or(i64::MAX)
    )
    .fetch_all(db)
    .await
}

/// Tira da fila os objetos já apagados do bucket.
pub async fn dequeue(db: &PgPool, keys: &[String]) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from print_deletions where object_key = any($1)",
        keys
    )
    .execute(db)
    .await?;
    Ok(())
}
