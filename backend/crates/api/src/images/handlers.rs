use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use super::{
    db::{self, PendingUpload},
    object::Place,
    types::{
        CoverPath, CoverUploadPath, ImageSettings, ImageUpload, LogoUploadPath, NewImageUpload,
        PublicImage,
    },
    upload::{self, CONTENT_TYPES, MAX_BYTES, NOT_FOUND, Target},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams, Storage, public_url},
    tenants::{self, Organizer, Owner, TenantMember},
    tournaments::{self, StoredTournament, find_visible},
};

const NOT_CONFIGURED: ApiError =
    ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "images.not_configured");

fn storage(state: &AppState) -> Result<&Storage, ApiError> {
    state.public_storage.as_ref().ok_or(NOT_CONFIGURED)
}

fn logo(owner: &TenantMember<Owner>) -> Target<'_> {
    Target {
        tenant: owner.tenant,
        user: owner.user.id,
        place: Place::Logo {
            tenant: &owner.slug,
        },
        tournament: None,
    }
}

fn cover<'a>(member: &'a TenantMember<Organizer>, t: &'a StoredTournament) -> Target<'a> {
    Target {
        tenant: member.tenant,
        user: member.user.id,
        place: Place::Cover {
            tenant: &member.slug,
            tournament: &t.slug,
        },
        tournament: Some(t.id),
    }
}

/// O torneio `slug`, que a organização vê (rascunho também).
async fn tournament(
    state: &AppState,
    member: &TenantMember<Organizer>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let mut conn = state.db.acquire().await?;
    find_visible(&mut conn, member, slug).await
}

fn image(pending: &PendingUpload) -> Result<Json<PublicImage>, ApiError> {
    let url = public_url(&pending.object_key).ok_or_else(ApiError::internal)?;
    Ok(Json(PublicImage { url }))
}

#[utoipa::path(
    get,
    path = "/images/settings",
    tag = "images",
    responses(
        (status = OK, body = ImageSettings, description = "Se há logo e capa nesta API (o bucket público está configurado) e os limites de cada arquivo. Pública, sem sessão"),
    )
)]
pub async fn get_image_settings(State(state): State<AppState>) -> Json<ImageSettings> {
    Json(ImageSettings {
        enabled: state.public_storage.is_some(),
        max_bytes: MAX_BYTES,
        content_types: CONTENT_TYPES.to_vec(),
    })
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
    let storage = storage(&state)?;
    let upload = upload::request(&state.db, storage, &logo(&owner), body, Utc::now()).await?;
    Ok((StatusCode::CREATED, Json(upload)))
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
    PathParams(path): PathParams<LogoUploadPath>,
) -> Result<Json<PublicImage>, ApiError> {
    let storage = storage(&state)?;
    let pending = upload::verify(&state.db, storage, &logo(&owner), path.upload).await?;
    let mut tx = state.db.begin().await?;
    // O logo primeiro: o upload que sai já é dele e não vai para a fila.
    db::set_logo(&mut tx, owner.tenant, Some(&pending.object_key)).await?;
    if !db::delete_upload(&mut tx, pending.id).await? {
        return Err(NOT_FOUND);
    }
    tenants::changed(&mut tx, owner.tenant).await?;
    tx.commit().await?;
    image(&pending)
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
    let mut tx = state.db.begin().await?;
    if db::set_logo(&mut tx, owner.tenant, None).await? {
        tenants::changed(&mut tx, owner.tenant).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/cover/uploads",
    tag = "images",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = NewImageUpload,
    responses(
        (status = CREATED, body = ImageUpload, description = "URL para o navegador mandar a capa direto ao bucket (como a do logo); depois, registrar com `.../cover/uploads/{upload}/attach`. Organizer e owner, em qualquer estado do torneio"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.size_invalid {max} (bytes), request.invalid_body (tipo fora de image/jpeg)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "images.too_many_uploads {max}"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn create_cover_upload(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<CoverPath>,
    JsonBody(body): JsonBody<NewImageUpload>,
) -> Result<(StatusCode, Json<ImageUpload>), ApiError> {
    let storage = storage(&state)?;
    let t = tournament(&state, &member, &path.tournament).await?;
    let target = cover(&member, &t);
    let upload = upload::request(&state.db, storage, &target, body, Utc::now()).await?;
    Ok((StatusCode::CREATED, Json(upload)))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/cover/uploads/{upload}/attach",
    tag = "images",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("upload" = i64, Path, description = "O `id` do upload"),
    ),
    responses(
        (status = OK, body = PublicImage, description = "Upload conferido no bucket e gravado como a capa do torneio; a de antes sai do bucket na rotina diária"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, images.not_found (upload de outra pessoa, de outro torneio ou do logo, já registrado ou vencido)"),
        (status = CONFLICT, body = ErrorBody, description = "images.upload_missing, tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.upload_invalid (o upload é descartado)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn attach_cover(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<CoverUploadPath>,
) -> Result<Json<PublicImage>, ApiError> {
    let storage = storage(&state)?;
    let t = tournament(&state, &member, &path.tournament).await?;
    let pending = upload::verify(&state.db, storage, &cover(&member, &t), path.upload).await?;
    let mut tx = state.db.begin().await?;
    // A capa primeiro: o upload que sai já é dela e não vai para a fila.
    db::set_cover(&mut tx, member.tenant, t.id, Some(&pending.object_key)).await?;
    if !db::delete_upload(&mut tx, pending.id).await? {
        return Err(NOT_FOUND);
    }
    tournaments::changed(&mut tx, t.id).await?;
    tx.commit().await?;
    image(&pending)
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/cover",
    tag = "images",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = NO_CONTENT, description = "O torneio fica sem capa (o arquivo sai do bucket na rotina diária); sem capa, nada muda"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
    )
)]
pub async fn remove_cover(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<CoverPath>,
) -> Result<StatusCode, ApiError> {
    let t = tournament(&state, &member, &path.tournament).await?;
    let mut tx = state.db.begin().await?;
    if db::set_cover(&mut tx, member.tenant, t.id, None).await? {
        tournaments::changed(&mut tx, t.id).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
