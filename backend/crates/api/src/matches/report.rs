//! O que os jogadores da partida fazem: um envia o placar e o adversário
//! confirma ou contesta (sem resposta, confirma sozinho no fim da janela).
//! O organizador também confirma: o pendente, ou o contestado (resolvendo
//! com o placar enviado).

use axum::{Json, extract::State};

use super::{
    actions, db,
    db::NewResult,
    detail, due,
    history::NewEvent,
    outcome,
    result::{Confirmation, ResultStatus},
    stage,
    types::{ContestReport, EventKind, MatchDetail, MatchPath, ReportScore},
};
use crate::{
    AppState, maps,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Role, TenantMember},
};

const NO_REPORT: ApiError = ApiError::conflict("match.no_report");
const NOT_PLAYER: ApiError = ApiError::forbidden("match.not_player");
const OWN_REPORT: ApiError = ApiError::forbidden("match.own_report");

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/report",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
    ),
    request_body = ReportScore,
    responses(
        (status = OK, body = MatchDetail, description = "Placar enviado por um jogador da partida: fica pendente até o adversário confirmar ou contestar; sem resposta, confirma sozinho no fim da janela de contestação. A chave só avança quando confirma"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player (quem envia não joga a partida)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.already_reported"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "match.invalid_score {best_of}, match.frags_invalid {max}, match.map_unknown, match.games_count_mismatch {expected, got}, match.map_repeated, match.game_tied {game}, match.games_disagree, request.invalid_body"),
    )
)]
pub async fn report(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
    JsonBody(body): JsonBody<ReportScore>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = due::lock_current(&mut tx, &member, &path.tournament).await?;
    let stage = stage::load(&mut tx, &tournament).await?;
    actions::in_progress(&tournament)?;
    let m = detail::find(&stage, &path.key)?;
    let sides = actions::ready(m)?;
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    if stage.result(m.id).is_some() {
        return Err(ApiError::conflict("match.already_reported"));
    }
    let maps = maps::active_ids(&mut tx, &actions::map_slugs(&body.games)).await?;
    let score = (body.a_wins, body.b_wins);
    let proposed = outcome::played(m.best_of, sides, score, &body.games, &maps)?;

    let at = actions::at(&member, &tournament, &stage, &path.key);
    let new = NewResult {
        best_of: m.best_of,
        sides,
        outcome: proposed.outcome,
        score: proposed.score,
        by: member.user.id,
    };
    let id = db::insert_report(&mut tx, &at, &new, side, tournament.contest_minutes).await?;
    db::replace_games(&mut tx, id, &proposed.games).await?;
    let event = NewEvent {
        kind: EventKind::Reported,
        actor: member.user.id,
        outcome: Some(proposed.outcome),
        comment: None,
    };
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/confirm",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "Resultado confirmado: pelo adversário de quem enviou, ou por organizer ou owner (no contestado, é a resolução que aceita o placar enviado). A chave avança e, se era a partida decisiva, o torneio encerra"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.own_report (quem enviou não confirma o próprio placar)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "match.no_report, match.already_confirmed, match.contested (espera o organizador)"),
    )
)]
pub async fn confirm(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = due::lock_current(&mut tx, &member, &path.tournament).await?;
    let stage = stage::load(&mut tx, &tournament).await?;
    let m = detail::find(&stage, &path.key)?;
    let r = stage.result(m.id).ok_or(NO_REPORT)?;
    let organizer = member.role >= Role::Organizer;
    let mine = stage
        .participant_of(member.user.id)
        .and_then(|p| r.side_of(p));
    let opponent = mine.is_some() && mine != r.reported_side;
    let (via, kind) = match r.status {
        ResultStatus::Confirmed => return Err(ApiError::conflict("match.already_confirmed")),
        ResultStatus::Pending if opponent => (Confirmation::Opponent, EventKind::Confirmed),
        ResultStatus::Contested if organizer => (Confirmation::Organizer, EventKind::Resolved),
        ResultStatus::Pending if organizer => (Confirmation::Organizer, EventKind::Confirmed),
        ResultStatus::Contested => return Err(ApiError::conflict("match.contested")),
        ResultStatus::Pending if mine.is_some() => return Err(OWN_REPORT),
        ResultStatus::Pending => return Err(NOT_PLAYER),
    };

    let at = actions::at(&member, &tournament, &stage, &path.key);
    db::confirm(&mut tx, &at, r.id, via, member.user.id).await?;
    let event = NewEvent {
        kind,
        actor: member.user.id,
        // A resolução registra o placar que passou a valer.
        outcome: (kind == EventKind::Resolved).then(|| r.outcome()),
        comment: None,
    };
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/contest",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1"),
    ),
    request_body = ContestReport,
    responses(
        (status = OK, body = MatchDetail, description = "O adversário de quem enviou contesta o placar, com comentário: o resultado sai da janela de contestação e espera o organizador"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.own_report"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.no_report, match.already_contested, match.already_confirmed (inclusive pela janela, que já acabou)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "match.comment_invalid, request.invalid_body"),
    )
)]
pub async fn contest(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
    JsonBody(body): JsonBody<ContestReport>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = due::lock_current(&mut tx, &member, &path.tournament).await?;
    let stage = stage::load(&mut tx, &tournament).await?;
    actions::in_progress(&tournament)?;
    let m = detail::find(&stage, &path.key)?;
    let r = stage.result(m.id).ok_or(NO_REPORT)?;
    match r.status {
        ResultStatus::Pending => {}
        ResultStatus::Contested => return Err(ApiError::conflict("match.already_contested")),
        ResultStatus::Confirmed => return Err(ApiError::conflict("match.already_confirmed")),
    }
    let mine = stage
        .participant_of(member.user.id)
        .and_then(|p| r.side_of(p))
        .ok_or(NOT_PLAYER)?;
    if Some(mine) == r.reported_side {
        return Err(OWN_REPORT);
    }
    let comment = actions::comment(&body.comment)?;

    let at = actions::at(&member, &tournament, &stage, &path.key);
    db::contest(&mut tx, r.id, member.user.id, &comment).await?;
    let event = NewEvent {
        kind: EventKind::Contested,
        actor: member.user.id,
        outcome: None,
        comment: Some(&comment),
    };
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}
