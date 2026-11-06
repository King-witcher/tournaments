//! Os passos comuns ao logo, à capa e à foto de perfil: pedir a URL de upload
//! (com o limite por pessoa, que vale para os três juntos) e conferir no
//! bucket o upload que se quer registrar, antes da transação que o grava (o
//! HEAD é uma ida à rede).

use std::time::Duration;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{
    db::{self, NewUpload, PendingUpload, Scope},
    object::{self, Place},
    types::{ImageUpload, NewImageUpload},
};
use crate::shared::{ApiError, ImageType, Storage, storage::PUBLIC_CACHE};

/// Tamanho máximo de um logo, de uma capa ou de uma foto (o mesmo check está
/// na tabela).
pub const MAX_BYTES: u32 = 1024 * 1024;
/// Só JPEG: a prévia dos links do front (o `ImageResponse` do Next), que
/// desenha o logo e a capa, não lê WebP.
pub const CONTENT_TYPES: [ImageType; 1] = [ImageType::Jpeg];
/// Validade da URL de upload (a fila de exclusão conta com ela no banco).
const UPLOAD_EXPIRES: Duration = Duration::from_secs(10 * 60);
/// Uploads pedidos e não registrados por pessoa na última hora: o que passa
/// disso é abuso do bucket, não troca de imagem.
const MAX_PENDING_PER_HOUR: i64 = 10;

pub const NOT_FOUND: ApiError = ApiError::not_found("images.not_found");

/// De quem é a imagem e quem age.
pub struct Target<'a> {
    pub scope: Scope,
    pub user: i64,
    pub place: Place<'a>,
}

/// Grava o upload pendente e devolve a URL do PUT. Erros: 422
/// `request.invalid_body` (tipo fora de `CONTENT_TYPES`),
/// `images.size_invalid` `{max}`, 429 `images.too_many_uploads` `{max}`.
pub async fn request(
    db: &PgPool,
    storage: &Storage,
    target: &Target<'_>,
    body: NewImageUpload,
    now: DateTime<Utc>,
) -> Result<ImageUpload, ApiError> {
    if !CONTENT_TYPES.contains(&body.content_type) {
        return Err(ApiError::invalid("request.invalid_body"));
    }
    if !(1..=MAX_BYTES).contains(&body.size) {
        return Err(
            ApiError::invalid("images.size_invalid").with_params([("max", i64::from(MAX_BYTES))])
        );
    }
    let mut tx = db.begin().await?;
    db::lock_uploads_of(&mut tx, target.user).await?;
    let hour_ago = now - chrono::Duration::hours(1);
    if db::recent_uploads(&mut tx, target.user, hour_ago).await? >= MAX_PENDING_PER_HOUR {
        return Err(
            ApiError::new(StatusCode::TOO_MANY_REQUESTS, "images.too_many_uploads")
                .with_params([("max", MAX_PENDING_PER_HOUR)]),
        );
    }
    let kind = body.content_type;
    let key = object::new_key(target.place, kind);
    let expires_at = now + UPLOAD_EXPIRES;
    let upload = NewUpload {
        scope: target.scope,
        object_key: &key,
        content_type: kind,
        size: body.size,
        user: target.user,
        expires_at,
    };
    let id = db::insert_upload(&mut tx, &upload).await?;
    tx.commit().await?;
    let size = body.size.into();
    let url = storage.put_url(
        &key,
        kind.mime(),
        size,
        Some(PUBLIC_CACHE),
        UPLOAD_EXPIRES,
        now,
    );
    Ok(ImageUpload {
        id,
        url: url.into(),
        content_type: kind,
        cache_control: PUBLIC_CACHE.into(),
        expires_at,
    })
}

/// O upload `id` pendente de quem age, para a imagem de `target`, conferido
/// no bucket. Erros: 404 `images.not_found`, 409 `images.upload_missing`
/// (não chegou), 422 `images.upload_invalid` (chegou outro tipo ou tamanho:
/// o upload sai e o objeto vai para a fila de exclusão).
pub async fn verify(
    db: &PgPool,
    storage: &Storage,
    target: &Target<'_>,
    id: i64,
) -> Result<PendingUpload, ApiError> {
    let mut conn = db.acquire().await?;
    let tournament = target.scope.tournament();
    let pending = db::find_upload(&mut conn, target.scope, target.user, id)
        .await?
        .filter(|p| p.uploaded_by == Some(target.user) && p.tournament == tournament)
        .ok_or(NOT_FOUND)?;
    let info = storage.head(&pending.object_key).await.map_err(|e| {
        tracing::error!(error = %e, "imagens: HEAD no bucket falhou");
        ApiError::internal()
    })?;
    let info = info.ok_or(ApiError::conflict("images.upload_missing"))?;
    let expected = (
        Some(pending.content_type.mime()),
        u64::try_from(pending.size).ok(),
    );
    if (info.content_type.as_deref(), Some(info.size)) != expected {
        db::delete_upload(&mut conn, pending.id).await?;
        return Err(ApiError::invalid("images.upload_invalid"));
    }
    Ok(pending)
}
