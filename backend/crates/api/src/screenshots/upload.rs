//! O registro de um upload, antes de travar o torneio: o upload pendente é de
//! quem pede e chegou ao bucket com o tipo e o tamanho pedidos (o HEAD é uma
//! ida à rede, que não segura o torneio travado). O que chegou diferente é
//! descartado (a linha sai e o objeto vai para a fila de exclusão).

use sqlx::PgPool;

use super::db::{self, StoredScreenshot};
use crate::{
    shared::{ApiError, Storage},
    tenants::TenantMember,
    tournaments::find_visible,
};

pub const NOT_FOUND: ApiError = ApiError::not_found("screenshots.not_found");

/// O upload `id` pendente de `member` no torneio `slug`, conferido no bucket.
/// Erros: 404 `screenshots.not_found`, 409 `screenshots.upload_missing` (não
/// chegou), 422 `screenshots.upload_invalid` (chegou outro tipo ou tamanho).
pub async fn verify<R>(
    db: &PgPool,
    storage: &Storage,
    member: &TenantMember<R>,
    slug: &str,
    id: i64,
) -> Result<StoredScreenshot, ApiError> {
    let mut conn = db.acquire().await?;
    let t = find_visible(&mut conn, member, slug).await?;
    let pending = db::find(&mut conn, member.tenant, t.id, id)
        .await?
        .filter(|p| p.attached_at.is_none() && p.uploaded_by == Some(member.user.id))
        .ok_or(NOT_FOUND)?;
    let info = storage.head(&pending.object_key).await.map_err(|e| {
        tracing::error!(error = %e, "screenshots: HEAD no bucket falhou");
        ApiError::internal()
    })?;
    let info = info.ok_or(ApiError::conflict("screenshots.upload_missing"))?;
    let expected = (
        Some(pending.content_type.mime()),
        u64::try_from(pending.size).ok(),
    );
    if (info.content_type.as_deref(), Some(info.size)) != expected {
        db::delete(&mut conn, pending.id).await?;
        return Err(ApiError::invalid("screenshots.upload_invalid"));
    }
    Ok(pending)
}
