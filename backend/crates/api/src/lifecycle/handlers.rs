use axum::{Json, extract::State};

use super::transition::{self, Transition};
use crate::{
    AppState, matches,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{self, Tournament, TournamentPath, UpdateTournament},
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
        (status = OK, body = Tournament, description = "Check-in aberto, e os inscritos avisados"),
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
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/check-in/open",
    tag = "lifecycle",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Check-in fechado (abriu por engano): volta para a inscrição, e os check-ins feitos ficam"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.check_in_closed"),
    )
)]
pub async fn close_check_in(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    run(&state, &member, &path.tournament, Transition::CloseCheckIn).await
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
        (status = OK, body = Tournament, description = "Início desfeito (iniciou por engano): volta para o check-in, ou para a inscrição sem check-in. Inscrições, check-ins e seeds continuam como estavam, menos as de quem desistiu ou saiu da comunidade, que saem"),
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

#[utoipa::path(
    patch,
    path = "/tenants/{slug}/tournaments/{tournament}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = UpdateTournament,
    responses(
        (status = OK, body = Tournament, description = "Torneio atualizado, e `slug` é o que vale; a checagem de campeões roda de novo se a configuração mudou. O endereço antigo fica livre na hora, sem redirecionamento. Tirar as fases que faltam com a atual já terminada encerra o torneio, com o pódio no Discord e o aviso de quem jogou"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.slug_locked (endereço depois de publicar), tournament.slug_taken, tournament.finished (regras, inscrição, janela de contestação ou screenshot obrigatório), tournament.setup_locked (modo, tamanho do time, vagas, campeões, veto ou conta para o rating depois de começar), tournament.stages_locked (depois do início: fase nova, ou fase já começada diferente; as que não começaram mudam e saem), tournament.schedule_locked (data de início ou check-in depois de começar), tournament.max_participants_below_registered {registered}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "Os mesmos da criação"),
    )
)]
pub async fn update_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<UpdateTournament>,
) -> Result<Json<Tournament>, ApiError> {
    let mut tx = state.db.begin().await?;
    let storage = state.storage.is_some();
    let cdn = tournaments::rules_images_cdn(&state);
    let updated =
        tournaments::update(&mut tx, &member, &path.tournament, body, storage, cdn).await?;
    if updated.finished {
        matches::announce_finished(&mut tx, &updated.tournament).await?;
    }
    tx.commit().await?;
    Ok(Json(updated.tournament.into()))
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
