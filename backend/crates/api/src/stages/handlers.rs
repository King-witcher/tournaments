use axum::{Json, extract::State};
use engine::core::ParticipantId;
use sqlx::PgConnection;

use super::{
    rules,
    types::{StageEntry, StagePath, StageTeamPath},
};
use crate::{
    AppState, matches,
    outbox::{self, Event, Notice, NoticeKind},
    shared::{ApiError, ErrorBody, PathParams},
    teams,
    tenants::{self, Organizer, Role, TenantMember},
    tournaments::{self, StoredTournament, find_visible, find_visible_for_update},
};

/// A fase `number` como ficou, para a resposta das rotas que mudam.
async fn entry<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
    number: i16,
) -> Result<StageEntry, ApiError> {
    let tournament = find_visible(conn, member, slug).await?;
    let next = matches::next_stage(conn, &tournament, number).await?;
    rules::view(&tournament, number, next)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/stages/{stage}",
    tag = "stages",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("stage" = i16, Path, description = "A fase, de 2 em diante"),
    ),
    responses(
        (status = OK, body = StageEntry, description = "A fase: a entrada nela (corte e check-in), onde ela está, os candidatos (quem jogou a anterior, na ordem da classificação dela, com a presença e o seed com que entrariam) e se ela pode começar agora"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, stage.not_found (a fase 1, que nasce com o torneio, ou uma que não existe)"),
    )
)]
pub async fn get_stage(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<StagePath>,
) -> Result<Json<StageEntry>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let tournament = find_visible(&mut conn, &member, &path.tournament).await?;
    rules::stage_of(&tournament, path.stage)?;
    let next = matches::next_stage(&mut conn, &tournament, path.stage).await?;
    Ok(Json(rules::view(&tournament, path.stage, next)?))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/check-in/open",
    tag = "stages",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("stage" = i16, Path, description = "A fase, de 2 em diante"),
    ),
    responses(
        (status = OK, body = StageEntry, description = "Check-in da fase aberto: os jogadores dos candidatos que não desistiram são avisados (`stage.check_in_opened`, também no Discord). Fecha quando a fase começa"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, stage.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "stage.check_in_disabled (a fase não pede check-in), stage.already_started, tournament.not_in_progress, stage.previous_not_finished, stage.check_in_already_open"),
    )
)]
pub async fn open_stage_check_in(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<StagePath>,
) -> Result<Json<StageEntry>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    rules::stage_of(&tournament, path.stage)?;
    let next = matches::next_stage(&mut tx, &tournament, path.stage).await?;
    rules::check_open(&tournament, path.stage, &next)?;
    tournaments::open_stage_check_in(&mut tx, tournament.id, path.stage).await?;
    let players = next
        .candidates
        .iter()
        .filter(|c| !c.team.withdrawn)
        .flat_map(|c| c.team.user_ids());
    let audience = tournament.address();
    let notice = Notice {
        kind: NoticeKind::StageCheckInOpened {
            tournament: audience.name.clone(),
            stage: path.stage,
        },
        path: audience.path(),
    };
    outbox::emit(&mut tx, Event::notice(players, notice)).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    let entry = entry(&mut tx, &member, &path.tournament, path.stage).await?;
    tx.commit().await?;
    Ok(Json(entry))
}

/// O time `team` como candidato da fase `number`, para confirmar ou desfazer
/// a presença: com o check-in aberto (409 `stage.check_in_not_open`), um dos
/// que jogaram a anterior e não desistiram (409 `stage.not_candidate`) e
/// `member` jogador dele ou da organização (403 `stage.not_your_team`).
async fn candidate<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    tournament: &StoredTournament,
    number: i16,
    team: i64,
) -> Result<(), ApiError> {
    rules::check_in_open(tournament, number)?;
    let next = matches::next_stage(conn, tournament, number).await?;
    let found = next
        .candidates
        .iter()
        .find(|c| c.team.id == team && !c.team.withdrawn)
        .ok_or(ApiError::conflict("stage.not_candidate"))?;
    if member.role < Role::Organizer && !found.team.has(member.user.id) {
        return Err(ApiError::forbidden("stage.not_your_team"));
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/teams/{team}/check-in",
    tag = "stages",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("stage" = i16, Path, description = "A fase, de 2 em diante"),
        ("team" = i64, Path, description = "Id do time"),
    ),
    responses(
        (status = OK, body = StageEntry, description = "Presença do time confirmada na fase (por qualquer jogador dele, ou pela organização). Confirmar de novo não muda nada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "stage.not_your_team (nem jogador do time nem da organização)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, stage.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "stage.check_in_not_open (fechado: não abriu ou a fase começou), stage.not_candidate (não jogou a fase anterior, ou desistiu)"),
    )
)]
pub async fn check_in_team(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<StageTeamPath>,
) -> Result<Json<StageEntry>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    candidate(&mut tx, &member, &tournament, path.stage, path.team).await?;
    let at = (tournament.tenant, tournament.id);
    if teams::check_in_stage(&mut tx, at, path.stage, path.team, member.user.id).await? {
        tournaments::changed(&mut tx, tournament.id).await?;
    }
    let entry = entry(&mut tx, &member, &path.tournament, path.stage).await?;
    tx.commit().await?;
    Ok(Json(entry))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/teams/{team}/check-in",
    tag = "stages",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("stage" = i16, Path, description = "A fase, de 2 em diante"),
        ("team" = i64, Path, description = "Id do time"),
    ),
    responses(
        (status = OK, body = StageEntry, description = "Presença do time desfeita (pelo próprio time ou pela organização), até a fase começar"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "stage.not_your_team"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, stage.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "stage.check_in_not_open, stage.not_candidate"),
    )
)]
pub async fn undo_team_check_in(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<StageTeamPath>,
) -> Result<Json<StageEntry>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    candidate(&mut tx, &member, &tournament, path.stage, path.team).await?;
    if teams::undo_stage_check_in(&mut tx, tournament.id, path.stage, path.team).await? {
        tournaments::changed(&mut tx, tournament.id).await?;
    }
    let entry = entry(&mut tx, &member, &path.tournament, path.stage).await?;
    tx.commit().await?;
    Ok(Json(entry))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/start",
    tag = "stages",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("stage" = i16, Path, description = "A fase, de 2 em diante"),
    ),
    responses(
        (status = OK, body = StageEntry, description = "Fase iniciada: entram os que jogaram a anterior, sem os desistentes e, com check-in, só os que confirmaram, na ordem da classificação final dela, até o corte; os seeds 1..K ficam gravados, o check-in fecha e a primeira rodada nasce. Avisa os lados das séries prontas (`match.ready`) e o Discord (`stage.started`)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, stage.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "stage.already_started, tournament.not_in_progress, stage.previous_not_finished, stage.check_in_not_open, stage.not_enough_entrants {min, got}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "Os da configuração da fase com os times de verdade (com `params.stage`), champions.pool_exhausted {required, available} (o pool mudou), tournament.blind_needs_mirror"),
    )
)]
pub async fn start_stage(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<StagePath>,
) -> Result<Json<StageEntry>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    rules::stage_of(&tournament, path.stage)?;
    let next = matches::next_stage(&mut tx, &tournament, path.stage).await?;
    rules::check_start(&tournament, path.stage, &next)?;
    let order: Vec<i64> = next.entrants.iter().map(|p| p.0).collect();
    let at = (tournament.tenant, tournament.id);
    teams::save_stage_seeds(&mut tx, at, path.stage, &order).await?;
    tournaments::mark_stage_started(&mut tx, tournament.id, path.stage).await?;
    let seeded: Vec<ParticipantId> = next.entrants;
    matches::start_stage(&mut tx, &tournament, path.stage, &seeded).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    // Sai do que espera a organização ("fase N pronta para iniciar").
    tenants::organizing_changed(&mut tx, tournament.tenant).await?;
    let entry = entry(&mut tx, &member, &path.tournament, path.stage).await?;
    tx.commit().await?;
    Ok(Json(entry))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/stages/{stage}/start",
    tag = "stages",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("stage" = i16, Path, description = "A fase, de 2 em diante"),
    ),
    responses(
        (status = OK, body = StageEntry, description = "Início da fase desfeito, só sem nada jogado nela: os seeds e os pareamentos dela saem; os check-ins ficam (e o check-in, se tinha aberto, volta a valer)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, stage.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "stage.not_started, stage.next_started {stage} (a seguinte começou: desfaça antes o início dela), tournament.not_in_progress, stage.has_activity (algum evento de veto ou de partida, mapa enviado, resultado ou screenshot na fase, mesmo de mapa já anulado)"),
    )
)]
pub async fn undo_stage_start(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<StagePath>,
) -> Result<Json<StageEntry>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    rules::stage_of(&tournament, path.stage)?;
    if !tournament.stage_started(path.stage) {
        return Err(ApiError::conflict("stage.not_started"));
    }
    if tournament.stage_started(path.stage + 1) {
        return Err(ApiError::conflict("stage.next_started")
            .with_params([("stage", i64::from(path.stage + 1))]));
    }
    if tournament.status != tournaments::TournamentStatus::InProgress {
        return Err(ApiError::conflict("tournament.not_in_progress"));
    }
    matches::discard_stage(&mut tx, &tournament, path.stage).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    tenants::organizing_changed(&mut tx, tournament.tenant).await?;
    let entry = entry(&mut tx, &member, &path.tournament, path.stage).await?;
    tx.commit().await?;
    Ok(Json(entry))
}
