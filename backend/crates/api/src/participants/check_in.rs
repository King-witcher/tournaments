//! Check-in dos inscritos, só com o torneio em check-in: o próprio inscrito
//! faz e desfaz, e o organizador faz e desfaz o de qualquer um. Quem não fez
//! continua inscrito, mas fica fora do seeding e da chave.

use axum::{extract::State, http::StatusCode};
use sqlx::PgConnection;

use super::{db, types::ParticipantPath};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{self, StoredTournament, TournamentPath, find_visible_for_update},
};

const PARTICIPANT_NOT_FOUND: ApiError = ApiError::not_found("participant.not_found");

/// 409 `tournament.check_in_closed` fora do check-in (antes de abrir, com a
/// inscrição reaberta ou depois de iniciar).
fn check_open(tournament: &StoredTournament) -> Result<(), ApiError> {
    if !tournament.status.check_in_open() {
        return Err(ApiError::conflict("tournament.check_in_closed"));
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/check-in",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = NO_CONTENT, description = "Check-in feito (também se já estava)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.check_in_closed, participant.not_registered"),
    )
)]
pub async fn check_in(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    check_open(&tournament)?;
    if !db::check_in(&mut tx, tournament.id, member.user.id, None).await? {
        return Err(ApiError::conflict("participant.not_registered"));
    }
    tournaments::changed(&mut tx, tournament.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/check-in",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = NO_CONTENT, description = "Check-in desfeito (também se não havia, ou se não está inscrito)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.check_in_closed"),
    )
)]
pub async fn undo_check_in(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    check_open(&tournament)?;
    db::undo_check_in(&mut tx, tournament.id, member.user.id).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}/check-in",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("user_id" = i64, Path, description = "Id do usuário inscrito"),
    ),
    responses(
        (status = NO_CONTENT, description = "Check-in marcado pelo organizador (também se já estava)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, participant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.check_in_closed"),
    )
)]
pub async fn check_in_participant(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<ParticipantPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = open_for_organizer(&mut tx, &member, &path.tournament).await?;
    let by = Some(member.user.id);
    if !db::check_in(&mut tx, tournament.id, path.user_id, by).await? {
        return Err(PARTICIPANT_NOT_FOUND);
    }
    tournaments::changed(&mut tx, tournament.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/participants/{user_id}/check-in",
    tag = "participants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("user_id" = i64, Path, description = "Id do usuário inscrito"),
    ),
    responses(
        (status = NO_CONTENT, description = "Check-in desmarcado pelo organizador (também se não havia)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, participant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.check_in_closed"),
    )
)]
pub async fn undo_check_in_participant(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<ParticipantPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = open_for_organizer(&mut tx, &member, &path.tournament).await?;
    if !db::undo_check_in(&mut tx, tournament.id, path.user_id).await? {
        return Err(PARTICIPANT_NOT_FOUND);
    }
    tournaments::changed(&mut tx, tournament.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// O torneio travado e em check-in, para o organizador mexer no check-in de
/// um inscrito (ele vê todo torneio do tenant).
async fn open_for_organizer(
    conn: &mut PgConnection,
    member: &TenantMember<Organizer>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let tournament = tournaments::find_for_update(conn, member.tenant, slug, None)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))?;
    check_open(&tournament)?;
    Ok(tournament)
}
