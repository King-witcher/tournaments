use axum::{Json, extract::State, http::StatusCode};

use super::{
    changes, contest,
    db::{self, NewTournament},
    edit,
    schedule::Schedule,
    setup::Setup,
    slug::validate_slug,
    status::TournamentStatus,
    texts::Texts,
    types::{CreateTournament, Tournament, TournamentPath, TournamentSummary, UpdateTournament},
    visibility::find_visible,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, Role, TenantMember},
};

const NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments",
    tag = "tournaments",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = CreateTournament,
    responses(
        (status = CREATED, body = Tournament, description = "Torneio criado como rascunho"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.slug_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.slug_invalid, tournament.slug_reserved, tournament.name_invalid, tournament.description_invalid {max}, tournament.rules_invalid {max}, tournament.max_participants_invalid {min, max}, tournament.bans_invalid {max}, tournament.stage_count_invalid {max}, tournament.check_in_minutes_invalid {min, max}, tournament.contest_minutes_invalid {min, max}, format.invalid_best_of {best_of}, champions.pool_exhausted {required, available}, request.invalid_body"),
    )
)]
pub async fn create_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    JsonBody(body): JsonBody<CreateTournament>,
) -> Result<(StatusCode, Json<Tournament>), ApiError> {
    validate_slug(&body.slug)?;
    let texts = Texts::clean(&body.name, &body.description, &body.rules)?;
    let setup = Setup::from_dto(body.max_participants, body.champion_rules, body.stages);
    setup.check(body.mode, db::champion_pool(&state.db).await?, 0)?;
    let schedule = Schedule {
        starts_at: body.starts_at,
        check_in_minutes: body.check_in_minutes,
    };
    schedule.check()?;
    contest::check(body.contest_minutes)?;

    let new = NewTournament {
        slug: &body.slug,
        locale: body.locale,
        texts: &texts,
        mode: body.mode,
        visibility: body.visibility,
        setup: &setup,
        schedule,
        contest_minutes: body.contest_minutes,
    };
    let mut tx = state.db.begin().await?;
    let id = db::create(&mut tx, member.tenant, new).await?;
    changes::changed(&mut tx, id).await?;
    let created = db::find(&mut tx, member.tenant, &body.slug, Some(member.user.id))
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments",
    tag = "tournaments",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Vec<TournamentSummary>, description = "Torneios do tenant, do mais novo ao mais antigo. Rascunhos e privados só para organizer e owner; privados também para os inscritos"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn list_tournaments(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<Json<Vec<TournamentSummary>>, ApiError> {
    db::list(
        &state.db,
        member.tenant,
        member.user.id,
        member.role >= Role::Organizer,
    )
    .await
    .map(Json)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "O torneio, com fases e traduções"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found (inexistente; para player, também rascunho ou privado sem inscrição)"),
    )
)]
pub async fn get_tournament(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    let found = find_visible(&mut *state.db.acquire().await?, &member, &path.tournament).await?;
    Ok(Json(found.into()))
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
        (status = OK, body = Tournament, description = "Torneio atualizado; a checagem de campeões roda de novo se a configuração mudou"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.finished (textos, visibilidade ou janela de contestação), tournament.setup_locked (vagas, campeões ou fases depois de começar), tournament.schedule_locked (data de início ou janela de check-in depois de começar), tournament.max_participants_below_registered {registered}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "Os mesmos da criação, menos slug; tournament.starts_at_required (tirar a data fora do rascunho)"),
    )
)]
pub async fn update_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<UpdateTournament>,
) -> Result<Json<Tournament>, ApiError> {
    let viewer = Some(member.user.id);
    let mut tx = state.db.begin().await?;
    let current = db::find_for_update(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or(NOT_FOUND)?;
    let edit = edit::apply(
        current.status,
        &current.texts,
        &current.setup,
        current.schedule,
        current.contest_minutes,
        body,
    )?;
    if let Some(setup) = &edit.setup {
        let pool = db::champion_pool(&mut *tx).await?;
        setup.check(current.mode, pool, current.participants)?;
    }
    db::update(&mut tx, current.id, &edit).await?;
    changes::changed(&mut tx, current.id).await?;
    let updated = db::find(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok(Json(updated.into()))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/publish",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = Tournament, description = "Rascunho publicado: a inscrição abre e os membros passam a ver o torneio (o privado, só os inscritos). Se a janela de check-in já abriu, vem em check-in"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_draft"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tournament.starts_at_required, champions.pool_exhausted {required, available} (o pool mudou desde a criação)"),
    )
)]
pub async fn publish_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<Tournament>, ApiError> {
    let viewer = Some(member.user.id);
    let mut tx = state.db.begin().await?;
    let current = db::find_for_update(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or(NOT_FOUND)?;
    if current.status != TournamentStatus::Draft {
        return Err(ApiError::conflict("tournament.not_draft"));
    }
    if current.schedule.starts_at.is_none() {
        return Err(ApiError::invalid("tournament.starts_at_required"));
    }
    let pool = db::champion_pool(&mut *tx).await?;
    current
        .setup
        .check(current.mode, pool, current.participants)?;
    db::publish(&mut tx, current.id).await?;
    changes::changed(&mut tx, current.id).await?;
    let published = db::find(&mut tx, member.tenant, &path.tournament, viewer)
        .await?
        .ok_or_else(ApiError::internal)?;
    tx.commit().await?;
    Ok(Json(published.into()))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}",
    tag = "tournaments",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = NO_CONTENT, description = "Rascunho excluído, com os inscritos e os links de inscrição dele"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_draft (só rascunho pode ser excluído)"),
    )
)]
pub async fn delete_tournament(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let found = db::find(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(NOT_FOUND)?;
    // Antes de excluir: os inscritos e o endereço ainda existem.
    changes::changed(&mut tx, found.id).await?;
    if !db::delete_draft(&mut tx, found.id).await? {
        return Err(ApiError::conflict("tournament.not_draft"));
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
