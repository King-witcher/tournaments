//! As rotas do veto: "Pronto", o organizador forçar o começo, cada passo,
//! desfazer (o organizador, ou o jogador com o aceite do adversário). Todas
//! devolvem a partida como ficou.

use axum::{Json, extract::State};
use engine::{
    core::ParticipantId,
    veto::{Choice, Event, Turn},
};
use sqlx::PgConnection;

use super::{
    Ctx,
    db::{self, NewVetoEvent},
    state::{Kind, MatchVeto, ev_side, other, slot},
    types::{UndoAnswer, VetoChoice},
};
use crate::{
    AppState,
    matches::{
        actions, changes, detail, due,
        result::Side,
        stage::Stage,
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

/// Começa o veto: o seed melhor começa; `actor` é o organizador que forçou.
async fn begin<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    tournament: &StoredTournament,
    stage: &Stage,
    key: &str,
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
        (status = OK, body = MatchDetail, description = "O jogador está pronto. Com os dois prontos, o veto começa (aviso `match.veto_started` para os dois); senão, o adversário recebe `match.opponent_ready`. Repetir não muda nada"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off (torneio sem veto), veto.already_begun"),
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
    let ready = mv.map(MatchVeto::ready).unwrap_or_default();
    if !ready[slot(side)] {
        let at = actions::at(&member, &tournament, &stage, &path.key);
        let event = NewVetoEvent::new(Kind::Ready, None, Some(side), Some(member.user.id));
        db::insert(&mut tx, &at, sides, &event).await?;
        if ready[slot(other(side))] {
            begin(
                &mut tx,
                &member,
                &tournament,
                &stage,
                &path.key,
                sides,
                None,
            )
            .await?;
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
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
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
    begin(
        &mut tx,
        &member,
        &tournament,
        &stage,
        &path.key,
        sides,
        actor,
    )
    .await?;
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
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
        (status = OK, body = MatchDetail, description = "O passo de quem está na vez: ban, pick ou voto de mapa, ou ban ou pick de campeão (o tipo é o da vez). Quando o veto do jogo acaba, o mapa pode ser enviado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.finished (o veto do jogo já acabou), veto.not_your_turn"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "veto.option_invalid (mapa ou campeão fora das opções, já banido ou escolhido, ou o tipo errado para o passo), veto.champion_forbidden (a regra de não repetição o tira), request.invalid_body"),
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
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    let mv = begun(&stage, m)?;
    let choice = choice(&stage, &body)?;
    let (game, action) = match mv.state.turn() {
        Turn::Act { game, action, .. } => (game, action),
        Turn::Done { .. } => return Err(ApiError::conflict("veto.finished")),
        Turn::NotBegun => return Err(NOT_BEGUN),
    };
    let ctx = Ctx::of(&stage, mv);
    let event = Event::Step {
        side: ev_side(side),
        action,
        choice,
    };
    mv.state.clone().apply(event, Some(&ctx.engine()))?;

    let at = actions::at(&member, &tournament, &stage, &path.key);
    let (map, champion) = match choice {
        Choice::Map(m) => (Some(m), None),
        Choice::Champion(c) => (None, Some(c)),
    };
    let row = NewVetoEvent {
        map,
        champion,
        ..NewVetoEvent::new(
            Kind::of(action),
            Some(i16::from(game)),
            Some(side),
            Some(member.user.id),
        )
    };
    db::insert(&mut tx, &at, sides, &row).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}

/// O mapa (do pool do torneio) ou o campeão (ativo) do passo, pelo slug. 422
/// `veto.option_invalid` se não existe ou se não veio exatamente um.
fn choice(stage: &Stage, body: &VetoChoice) -> Result<Choice, ApiError> {
    const INVALID: ApiError = ApiError::invalid("veto.option_invalid");
    match (&body.map, &body.champion) {
        (Some(slug), None) => stage
            .map_pool
            .iter()
            .find(|m| &m.slug == slug)
            .map(|m| Choice::Map(m.id))
            .ok_or(INVALID),
        (None, Some(slug)) => stage
            .champions
            .iter()
            .find(|c| c.active && &c.slug == slug)
            .map(|c| Choice::Champion(c.id))
            .ok_or(INVALID),
        _ => Err(INVALID),
    }
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
    responses(
        (status = OK, body = MatchDetail, description = "O organizador desfaz o último passo do jogo em andamento (inclusive o sorteado pelo fim do tempo): a vez volta para quem o fez, com o tempo inteiro"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.nothing_to_undo"),
    )
)]
pub async fn veto_undo(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    let mv = begun(&stage, m)?;
    let game = undoable(mv)?;
    let at = actions::at(&member, &tournament, &stage, &path.key);
    let event = NewVetoEvent::new(Kind::Undo, Some(game), None, Some(member.user.id));
    db::insert(&mut tx, &at, sides, &event).await?;
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
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
    responses(
        (status = OK, body = MatchDetail, description = "Um jogador pede para desfazer o último passo do jogo em andamento; o adversário aceita ou recusa. Um passo novo, antes da resposta, cancela o pedido. O tempo do passo não para"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.nothing_to_undo, veto.undo_requested (já há um pedido esperando resposta)"),
    )
)]
pub async fn veto_undo_request(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MatchPath>,
) -> Result<Json<MatchDetail>, ApiError> {
    let mut tx = state.db.begin().await?;
    let (tournament, stage) = due::lock_stage(&mut tx, &member, &path.tournament).await?;
    let (m, sides) = open(&tournament, &stage, &path.key)?;
    let side = actions::side_of(&stage, &member, sides).ok_or(NOT_PLAYER)?;
    let mv = begun(&stage, m)?;
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
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
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
        (status = OK, body = MatchDetail, description = "O adversário de quem pediu aceita (o último passo é desfeito e a vez volta para quem o fez) ou recusa o pedido de desfazer"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "match.not_player, veto.own_request (quem pediu não responde)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, bracket.not_found, match.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, match.not_ready, match.series_decided, veto.off, veto.not_begun, veto.no_undo_request"),
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
    let detail = actions::respond(&mut tx, &member, tournament, &stage, &path.key).await?;
    tx.commit().await?;
    Ok(Json(detail))
}
