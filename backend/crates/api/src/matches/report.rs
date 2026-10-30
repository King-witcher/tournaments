//! O que os jogadores da série fazem, partida a partida: um envia a partida
//! e o outro time confirma; qualquer um deles menos quem enviou contesta,
//! também depois de confirmada, até a próxima série de um dos lados começar
//! (`contest`). Sem resposta, a partida confirma sozinha no fim do prazo: a
//! janela de contestação depois do envio, ou depois da duração prevista da
//! partida, contada de quando ela foi liberada (`release`), o que vier por
//! último. O organizador também confirma, na hora: o pendente, ou o
//! contestado (resolvendo com o envio). A série fecha sozinha quando as
//! partidas confirmadas dão a maioria (`settle`).

use axum::{Json, extract::State};
use engine::series;

use super::{
    actions, changes, contest as rules, correction, db,
    db::{Deadline, NewGame},
    detail, due, evidence,
    games::{GameStatus, series_games},
    history::NewEvent,
    outcome, release,
    result::{Confirmation, Side},
    types::{ContestReport, EventKind, GameInput, GamePath, MatchDetail},
    veto,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Role, TenantMember},
};

const NOT_PLAYER: ApiError = ApiError::forbidden("match.not_player");
const OWN_REPORT: ApiError = ApiError::forbidden("match.own_report");

/// Duração prevista de uma partida, em minutos: o envio feito antes dela
/// conta o prazo do fim dela.
const GAME_MINUTES: i32 = 15;

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/report",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
        ("game" = i16, Path, description = "Número do mapa na série, a partir de 1"),
    ),
    request_body = GameInput,
    responses(
        (status = OK, body = MatchDetail, description = "Partida enviada por um jogador da série: fica pendente até o outro time confirmar ou alguém contestar; sem resposta, confirma sozinha no fim do prazo (`confirm_by`): a janela de contestação depois do envio ou depois de 15 minutos da partida liberada (veto dela concluído, partida anterior enviada ou série pronta), o que vier por último. A próxima já pode ser enviada. A série fecha quando as partidas confirmadas dão a maioria a um lado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player (quem envia não joga a partida)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided (pelos mapas enviados, a série já acabou), match.no_games_left {best_of}, match.game_already_reported, match.game_out_of_order {next}, match.veto_pending (o veto deste mapa ainda não acabou), match.screenshot_required (o torneio exige screenshot e o mapa não tem nenhum anexado)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "match.map_required, match.map_unknown, match.map_repeated, match.map_mismatch (outro mapa que o do veto), match.frags_invalid {min, max}, match.game_tied {game}, match.frags_disagree {game}, request.invalid_body"),
    )
)]
pub async fn report(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<GamePath>,
    JsonBody(body): JsonBody<GameInput>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    actions::in_progress(&tournament)?;
    let m = detail::find(&stage, &path.key)?;
    let sides = actions::ready(m)?;
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    let games = stage.games_of(m.id);
    if games.iter().any(|g| g.number == path.game) {
        return Err(ApiError::conflict("match.game_already_reported"));
    }
    let index = actions::game_index(path.game, games.len())?;
    let maps = actions::map_ids(&mut tx, &body).await?;
    let new: NewGame = outcome::game(path.game, sides, &body, &maps, true)?;
    veto::check_report(&stage, m, path.game, new.map)?;
    let candidate = series::SeriesGame {
        winner: body.winner.into(),
        status: series::GameStatus::Pending,
        map: new.map,
    };
    series::put(m.best_of, &series_games(&games), index, candidate)?;

    let at = actions::at(&member, &tournament, &stage, &path.key);
    evidence::require(
        &mut tx,
        state.storage.is_some(),
        &tournament,
        &at,
        path.game,
    )
    .await?;
    let started_at = db::started_at(&mut tx, tournament.id).await?;
    let deadline = Deadline {
        released: release::released_at(&stage, m, sides, path.game, started_at),
        game_minutes: GAME_MINUTES,
        contest_minutes: tournament.contest_minutes,
    };
    let confirm_by = db::insert_report(&mut tx, &at, &new, side, member.user.id, &deadline).await?;
    let (from, to) = match side {
        Side::A => sides,
        Side::B => (sides.1, sides.0),
    };
    if let (Some(from), Some(to)) = (stage.entrant(from), stage.entrant(to)) {
        let players = (from, to);
        changes::game_reported(
            &mut tx,
            &tournament,
            &path.key,
            path.game,
            players,
            confirm_by,
        )
        .await?;
    }
    let event = NewEvent::game(
        EventKind::Reported,
        Some(member.user.id),
        path.game,
        Some(new.winner),
    );
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
        ("game" = i16, Path, description = "Número do mapa na série, a partir de 1"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "Mapa confirmado como foi enviado: pelo adversário de quem enviou, ou por organizer ou owner na hora (no contestado, é a resolução que aceita o mapa enviado). Se os mapas confirmados dão a maioria, a série fecha, a chave avança e, se era a partida decisiva, o torneio encerra"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.own_report (o time de quem enviou não confirma)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found, match.game_not_found"),
        (status = CONFLICT, body = ErrorBody, description = "match.already_confirmed, match.contested (espera o organizador)"),
    )
)]
pub async fn confirm(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<GamePath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let m = detail::find(&stage, &path.key)?;
    let games = stage.games_of(m.id);
    let g = actions::game(&games, path.game)?;
    let organizer = member.role >= Role::Organizer;
    let mine = stage
        .participant_of(member.user.id)
        .and_then(|p| g.side_of(p));
    let opponent = mine.is_some() && mine != g.reported_side;
    let (via, kind) = match g.status {
        GameStatus::Confirmed => return Err(ApiError::conflict("match.already_confirmed")),
        GameStatus::Pending if opponent => (Confirmation::Opponent, EventKind::Confirmed),
        GameStatus::Contested if organizer => (Confirmation::Organizer, EventKind::Resolved),
        GameStatus::Pending if organizer => (Confirmation::Organizer, EventKind::Confirmed),
        GameStatus::Contested => return Err(ApiError::conflict("match.contested")),
        GameStatus::Pending if mine.is_some() => return Err(OWN_REPORT),
        GameStatus::Pending => return Err(NOT_PLAYER),
    };

    let at = actions::at(&member, &tournament, &stage, &path.key);
    db::confirm_game(&mut tx, g.id, via, member.user.id).await?;
    // A resolução registra o vencedor que passou a valer.
    let winner = (kind == EventKind::Resolved).then_some(g.winner);
    let event = NewEvent::game(kind, Some(member.user.id), g.number, winner);
    db::insert_event(&mut tx, &at, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/contest",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Série: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
        ("game" = i16, Path, description = "Número da partida na série, a partir de 1"),
    ),
    request_body = ContestReport,
    responses(
        (status = OK, body = MatchDetail, description = "Um jogador da série, menos quem enviou, contesta a partida, com comentário: ela sai do prazo e espera o organizador, e a próxima continua podendo ser enviada. Também a já confirmada pelo adversário ou pelo prazo, até a próxima série de um dos lados começar (veto começado ou partida enviada): se ela decidia a série, a série reabre e a chave é refeita"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.own_report (quem enviou não contesta)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found, match.game_not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.already_contested, match.contest_closed (confirmada e a contestação tardia não vale mais: outra série de um dos lados começou, a organização confirmou ou lançou, a série acabou em W.O., ou reabrir contradiz uma série decidida depois)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "match.comment_invalid, request.invalid_body"),
    )
)]
pub async fn contest(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<GamePath>,
    JsonBody(body): JsonBody<ContestReport>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    actions::in_progress(&tournament)?;
    let m = detail::find(&stage, &path.key)?;
    let games = stage.games_of(m.id);
    let g = actions::game(&games, path.game)?;
    stage
        .participant_of(member.user.id)
        .and_then(|p| g.side_of(p))
        .ok_or(NOT_PLAYER)?;
    if g.reported_by == Some(member.user.id) {
        return Err(OWN_REPORT);
    }
    let reopen = match g.status {
        GameStatus::Pending => None,
        GameStatus::Contested => return Err(ApiError::conflict("match.already_contested")),
        GameStatus::Confirmed => rules::late(&stage, m, g)?,
    };
    let comment = actions::comment(&body.comment)?;

    let at = actions::at(&member, &tournament, &stage, &path.key);
    db::contest_game(&mut tx, g.id, member.user.id, &comment).await?;
    let by = (member.user.id, member.nickname.as_str());
    changes::game_contested(&mut tx, &tournament, &path.key, g.number, by).await?;
    let event = NewEvent::game(EventKind::Contested, Some(member.user.id), g.number, None)
        .with_comment(Some(&comment));
    db::insert_event(&mut tx, &at, &event).await?;
    if let Some(change) = reopen {
        correction::void_stale(&mut tx, &member, &tournament, &stage, m.id, &change)
            .await
            .map_err(|e| {
                if e.code == correction::CONFLICTS.code {
                    rules::CLOSED
                } else {
                    e
                }
            })?;
        let series = (path.key.as_str(), m.best_of);
        correction::write(
            &mut tx,
            &member,
            &tournament,
            &stage,
            series,
            g.sides(),
            change,
        )
        .await?;
    }
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}
