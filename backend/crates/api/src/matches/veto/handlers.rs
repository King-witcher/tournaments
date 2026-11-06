//! As rotas do veto: "Pronto" e desfazê-lo, o organizador forçar o começo,
//! cada passo, desfazer (o organizador, ou o jogador com o aceite do
//! adversário). Todas devolvem a série como ficou. Quem do time faz o que é do
//! lado é a regra do torneio (`acting`), e vale o primeiro clique. No pick por
//! jogador, cada um escolhe os seus, ou o capitão, os de todos. Pedir e responder
//! o desfazer é de quem pode fazer aquele tipo de passo.

use axum::{Json, extract::State};
use engine::core::ParticipantId;

use super::{
    Ctx, advance,
    db::{self, NewVetoEvent},
    state::{Kind, MatchVeto, other, slot},
    step,
    types::{UndoAnswer, VetoChoice, VetoSeen},
};
use crate::{
    AppState,
    matches::{
        Side, acting, actions, changes, detail, due,
        stage::{Stage, Wrote},
        types::{MatchDetail, MatchPath},
    },
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{StoredTournament, TeamPicker},
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

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/ready",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "Quem clica está pronto. Com tudo no capitão, só ele marca, e um basta por lado; senão, o lado fica pronto quando todos do elenco de agora marcaram. Com os dois lados prontos, o veto começa (aviso `match.veto_started`, para quem age nele); com o lado dele pronto pela primeira vez, quem falta do adversário recebe `match.opponent_ready` (depois de alguém desfazer, marcar de novo não avisa outra vez). Repetir não muda nada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.not_captain (tudo no capitão, e ele não é)"),
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
    let (m, sides, side) = may_mark(&tournament, &stage, &member, &path.key)?;
    let who = member.user.id;
    let before = stage.readiness(m.id, sides);
    if before.awaits(who) {
        let at = actions::at(&member, &tournament, &stage, &path.key);
        let event = NewVetoEvent::new(Kind::Ready, None, Some(side), Some(who));
        db::insert(&mut tx, &at, sides, &event).await?;
        let been_ready = before.been_ready[slot(side)];
        let after = before.without(who);
        if after.both() {
            advance::begin(&mut tx, &tournament, &stage, m, sides, None).await?;
        } else if after.side(side) && !been_ready {
            let (mine, _) = acting::of_side(side, sides);
            let name = stage
                .entrant(mine)
                .map_or_else(|| member.nickname.clone(), |t| t.display_name());
            let to = &after.waiting[slot(other(side))];
            changes::opponent_ready(&mut tx, &tournament, &path.key, to, &name).await?;
        }
    }
    let detail =
        actions::respond_to(&mut tx, &member, tournament, stage, Wrote::Veto, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

/// O que marcar e desfazer o "Pronto" conferem: a série aberta ao veto, quem
/// clica joga nela, o veto não começou e, com tudo no capitão, é ele. Devolve a
/// partida, os lados e o lado de quem clica.
fn may_mark<'a>(
    tournament: &StoredTournament,
    stage: &'a Stage,
    member: &TenantMember,
    key: &str,
) -> Result<
    (
        &'a engine::core::Match,
        (ParticipantId, ParticipantId),
        Side,
    ),
    ApiError,
> {
    let (m, sides) = open(tournament, stage, key)?;
    let side = actions::side_of(stage, member, sides).ok_or(NOT_PLAYER)?;
    if stage.veto_of(m.id).is_some_and(|v| v.state.begun()) {
        return Err(ApiError::conflict("veto.already_begun"));
    }
    acting::require_ready(stage, member.user.id)?;
    Ok((m, sides, side))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/matches/{match}/veto/ready",
    tag = "matches",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2"),
    ),
    responses(
        (status = OK, body = MatchDetail, description = "Quem tinha marcado \"Pronto\" desfaz, enquanto o veto não começou: o lado dele deixa de estar pronto. Ninguém é avisado, e marcar de novo não avisa o adversário outra vez. Sem marca, não muda nada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.not_captain (tudo no capitão, e ele não é)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off (torneio sem veto), veto.already_begun"),
    )
)]
pub async fn veto_unready(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides, side) = may_mark(&tournament, &stage, &member, &path.key)?;
    let who = member.user.id;
    if stage.readiness(m.id, sides).has_marked(who) {
        let at = actions::at(&member, &tournament, &stage, &path.key);
        let event = NewVetoEvent::new(Kind::Unready, None, Some(side), Some(who));
        db::insert(&mut tx, &at, sides, &event).await?;
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
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2"),
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
    advance::begin(&mut tx, &tournament, &stage, m, sides, actor).await?;
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
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2"),
    ),
    request_body = VetoChoice,
    responses(
        (status = OK, body = MatchDetail, description = "O passo do lado da vez: ban, pick ou voto de mapa, ou ban ou pick de campeão (o tipo é o da vez; o pick do duelo é para a partida que a vez diz). Quem do time faz o passo é a regra do torneio (`team_rules`). No pick por jogador (`team_pick`), cada um escolhe o próprio campeão, uma vez por partida, ou o capitão, o de cada colega (`player`): na janela simultânea, para a partida que disser (`game`); na ordem alternada, se é do lado da vez, para a partida que a vez diz. Vale até 2 s depois do prazo que a série mostra. Quando o veto da partida acaba, ela pode ser enviada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.not_captain (a regra do torneio deixa este passo com o capitão), veto.not_your_pick (cada jogador escolhe o próprio)"),
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
    let who = member.user.id;
    acting::require_turn(&stage, who, &mv.state.turn())?;
    let player = step::target_player(
        &mv.state,
        turn,
        (side, who),
        stage.team_rules.pick == TeamPicker::Captain,
        body.player,
        body.game,
    )?;
    let by = step::Acting {
        side,
        actor: who,
        player,
    };
    let row = step::planned(&mv.state, &ctx, turn, by, choice, body.game)?;
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
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2"),
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

/// 403 `match.not_captain`: quem pede ou responde o desfazer é quem pode fazer o
/// tipo de passo que ele tiraria.
fn may_undo(stage: &Stage, mv: &MatchVeto, who: i64) -> Result<(), ApiError> {
    match mv.last_step() {
        Some(action) => acting::require(stage, who, action),
        None => Ok(()),
    }
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
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2"),
    ),
    request_body = Option<VetoSeen>,
    responses(
        (status = OK, body = MatchDetail, description = "Um jogador que pode fazer aquele tipo de passo pela regra do torneio pede, pelo lado, para desfazer o último passo da partida em andamento; o adversário aceita ou recusa. Um passo novo, antes da resposta, cancela o pedido. O tempo do passo não para"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, match.not_captain (a regra deixa aquele tipo de passo com o capitão)"),
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
    may_undo(&stage, mv, member.user.id)?;
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
        ("match" = String, Path, description = "Partida: wb-1-3, lb-2-1, gf-1-1, rr-2-1, sw-3-2, pl-1-2"),
    ),
    request_body = UndoAnswer,
    responses(
        (status = OK, body = MatchDetail, description = "Um jogador do lado adversário ao que pediu, que pode fazer aquele tipo de passo pela regra do torneio, aceita (o último passo é desfeito e a vez volta para quem o fez) ou recusa o pedido de desfazer"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, veto.own_request (o lado que pediu não responde), match.not_captain (a regra deixa aquele tipo de passo com o capitão)"),
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
    may_undo(&stage, mv, member.user.id)?;
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
