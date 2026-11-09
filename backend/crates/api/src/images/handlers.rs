use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use super::{
    db::{self, PendingUpload, Scope},
    object::Place,
    types::{
        AvatarUploadPath, ImageSettings, ImageUpload, NewImageUpload, PublicImage,
        TournamentImagePath, TournamentUploadPath,
    },
    upload::{self, CONTENT_TYPES, MAX_BYTES, NOT_FOUND, Target},
};
use crate::{
    AppState,
    auth::CurrentUser,
    shared::{ApiError, ErrorBody, JsonBody, PathParams, Storage, public_base, public_url},
    tenants::{Organizer, TenantMember},
    tournaments::{self, StoredTournament, find_visible},
};

const NOT_CONFIGURED: ApiError =
    ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "images.not_configured");

pub(super) fn storage(state: &AppState) -> Result<&Storage, ApiError> {
    state.public_storage.as_ref().ok_or(NOT_CONFIGURED)
}

fn cover<'a>(member: &'a TenantMember<Organizer>, t: &'a StoredTournament) -> Target<'a> {
    Target {
        scope: Scope::TournamentCover {
            tenant: member.tenant,
            tournament: t.id,
        },
        user: member.user.id,
        place: Place::TournamentCover {
            tenant: &member.slug,
            tournament: &t.slug,
        },
    }
}

fn avatar(user: &CurrentUser) -> Target<'static> {
    Target {
        scope: Scope::Avatar,
        user: user.id,
        place: Place::Avatar,
    }
}

/// O torneio `slug`, que a organização vê (rascunho também).
pub(super) async fn tournament(
    state: &AppState,
    member: &TenantMember<Organizer>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let mut conn = state.db.acquire().await?;
    find_visible(&mut conn, member, slug).await
}

pub(super) fn image(pending: &PendingUpload) -> Result<Json<PublicImage>, ApiError> {
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
        rules_image_prefix: state
            .public_storage
            .as_ref()
            .and(public_base())
            .map(|base| format!("{base}/rules/")),
    })
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
pub async fn create_tournament_cover_upload(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentImagePath>,
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
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, images.not_found (upload de outra pessoa, de outro torneio ou da comunidade, já registrado ou vencido)"),
        (status = CONFLICT, body = ErrorBody, description = "images.upload_missing, tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.upload_invalid (o upload é descartado)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn attach_tournament_cover(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentUploadPath>,
) -> Result<Json<PublicImage>, ApiError> {
    let storage = storage(&state)?;
    let t = tournament(&state, &member, &path.tournament).await?;
    let pending = upload::verify(&state.db, storage, &cover(&member, &t), path.upload).await?;
    let mut tx = state.db.begin().await?;
    // A capa primeiro: o upload que sai já é dela e não vai para a fila.
    db::set_tournament_cover(&mut tx, member.tenant, t.id, Some(&pending.object_key)).await?;
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
pub async fn remove_tournament_cover(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentImagePath>,
) -> Result<StatusCode, ApiError> {
    let t = tournament(&state, &member, &path.tournament).await?;
    let mut tx = state.db.begin().await?;
    if db::set_tournament_cover(&mut tx, member.tenant, t.id, None).await? {
        tournaments::changed(&mut tx, t.id).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/me/avatar/uploads",
    tag = "images",
    request_body = NewImageUpload,
    responses(
        (status = CREATED, body = ImageUpload, description = "URL para o navegador mandar a foto de perfil direto ao bucket (como a do logo); depois, registrar com `/me/avatar/uploads/{upload}/attach`. Até lá, a foto de antes continua"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.size_invalid {max} (bytes), request.invalid_body (tipo fora de image/jpeg)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "images.too_many_uploads {max} (contando logos e capas)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn create_avatar_upload(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<NewImageUpload>,
) -> Result<(StatusCode, Json<ImageUpload>), ApiError> {
    let storage = storage(&state)?;
    let upload = upload::request(&state.db, storage, &avatar(&user), body, Utc::now()).await?;
    Ok((StatusCode::CREATED, Json(upload)))
}

#[utoipa::path(
    post,
    path = "/me/avatar/uploads/{upload}/attach",
    tag = "images",
    params(("upload" = i64, Path, description = "O `id` do upload")),
    responses(
        (status = OK, body = PublicImage, description = "Upload conferido no bucket e gravado como a foto de perfil, que passa a valer no lugar do avatar do Discord; a de antes sai do bucket na rotina diária"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "images.not_found (upload de outra pessoa ou de um logo ou capa, já registrado ou vencido)"),
        (status = CONFLICT, body = ErrorBody, description = "images.upload_missing (o arquivo não chegou ao bucket)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "images.upload_invalid (o upload é descartado)"),
        (status = SERVICE_UNAVAILABLE, body = ErrorBody, description = "images.not_configured"),
    )
)]
pub async fn attach_avatar(
    State(state): State<AppState>,
    user: CurrentUser,
    PathParams(path): PathParams<AvatarUploadPath>,
) -> Result<Json<PublicImage>, ApiError> {
    let storage = storage(&state)?;
    let pending = upload::verify(&state.db, storage, &avatar(&user), path.upload).await?;
    let mut tx = state.db.begin().await?;
    // A foto primeiro: o upload que sai já é dela e não vai para a fila.
    db::set_avatar(&mut tx, user.id, Some(&pending.object_key)).await?;
    if !db::delete_upload(&mut tx, pending.id).await? {
        return Err(NOT_FOUND);
    }
    tx.commit().await?;
    image(&pending)
}

#[utoipa::path(
    delete,
    path = "/me/avatar",
    tag = "images",
    responses(
        (status = NO_CONTENT, description = "A conta fica sem foto (o arquivo sai do bucket na rotina diária) e volta a mostrar o avatar do Discord, se houver; sem foto, nada muda"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn remove_avatar(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.db.acquire().await?;
    db::set_avatar(&mut conn, user.id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}
