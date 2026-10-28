use axum::{Json, extract::State};

use super::transition::{self, Transition};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{self, Tournament, TournamentPath},
};

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/check-in/open",
    tag = "lifecycle",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Check-in aberto antes da hora da abertura automática"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_registration, tournament.check_in_disabled"),
    )
)]
pub async fn open_check_in(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    run(&state, &member, &path.tournament, Transition::OpenCheckIn).await
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/registration/reopen",
    tag = "lifecycle",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Inscrição reaberta durante o check-in. Os check-ins feitos continuam. A abertura automática não reabre a mesma janela: o check-in volta pelo botão ou se a data de início ou a janela mudarem para uma abertura posterior"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_check_in"),
    )
)]
pub async fn reopen_registration(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    let reopen = Transition::ReopenRegistration;
    run(&state, &member, &path.tournament, reopen).await
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/start",
    tag = "lifecycle",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Torneio em andamento: inscrições e check-ins congelam. Entram os elegíveis (com check-in; sem check-in no torneio, todos), na ordem do seeding gravado; os sem seed vão para o fim e, se ninguém tem seed, a ordem é sorteada. Os seeds viram 1..N"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_published, tournament.check_in_not_open (com check-in, só a partir do check-in), tournament.already_started, tournament.not_enough_eligible {min, eligible}"),
    )
)]
pub async fn start(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    run(&state, &member, &path.tournament, Transition::Start).await
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/start",
    tag = "lifecycle",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Início desfeito (iniciou por engano): volta para o check-in, ou para a inscrição sem check-in. Inscrições, check-ins e seeds continuam como estavam, menos as de quem saiu da comunidade, que saem"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress"),
    )
)]
pub async fn undo_start(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    run(&state, &member, &path.tournament, Transition::UndoStart).await
}

/// Trava o torneio, aplica a transição e devolve o torneio como ficou.
async fn run(
    state: &AppState,
    member: &TenantMember<Organizer>,
    slug: &str,
    transition: Transition,
) -> Result<Json<Tournament>, ApiError> {
    let viewer = Some(member.user.id);
    let mut tx = state.db.begin().await?;
    let current = tournaments::find_for_update(&mut tx, member.tenant, slug, viewer)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))?;
    transition::apply(&mut tx, &current, transition).await?;
    let updated = tournaments::find(&mut tx, member.tenant, slug, viewer)
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok(Json(updated.into()))
}
