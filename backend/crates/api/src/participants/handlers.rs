use axum::{Json, extract::State, http::StatusCode};

use super::{
    db,
    registration::{self, Entry},
    types::{AddParticipant, Participant, ParticipantPath},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{self, TournamentPath, find_visible, find_visible_for_update},
};

const TOURNAMENT_NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/participants",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Vec<Participant>, description = "Inscritos, na ordem de inscrição (para quem vê o torneio)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
    )
)]
pub async fn list_participants(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Vec<Participant>>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let tournament = find_visible(&mut conn, &member, &path.tournament).await?;
    Ok(Json(db::list(&mut conn, tournament.id, None).await?))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/registration",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = NO_CONTENT, description = "Inscrito (também se já estava)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.private (só por convite ou pelo organizador), tournament.registration_closed, tournament.full {max}"),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    registration::register(
        &mut tx,
        member.tenant,
        &tournament,
        member.user.id,
        Entry::Member,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/registration",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = NO_CONTENT, description = "Inscrição cancelada (também se não havia)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.registration_closed (depois disso, só o organizador tira)"),
    )
)]
pub async fn withdraw(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    if !tournament.status.registration_open() {
        return Err(ApiError::conflict("tournament.registration_closed"));
    }
    if db::delete(&mut tx, tournament.id, member.user.id).await? {
        tournaments::changed_with(&mut tx, tournament.id, &[member.user.id]).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/participants",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = AddParticipant,
    responses(
        (status = CREATED, body = Participant, description = "Membro inscrito pelo organizador, em torneio público ou privado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, member.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "participant.already_registered, tournament.participants_locked (já começou), tournament.full {max}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn add_participant(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<AddParticipant>,
) -> Result<(StatusCode, Json<Participant>), ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = tournaments::find_for_update(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(TOURNAMENT_NOT_FOUND)?;
    let entry = Entry::Organizer(member.user.id);
    if !registration::register(&mut tx, member.tenant, &tournament, body.user_id, entry).await? {
        return Err(ApiError::conflict("participant.already_registered"));
    }
    let added = db::list(&mut tx, tournament.id, Some(body.user_id))
        .await?
        .pop()
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(added)))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("user_id" = i64, Path, description = "Id do usuário inscrito"),
    ),
    responses(
        (status = NO_CONTENT, description = "Inscrição removida pelo organizador"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, participant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.participants_locked (já começou)"),
    )
)]
pub async fn remove_participant(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<ParticipantPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = tournaments::find_for_update(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(TOURNAMENT_NOT_FOUND)?;
    if !tournament.status.participants_editable() {
        return Err(ApiError::conflict("tournament.participants_locked"));
    }
    if !db::delete(&mut tx, tournament.id, path.user_id).await? {
        return Err(ApiError::not_found("participant.not_found"));
    }
    tournaments::changed_with(&mut tx, tournament.id, &[path.user_id]).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
