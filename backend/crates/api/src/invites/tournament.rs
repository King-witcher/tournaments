//! Handlers dos links de inscrição de um torneio. São convites do tenant que
//! também inscrevem no torneio; revogar, ver a prévia e aceitar são as rotas
//! de sempre (`/tenants/{slug}/invites/{id}` e `/invites/{token}`).

use axum::{Json, extract::State, http::StatusCode};

use super::{
    db,
    types::{CreatedTournamentInvite, TournamentInvite},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, PathParams, token},
    tenants::{Organizer, TenantMember},
    tournaments::{self, StoredTournament, TournamentPath},
};

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/invites",
    tag = "invites",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = CREATED, body = CreatedTournamentInvite, description = "Link multiuso de inscrição, público ou privado: quem aceita entra no tenant como player e fica inscrito. Vale enquanto a inscrição estiver aberta"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.registration_closed (só no rascunho ou com a inscrição aberta)"),
    )
)]
pub async fn create_tournament_invite(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<(StatusCode, Json<CreatedTournamentInvite>), ApiError> {
    let found = find(&state, &member, &path.tournament).await?;
    if !found.status.invites_allowed() {
        return Err(ApiError::conflict("tournament.registration_closed"));
    }
    let token = token::random_token();
    let mut tx = state.db.begin().await?;
    let id =
        db::create_for_tournament(&mut tx, member.tenant, found.id, &token, member.user.id).await?;
    tournaments::changed(&mut tx, found.id).await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedTournamentInvite { id, token }),
    ))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/invites",
    tag = "invites",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Vec<TournamentInvite>, description = "Links de inscrição não revogados, do mais novo ao mais antigo"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
    )
)]
pub async fn list_tournament_invites(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Vec<TournamentInvite>>, ApiError> {
    let found = find(&state, &member, &path.tournament).await?;
    Ok(Json(
        db::list_for_tournament(&state.db, member.tenant, found.id).await?,
    ))
}

/// O organizador vê todo torneio do tenant.
async fn find(
    state: &AppState,
    member: &TenantMember<Organizer>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    tournaments::find(&mut *state.db.acquire().await?, member.tenant, slug, None)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))
}
