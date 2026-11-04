//! As rotas do veto: "Pronto", o organizador forçar o começo, cada passo,
//! desfazer (o organizador, ou o jogador com o aceite do adversário). Todas
//! devolvem a série como ficou. O que é do lado (Pronto, veto de mapas, bans,
//! pick do duelo, pedir e responder o desfazer) é de qualquer jogador do
//! time: vale o primeiro clique. No pick por jogador, cada um escolhe os seus.

use axum::{Json, extract::State};
use engine::{
    core::ParticipantId,
    veto::{Event, Veto},
};
use sqlx::PgConnection;

use super::{
    Ctx,
    db::{self, NewVetoEvent},
    state::{Kind, MatchVeto, ev_side, other, slot},
    step,
    types::{UndoAnswer, VetoChoice, VetoSeen},
};
use crate::{
    AppState,
    matches::{
        actions, changes, detail, due,
        result::Side,
        stage::{Stage, Wrote},
        types::{MatchDetail, MatchPath},
    },
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::StoredTournament,
};

const NOT_PLAYER: ApiError = ApiError::forbidden("match.not_player");
const NOT_BEGUN: ApiError = ApiError::conflict("veto.not_begun");

/// O que toda ação do veto confere: torneio em andamento, partida pronta e
/// torneio com alguma fase de veto (409 `veto.off`).
fn open<'a>(
    tournament: &StoredTournament,
    stage: &'a Stage,
    key: &str,
) -> Result<(&'a engine::core::Match, (ParticipantId, ParticipantId)), ApiError> {
    actions::in_progress(tournament)?;
    let m = detail::find(stage, key)?;
    let sides = actions::ready(m)?;
    if !stage.veto_rules.has_phases() {
        return Err(ApiError::conflict("veto.off"));
    }
    Ok((m, sides))
}

/// O veto começado da partida. 409 `veto.not_begun`.
fn begun<'a>(stage: &'a Stage, m: &engine::core::Match) -> Result<&'a MatchVeto, ApiError> {
    stage
        .veto_of(m.id)
        .filter(|v| v.state.begun())
        .ok_or(NOT_BEGUN)
}

/// A vez que a tela mostrava (`step::seen`) ainda é a de agora.
fn seen(mv: &MatchVeto, step: Option<i64>) -> Result<(), ApiError> {
    step::seen(&mv.log, step)
}

/// Começa o veto: o seed melhor começa; `actor` é o organizador que forçou.
/// 422 `champions.pool_exhausted` `{required, available}` com campeões
/// ativos de menos para os bans da série.
async fn begin<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    tournament: &StoredTournament,
    stage: &Stage,
    m: &engine::core::Match,
    sides: (ParticipantId, ParticipantId),
    actor: Option<i64>,
) -> Result<(), ApiError> {
    let seed_of = |p: ParticipantId| stage.entrant(p).and_then(|e| e.seed).unwrap_or(i32::MAX);
    let first = if seed_of(sides.0) <= seed_of(sides.1) {
        Side::A
    } else {
        Side::B
    };
    let seed: u64 = rand::random();
    let ctx = Ctx::between(stage, m.id, sides);
    Veto::new(stage.veto_rules.clone(), m.best_of).apply(
        Event::Begin {
            seed,
            first: ev_side(first),
        },
        Some(&ctx.engine()),
    )?;
    let key = &crate::matches::key::format(m.id);
    let at = actions::at(member, tournament, stage, key);
    let event = NewVetoEvent {
        // O u64 gravado bit a bit num bigint.
        seed: Some(seed as i64),
        ..NewVetoEvent::new(Kind::Begin, Some(1), Some(first), actor)
    };
    db::insert(conn, &at, sides, &event).await?;
    changes::veto_started(conn, tournament, stage, key, sides).await
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/ready",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "O lado de quem clica está pronto (no TDM, qualquer jogador do time responde pelo time). Com os dois prontos, o veto começa (aviso `match.veto_started` para os dois); senão, o adversário recebe `match.opponent_ready`. Repetir não muda nada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off (torneio sem veto), veto.already_begun"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "champions.pool_exhausted `{required, available}` (com os dois prontos: campeões ativos de menos para os bans da série)"),
    )
)]
pub async fn veto_ready(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    let mv = stage.veto_of(m.id);
    if mv.is_some_and(|v| v.state.begun()) {
        return Err(ApiError::conflict("veto.already_begun"));
    }
    let ready = mv.map(|v| v.ready()).unwrap_or_default();
    if !ready[slot(side)] {
        let at = actions::at(&member, &tournament, &stage, &path.key);
        let event = NewVetoEvent::new(Kind::Ready, None, Some(side), Some(member.user.id));
        db::insert(&mut tx, &at, sides, &event).await?;
        if ready[slot(other(side))] {
            begin(&mut tx, &member, &tournament, &stage, m, sides, None).await?;
        } else {
            let (mine, to) = match side {
                Side::A => sides,
                Side::B => (sides.1, sides.0),
            };
            let name = stage
                .entrant(mine)
                .map_or_else(|| member.nickname.clone(), |t| t.display_name());
            changes::opponent_ready(&mut tx, &tournament, &stage, &path.key, to, &name).await?;
        }
    }
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Veto, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/begin",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "O organizador começa o veto sem esperar os dois prontos (aviso `match.veto_started` para os dois)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.already_begun"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "champions.pool_exhausted `{required, available}` (campeões ativos de menos para os bans da série)"),
    )
)]
pub async fn veto_begin(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    if stage.veto_of(m.id).is_some_and(|v| v.state.begun()) {
        return Err(ApiError::conflict("veto.already_begun"));
    }
    let actor = Some(member.user.id);
    begin(&mut tx, &member, &tournament, &stage, m, sides, actor).await?;
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Veto, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/steps",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    request_body = VetoChoice,
    responses(
        (status = OK, body = MatchDetail, description = "O passo do lado da vez: ban, pick ou voto de mapa, ou ban ou pick de campeão (o tipo é o da vez; o pick do duelo é para a partida que a vez diz). Qualquer jogador do time faz o passo do lado. No pick por jogador (`team_pick`), cada um escolhe o próprio campeão, uma vez por partida: na janela simultânea, para a partida que disser (`game`); na ordem alternada, se é do lado da vez, para a partida que a vez diz. Vale até 2 s depois do prazo que a série mostra. Quando o veto da partida acaba, ela pode ser enviada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.step_changed (veio `step` e a vez já não é essa: o tempo acabou ou o outro lado agiu), veto.finished (o veto da partida já acabou, ou a partida pedida já foi enviada), veto.not_your_turn (não é a vez do lado; no pick por jogador, também quem saiu da comunidade), veto.already_picked (já escolheu para esta partida), veto.champion_taken (outro escolheu o mesmo campeão para a partida antes: sem espelho, ou sem repetir no time)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "veto.option_invalid (mapa ou campeão fora das opções, já banido, a partida fora da série ou faltando na janela simultânea, ou o tipo errado para o passo), veto.champion_forbidden (a regra de não repetição o tira), request.invalid_body"),
    )
)]
pub async fn veto_step(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
    JsonBody(body): JsonBody<VetoChoice>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    let mv = begun(&stage, m)?;
    seen(mv, body.step)?;
    let choice = step::choice(&stage.map_pool, &stage.champions, &body)?;
    let turn = step::open_turn(&mv.state)?;
    let ctx = Ctx::of(&stage, mv);
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    let row = step::planned(
        &mv.state,
        &ctx,
        turn,
        side,
        member.user.id,
        choice,
        body.game,
    )?;
    let at = actions::at(&member, &tournament, &stage, &path.key);
    db::insert(&mut tx, &at, sides, &row).await?;
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Veto, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    request_body = Option<VetoSeen>,
    responses(
        (status = OK, body = MatchDetail, description = "O organizador desfaz o último passo da partida em andamento (inclusive o sorteado pelo fim do tempo): a vez volta para quem o fez, com o tempo inteiro. Partida enviada não se desfaz, nem o pick às cegas (`veto.nothing_to_undo`)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.step_changed (veio `step` e a vez já não é essa: o último passo não é o que a tela mostrava), veto.nothing_to_undo"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn veto_undo(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<MatchPath>,
    body: Option<JsonBody<VetoSeen>>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    let mv = begun(&stage, m)?;
    seen(mv, body.and_then(|JsonBody(b)| b.step))?;
    let game = undoable(mv)?;
    let at = actions::at(&member, &tournament, &stage, &path.key);
    let event = NewVetoEvent::new(Kind::Undo, Some(game), None, Some(member.user.id));
    db::insert(&mut tx, &at, sides, &event).await?;
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Veto, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

/// O jogo em andamento, se tem passo para desfazer. 409
/// `veto.nothing_to_undo`.
fn undoable(mv: &MatchVeto) -> Result<i16, ApiError> {
    match mv.state.game() {
        Some(game) if mv.state.can_undo() => Ok(i16::from(game)),
        _ => Err(ApiError::conflict("veto.nothing_to_undo")),
    }
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo-request",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    request_body = Option<VetoSeen>,
    responses(
        (status = OK, body = MatchDetail, description = "Um jogador (no TDM, qualquer um do time, pelo lado) pede para desfazer o último passo da partida em andamento; o adversário aceita ou recusa. Um passo novo, antes da resposta, cancela o pedido. O tempo do passo não para"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.step_changed (veio `step` e a vez já não é essa), veto.nothing_to_undo, veto.undo_requested (já há um pedido esperando resposta)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn veto_undo_request(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
    body: Option<JsonBody<VetoSeen>>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    let mv = begun(&stage, m)?;
    seen(mv, body.and_then(|JsonBody(b)| b.step))?;
    let game = undoable(mv)?;
    if mv.undo_request().is_some() {
        return Err(ApiError::conflict("veto.undo_requested"));
    }
    let at = actions::at(&member, &tournament, &stage, &path.key);
    let event = NewVetoEvent::new(
        Kind::UndoRequest,
        Some(game),
        Some(side),
        Some(member.user.id),
    );
    db::insert(&mut tx, &at, sides, &event).await?;
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Veto, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/undo-request/answer",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    request_body = UndoAnswer,
    responses(
        (status = OK, body = MatchDetail, description = "Um jogador do lado adversário ao que pediu aceita (o último passo é desfeito e a vez volta para quem o fez) ou recusa o pedido de desfazer"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, veto.own_request (o lado que pediu não responde)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.step_changed (veio `step` e a vez já não é essa), veto.no_undo_request"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn veto_undo_answer(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
    JsonBody(body): JsonBody<UndoAnswer>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    let mv = begun(&stage, m)?;
    seen(mv, body.step)?;
    let requested = mv
        .undo_request()
        .ok_or(ApiError::conflict("veto.no_undo_request"))?;
    if requested == side {
        return Err(ApiError::forbidden("veto.own_request"));
    }
    let game = undoable(mv)?;
    let at = actions::at(&member, &tournament, &stage, &path.key);
    let event = if body.accept {
        NewVetoEvent::new(Kind::Undo, Some(game), None, Some(member.user.id))
    } else {
        NewVetoEvent::new(
            Kind::UndoDeclined,
            Some(game),
            Some(side),
            Some(member.user.id),
        )
    };
    db::insert(&mut tx, &at, sides, &event).await?;
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Veto, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}
