//! O registro de um upload, antes de travar o torneio: o upload pendente é de
//! quem pede e chegou ao bucket com o tipo e o tamanho pedidos (o HEAD é uma
//! ida à rede, que não segura o torneio travado). O que chegou diferente é
//! descartado (a linha sai e o objeto vai para a fila de exclusão).

use sqlx::PgPool;

use super::db::{self, StoredPrint};
use crate::{
    shared::{ApiError, Storage},
    tenants::TenantMember,
    tournaments::find_visible,
};

pub const NOT_FOUND: ApiError = ApiError::not_found("prints.not_found");

/// O upload `id` pendente de `member` no torneio `slug`, conferido no bucket.
/// Erros: 404 `prints.not_found`, 409 `prints.upload_missing` (não chegou),
/// 422 `prints.upload_invalid` (chegou outro tipo ou tamanho).
pub async fn verify<R>(
    db: &PgPool,
    storage: &Storage,
    member: &TenantMember<R>,
    slug: &str,
    id: i64,
) -> Result<StoredPrint, ApiError> {
    let mut conn = db.acquire().await?;
    let t = find_visible(&mut conn, member, slug).await?;
    let pending = db::find(&mut conn, member.tenant, t.id, id)
        .await?
        .filter(|p| p.attached_at.is_none() && p.uploaded_by == Some(member.user.id))
        .ok_or(NOT_FOUND)?;
    let info = storage.head(&pending.object_key).await.map_err(|e| {
        tracing::error!(error = %e, "prints: HEAD no bucket falhou");
        ApiError::internal()
    })?;
    let info = info.ok_or(ApiError::conflict("prints.upload_missing"))?;
    let expected = (
        Some(pending.content_type.mime()),
        u64::try_from(pending.size).ok(),
    );
    if (info.content_type.as_deref(), Some(info.size)) != expected {
        db::delete(&mut conn, pending.id).await?;
        return Err(ApiError::invalid("prints.upload_invalid"));
    }
    Ok(pending)
}
