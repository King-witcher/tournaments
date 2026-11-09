//! SQL do logo e da capa da comunidade (`tenants.logo_key`,
//! `tenants.cover_key`), da capa do torneio (`tournaments.cover_key`), da
//! foto de perfil (`users.avatar_key`), dos uploads pedidos
//! (`image_uploads`, inclusive os das imagens das regras, que o registro passa
//! a `tournaments::register_rules_image`) e da fila do que sai do bucket público
//! (`image_deletions`, preenchida pelo banco a cada objeto que deixa de ser
//! usado).

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::{
    shared::{ApiError, ImageType},
    tenants::TenantId,
    tournaments::TournamentId,
};

/// Base das chaves do advisory lock dos uploads ("imag" e o id da pessoa);
/// longe das outras chaves do projeto.
const LOCK_BASE: i64 = 0x696d_6167_0000_0000;

/// De onde é um upload: o logo ou a capa da comunidade, a capa de um torneio,
/// a foto de perfil de quem pediu ou uma imagem das regras, pedida num torneio
/// e registrada na comunidade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Logo {
        tenant: TenantId,
    },
    TenantCover {
        tenant: TenantId,
    },
    TournamentCover {
        tenant: TenantId,
        tournament: TournamentId,
    },
    Avatar,
    Rules {
        tenant: TenantId,
        tournament: TournamentId,
    },
}

impl Scope {
    fn tenant(self) -> Option<i64> {
        match self {
            Self::Logo { tenant }
            | Self::TenantCover { tenant }
            | Self::TournamentCover { tenant, .. }
            | Self::Rules { tenant, .. } => Some(tenant.get()),
            Self::Avatar => None,
        }
    }

    /// O torneio da capa ou da imagem das regras; `None`: o que é da
    /// comunidade ou a foto.
    pub fn tournament(self) -> Option<i64> {
        match self {
            Self::TournamentCover { tournament, .. } | Self::Rules { tournament, .. } => {
                Some(tournament.get())
            }
            Self::Logo { .. } | Self::TenantCover { .. } | Self::Avatar => None,
        }
    }

    /// `image_uploads.kind`.
    fn kind(self) -> &'static str {
        match self {
            Self::Logo { .. } => "logo",
            Self::TenantCover { .. } => "tenant_cover",
            Self::TournamentCover { .. } => "tournament_cover",
            Self::Avatar => "avatar",
            Self::Rules { .. } => "rules",
        }
    }

    /// A conta dona da foto (`image_uploads.user_id`).
    fn avatar_of(self, user: i64) -> Option<i64> {
        matches!(self, Self::Avatar).then_some(user)
    }
}

/// Um upload pedido e ainda não registrado.
pub struct PendingUpload {
    pub id: i64,
    /// O torneio da capa ou da imagem das regras; `None`: o que é da
    /// comunidade ou a foto.
    pub tournament: Option<i64>,
    pub object_key: String,
    pub content_type: ImageType,
    pub size: i32,
    pub uploaded_by: Option<i64>,
}

/// O que um upload novo grava.
pub struct NewUpload<'a> {
    pub scope: Scope,
    pub object_key: &'a str,
    pub content_type: ImageType,
    pub size: u32,
    pub user: i64,
    pub expires_at: DateTime<Utc>,
}

/// Espera os pedidos de upload de `user` em outra transação e trava até o fim
/// desta: a contagem de `recent_uploads` vale até o insert.
pub async fn lock_uploads_of(conn: &mut PgConnection, user: i64) -> Result<(), sqlx::Error> {
    sqlx::query_scalar!(
        r#"select true as "locked!" from pg_advisory_xact_lock($1)"#,
        LOCK_BASE + user
    )
    .fetch_one(conn)
    .await?;
    Ok(())
}

/// Uploads de `user` pedidos depois de `since` e ainda não registrados.
pub async fn recent_uploads(
    conn: &mut PgConnection,
    user: i64,
    since: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "n!" from image_uploads where uploaded_by = $1 and created_at > $2"#,
        user,
        since
    )
    .fetch_one(conn)
    .await
}

pub async fn insert_upload(
    conn: &mut PgConnection,
    upload: &NewUpload<'_>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"insert into image_uploads
               (kind, tenant_id, tournament_id, user_id, object_key, content_type, size,
                uploaded_by, upload_expires_at)
           values ($9, $1, $2, $3, $4, $5, $6, $7, $8)
           returning id"#,
        upload.scope.tenant(),
        upload.scope.tournament(),
        upload.scope.avatar_of(upload.user),
        upload.object_key,
        upload.content_type.mime(),
        i32::try_from(upload.size).unwrap_or(i32::MAX),
        upload.user,
        upload.expires_at,
        upload.scope.kind()
    )
    .fetch_one(conn)
    .await
}

/// O upload `id` pendente do tipo de `scope` na comunidade (o logo, uma capa
/// ou uma imagem das regras) ou, com `Scope::Avatar`, a foto de `user`.
pub async fn find_upload(
    conn: &mut PgConnection,
    scope: Scope,
    user: i64,
    id: i64,
) -> Result<Option<PendingUpload>, ApiError> {
    let Some(r) = sqlx::query!(
        r#"select id, tournament_id, object_key, content_type, size, uploaded_by
           from image_uploads
           where id = $3 and kind = $4 and tenant_id is not distinct from $1
             and user_id is not distinct from $2"#,
        scope.tenant(),
        scope.avatar_of(user),
        id,
        scope.kind()
    )
    .fetch_optional(conn)
    .await?
    else {
        return Ok(None);
    };
    let content_type = ImageType::from_mime(&r.content_type).ok_or_else(|| {
        tracing::error!(
            value = r.content_type,
            "tipo de imagem desconhecido no banco"
        );
        ApiError::internal()
    })?;
    Ok(Some(PendingUpload {
        id: r.id,
        tournament: r.tournament_id,
        object_key: r.object_key,
        content_type,
        size: r.size,
        uploaded_by: r.uploaded_by,
    }))
}

/// Tira o upload `id` dos pendentes; falso se ele já não estava lá. O objeto
/// vai para a fila de exclusão se não for o logo, a capa nem a foto de ninguém.
pub async fn delete_upload(conn: &mut PgConnection, id: i64) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!("delete from image_uploads where id = $1", id)
        .execute(conn)
        .await?;
    Ok(done.rows_affected() > 0)
}

/// Troca o logo da comunidade por `key` (`None` tira); falso se nada mudou. O
/// de antes vai para a fila de exclusão.
pub async fn set_logo(
    conn: &mut PgConnection,
    tenant: TenantId,
    key: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "update tenants set logo_key = $2 where id = $1 and logo_key is distinct from $2",
        tenant.get(),
        key
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Troca a capa da comunidade por `key` (`None` tira); falso se nada mudou. A
/// de antes vai para a fila de exclusão.
pub async fn set_tenant_cover(
    conn: &mut PgConnection,
    tenant: TenantId,
    key: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "update tenants set cover_key = $2 where id = $1 and cover_key is distinct from $2",
        tenant.get(),
        key
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Troca a capa do torneio por `key` (`None` tira); falso se nada mudou. A de
/// antes vai para a fila de exclusão.
pub async fn set_tournament_cover(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    key: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"update tournaments set cover_key = $3
           where tenant_id = $1 and id = $2 and cover_key is distinct from $3"#,
        tenant.get(),
        tournament.get(),
        key
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Troca a foto de perfil por `key` (`None` tira); falso se nada mudou. A de
/// antes vai para a fila de exclusão.
pub async fn set_avatar(
    conn: &mut PgConnection,
    user: i64,
    key: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"update users set avatar_key = $2
           where id = $1 and deleted_at is null and avatar_key is distinct from $2"#,
        user,
        key
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Quando a rotina diária passou pela última vez.
pub async fn last_sweep(db: &PgPool) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!("select swept_at from image_sweeps")
        .fetch_optional(db)
        .await
}

pub async fn mark_swept(db: &PgPool, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into image_sweeps (swept_at) values ($1)
           on conflict (singleton) do update set swept_at = excluded.swept_at"#,
        now
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Apaga os uploads pedidos antes de `before` e nunca registrados (os
/// objetos vão para a fila de exclusão).
pub async fn delete_pending_before(db: &PgPool, before: DateTime<Utc>) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!("delete from image_uploads where created_at < $1", before)
        .execute(db)
        .await?;
    Ok(done.rows_affected())
}

/// Até `limit` objetos da fila de exclusão cuja URL de upload venceu antes de
/// `now`, os mais antigos primeiro.
pub async fn queued(
    db: &PgPool,
    limit: usize,
    now: DateTime<Utc>,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select object_key from image_deletions
           where upload_expires_at < $2
           order by queued_at, object_key limit $1"#,
        i64::try_from(limit).unwrap_or(i64::MAX),
        now
    )
    .fetch_all(db)
    .await
}

/// Tira da fila os objetos já apagados do bucket.
pub async fn dequeue(db: &PgPool, keys: &[String]) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from image_deletions where object_key = any($1)",
        keys
    )
    .execute(db)
    .await?;
    Ok(())
}
