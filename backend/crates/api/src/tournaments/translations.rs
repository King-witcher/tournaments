//! Handlers das traduções de um torneio: uma por idioma diferente do original.

use axum::{Json, extract::State, http::StatusCode};

use super::{
    changes, db,
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
        (status = OK, body = TournamentTranslation, description = "Tradução criada ou substituída; encerrado o torneio, só o nome e a descrição mudam"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path (idioma desconhecido)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.finished (regras diferentes das gravadas, com o torneio encerrado)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.translation_same_locale, tournament.name_invalid, tournament.description_invalid {max}, tournament.rules_invalid {max}, request.invalid_body"),
    )
)]
pub async fn put_translation(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TranslationPath>,
    JsonBody(body): JsonBody<PutTranslation>,
) -> Result<Json<TournamentTranslation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let found = db::find(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))?;
    if path.locale == found.locale {
        return Err(ApiError::invalid("tournament.translation_same_locale"));
    }
    let texts = Texts::clean(&body.name, &body.description, &body.rules)?;
    let saved_rules = found
        .translations
        .iter()
        .find(|t| t.locale == path.locale)
        .map_or("", |t| t.rules.as_str());
    if found.status.is_finished() && texts.rules != saved_rules {
        return Err(ApiError::conflict("tournament.finished"));
    }
    db::put_translation(&mut tx, found.id, path.locale, &texts).await?;
    changes::changed(&mut tx, found.id).await?;
    tx.commit().await?;
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
        (status = NO_CONTENT, description = "Tradução removida; o idioma volta a mostrar o original. Vale também com o torneio encerrado"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, tournament.translation_not_found"),
    )
)]
pub async fn delete_translation(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TranslationPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let found = db::find(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))?;
    if !db::delete_translation(&mut tx, found.id, path.locale).await? {
        return Err(ApiError::not_found("tournament.translation_not_found"));
    }
    changes::changed(&mut tx, found.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
