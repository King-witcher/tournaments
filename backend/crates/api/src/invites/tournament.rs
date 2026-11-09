//! Handlers do link de inscrição de um torneio, que é um por torneio e também
//! faz entrar no tenant. A prévia e o aceite são as rotas de sempre
//! (`/invites/{token}`).

use axum::{Json, extract::State};

use super::{db, types::InviteLink};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, PathParams},
    tenants::{self, Organizer, TenantMember},
    tournaments::{self, StoredTournament, TournamentPath},
};

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/invite",
    tag = "invites",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = InviteLink, description = "O link de inscrição, com qualquer tipo de inscrição: quem aceita entra no tenant como player e fica inscrito (com o check-in feito, se for durante o check-in). Só inscreve com a inscrição aberta"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
    )
)]
pub async fn get_tournament_invite_link(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<InviteLink>, ApiError> {
    let found = find(&state, &member, &path.tournament).await?;
    let token = db::tournament_token(&state.db, member.tenant, found.id).await?;
    Ok(Json(InviteLink { token }))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/invite/rotate",
    tag = "invites",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = InviteLink, description = "O link novo. O antigo deixa de valer (responde invite.not_found); quem já se inscreveu continua inscrito"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.registration_closed (só depois de publicar e até o início), tenant.deletion_scheduled"),
    )
)]
pub async fn rotate_tournament_invite_link(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<InviteLink>, ApiError> {
    let found = find(&state, &member, &path.tournament).await?;
    if !found.status.invites_allowed() {
        return Err(ApiError::conflict("tournament.registration_closed"));
    }
    let mut tx = state.db.begin().await?;
    let token = tenants::free_invite_code(&mut tx).await?;
    db::set_tournament_token(&mut tx, member.tenant, found.id, &token).await?;
    tournaments::changed(&mut tx, found.id).await?;
    tx.commit().await?;
    Ok(Json(InviteLink { token }))
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
