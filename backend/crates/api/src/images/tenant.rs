//! As imagens da comunidade, as duas só do owner: o logo (`/tenants/{slug}/logo`)
//! e a capa (`/tenants/{slug}/cover`). O caminho é o mesmo; muda onde o objeto
//! fica e a coluna que o guarda (`TenantImage`).

use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::PgConnection;

use super::{
    db::{self, Scope},
    handlers::{image, storage},
    object::Place,
    types::{ImageUpload, NewImageUpload, PublicImage, TenantUploadPath},
    upload::{self, NOT_FOUND, Target},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{self, Owner, TenantId, TenantMember},
};

#[derive(Clone, Copy)]
enum TenantImage {
    Logo,
    Cover,
}

impl TenantImage {
    fn target(self, owner: &TenantMember<Owner>) -> Target<'_> {
        let (tenant, slug) = (owner.tenant, owner.slug.as_str());
        let (scope, place) = match self {
            Self::Logo => (Scope::Logo { tenant }, Place::Logo { tenant: slug }),
            Self::Cover => (
                Scope::TenantCover { tenant },
                Place::TenantCover { tenant: slug },
            ),
        };
        Target {
            scope,
            user: owner.user.id,
            place,
        }
    }

    async fn set(
        self,
        conn: &mut PgConnection,
        tenant: TenantId,
        key: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        match self {
            Self::Logo => db::set_logo(conn, tenant, key).await,
            Self::Cover => db::set_tenant_cover(conn, tenant, key).await,
        }
    }
}

async fn request(
    state: &AppState,
    owner: &TenantMember<Owner>,
    kind: TenantImage,
    body: NewImageUpload,
) -> Result<(StatusCode, Json<ImageUpload>), ApiError> {
    let storage = storage(state)?;
    let upload = upload::request(&state.db, storage, &kind.target(owner), body, Utc::now()).await?;
    Ok((StatusCode::CREATED, Json(upload)))
}

async fn attach(
    state: &AppState,
    owner: &TenantMember<Owner>,
    kind: TenantImage,
    upload: i64,
) -> Result<Json<PublicImage>, ApiError> {
    let storage = storage(state)?;
    let pending = upload::verify(&state.db, storage, &kind.target(owner), upload).await?;
    let mut tx = state.db.begin().await?;
    // A imagem primeiro: o upload que sai já é dela e não vai para a fila.
    kind.set(&mut tx, owner.tenant, Some(&pending.object_key))
        .await?;
    if !db::delete_upload(&mut tx, pending.id).await? {
        return Err(NOT_FOUND);
    }
    tenants::changed(&mut tx, owner.tenant).await?;
    tx.commit().await?;
    image(&pending)
}

async fn remove(
    state: &AppState,
    owner: &TenantMember<Owner>,
    kind: TenantImage,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    if kind.set(&mut tx, owner.tenant, None).await? {
        tenants::changed(&mut tx, owner.tenant).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/logo/uploads",
    tag = "images",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = NewImageUpload,
    responses(
        (status = CREATED, body = ImageUpload, description = "URL para o navegador mandar o logo direto ao bucket (PUT com o Content-Type e o Cache-Control dados, `If-None-Match: *` e exatamente `size` bytes, uma vez, em 10 minutos); depois, registrar com `.../logo/uploads/{upload}/attach`. Só o owner"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.size_invalid {max} (bytes), request.invalid_body (tipo fora de image/jpeg)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "images.too_many_uploads {max} (uploads pedidos e não registrados na última hora)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured (a API está sem o bucket público)"),
    )
)]
pub async fn create_logo_upload(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<NewImageUpload>,
) -> Result<(StatusCode, Json<ImageUpload>), ApiError> {
    request(&state, &owner, TenantImage::Logo, body).await
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/logo/uploads/{upload}/attach",
    tag = "images",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("upload" = i64, Path, description = "O `id` do upload"),
    ),
    responses(
        (status = OK, body = PublicImage, description = "Upload conferido no bucket (tipo e tamanho pedidos) e gravado como o logo da comunidade; o de antes sai do bucket na rotina diária"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, images.not_found (upload de outra pessoa, de uma capa, já registrado ou vencido)"),
        (status = CONFLICT, body = ErrorBody, description = "images.upload_missing (o arquivo não chegou ao bucket), tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.upload_invalid (chegou outro tipo ou tamanho: o upload é descartado)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn attach_logo(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    PathParams(path): PathParams<TenantUploadPath>,
) -> Result<Json<PublicImage>, ApiError> {
    attach(&state, &owner, TenantImage::Logo, path.upload).await
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/logo",
    tag = "images",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = NO_CONTENT, description = "A comunidade fica sem logo (o arquivo sai do bucket na rotina diária); sem logo, nada muda. Só o owner"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
    )
)]
pub async fn remove_logo(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
) -> Result<StatusCode, ApiError> {
    remove(&state, &owner, TenantImage::Logo).await
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/cover/uploads",
    tag = "images",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = NewImageUpload,
    responses(
        (status = CREATED, body = ImageUpload, description = "URL para o navegador mandar a capa da comunidade direto ao bucket (como a do logo); depois, registrar com `.../cover/uploads/{upload}/attach`. Só o owner"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.size_invalid {max} (bytes), request.invalid_body (tipo fora de image/jpeg)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "images.too_many_uploads {max}"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn create_tenant_cover_upload(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<NewImageUpload>,
) -> Result<(StatusCode, Json<ImageUpload>), ApiError> {
    request(&state, &owner, TenantImage::Cover, body).await
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/cover/uploads/{upload}/attach",
    tag = "images",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("upload" = i64, Path, description = "O `id` do upload"),
    ),
    responses(
        (status = OK, body = PublicImage, description = "Upload conferido no bucket e gravado como a capa da comunidade; a de antes sai do bucket na rotina diária"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, images.not_found (upload de outra pessoa, do logo, já registrado ou vencido)"),
        (status = CONFLICT, body = ErrorBody, description = "images.upload_missing, tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.upload_invalid (o upload é descartado)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn attach_tenant_cover(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    PathParams(path): PathParams<TenantUploadPath>,
) -> Result<Json<PublicImage>, ApiError> {
    attach(&state, &owner, TenantImage::Cover, path.upload).await
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/cover",
    tag = "images",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = NO_CONTENT, description = "A comunidade fica sem capa (o arquivo sai do bucket na rotina diária); sem capa, nada muda. Só o owner"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
    )
)]
pub async fn remove_tenant_cover(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
) -> Result<StatusCode, ApiError> {
    remove(&state, &owner, TenantImage::Cover).await
}
