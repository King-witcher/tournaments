//! Handlers das traduções de um torneio: uma por idioma diferente do original.

use axum::{Json, extract::State, http::StatusCode};

use super::{
    db,
    texts::Texts,
    types::{PutTranslation, TournamentTranslation, TranslationPath},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
};

#[utoipa::path(
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("locale" = crate::shared::Locale, Path, description = "Idioma da tradução"),
    ),
    request_body = PutTranslation,
    responses(
        (status = OK, body = TournamentTranslation, description = "Tradução criada ou substituída"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path (idioma desconhecido)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.finished"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.translation_same_locale, tournament.name_invalid, tournament.description_invalid {max}, tournament.rules_invalid {max}, request.invalid_body"),
    )
)]
pub async fn put_translation(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TranslationPath>,
    JsonBody(body): JsonBody<PutTranslation>,
) -> Result<Json<TournamentTranslation>, ApiError> {
    let found = db::find(
        &mut *state.db.acquire().await?,
        member.tenant,
        &path.tournament,
    )
    .await?
    .ok_or(ApiError::not_found("tournament.not_found"))?;
    if !found.status.texts_editable() {
        return Err(ApiError::conflict("tournament.finished"));
    }
    if path.locale == found.locale {
        return Err(ApiError::invalid("tournament.translation_same_locale"));
    }
    let texts = Texts::clean(&body.name, &body.description, &body.rules)?;
    db::put_translation(&state.db, found.id, path.locale, &texts).await?;
    Ok(Json(TournamentTranslation {
        locale: path.locale,
        name: texts.name,
        description: texts.description,
        rules: texts.rules,
    }))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/translations/{locale}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("locale" = crate::shared::Locale, Path, description = "Idioma da tradução"),
    ),
    responses(
        (status = NO_CONTENT, description = "Tradução removida; o idioma volta a mostrar o original"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, tournament.translation_not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.finished"),
    )
)]
pub async fn delete_translation(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TranslationPath>,
) -> Result<StatusCode, ApiError> {
    let found = db::find(
        &mut *state.db.acquire().await?,
        member.tenant,
        &path.tournament,
    )
    .await?
    .ok_or(ApiError::not_found("tournament.not_found"))?;
    if !found.status.texts_editable() {
        return Err(ApiError::conflict("tournament.finished"));
    }
    if !db::delete_translation(&state.db, found.id, path.locale).await? {
        return Err(ApiError::not_found("tournament.translation_not_found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
