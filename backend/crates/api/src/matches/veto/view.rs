//! Veto refeito → o que a página da série mostra, do ponto de vista de quem
//! vê (as opções e os picks são só de quem joga e pode agir agora).

use std::borrow::Cow;

use engine::{
    core::{ChampionId, MapId, Match, MatchState, PlayerId},
    veto::{self as ev, Turn, Veto},
};

use super::{
    Ctx,
    state::{Kind, MatchVeto, VetoRow, api_side, ev_side},
    types::{
        ChampionRef, VetoAction, VetoBan, VetoChampion, VetoEvent, VetoEventKind, VetoGame,
        VetoGaveWay, VetoMapStep, VetoPickSlot, VetoPlayer, VetoStatus, VetoTarget, VetoTeamPick,
        VetoTurn, VetoUnavailable, VetoUnavailableReason, VetoView,
    },
};
use crate::matches::{result::Side, stage::Stage, types::MapRef};

/// Onde o veto da série pronta `m` está; `None` sem fase de veto no torneio
/// ou fora da série pronta.
pub fn status(stage: &Stage, m: &Match) -> Option<VetoStatus> {
    if !stage.veto_rules.has_phases() || !matches!(m.state, MatchState::Ready { .. }) {
        return None;
    }
    Some(match stage.veto_of(m.id).map(|v| v.state.turn()) {
        None | Some(Turn::NotBegun) => VetoStatus::WaitingReady,
        Some(Turn::Act { .. } | Turn::RostersUnknown { .. }) => VetoStatus::InProgress,
        Some(Turn::Done { .. }) => VetoStatus::Done,
    })
}

fn map_ref(stage: &Stage, id: MapId) -> MapRef {
    stage.map_pool.iter().find(|m| m.id == id).map_or_else(
        || MapRef {
            slug: String::new(),
            name: String::new(),
        },
        |m| MapRef {
            slug: m.slug.clone(),
            name: m.name.clone(),
        },
    )
}

fn champion_ref(stage: &Stage, id: ChampionId) -> ChampionRef {
    stage.champions.iter().find(|c| c.id == id).map_or_else(
        || ChampionRef {
            slug: String::new(),
            name: String::new(),
        },
        |c| ChampionRef {
            slug: c.slug.clone(),
            name: c.name.clone(),
        },
    )
}

const fn action(a: ev::Action) -> VetoAction {
    match a {
        ev::Action::Ban => VetoAction::Ban,
        ev::Action::Pick => VetoAction::Pick,
        ev::Action::Vote => VetoAction::Vote,
        ev::Action::TeamPick => VetoAction::TeamPick,
    }
}

/// O apelido de `player` (TDM): de quem está num time da chave, ou de quem
/// escolheu no veto e depois saiu do time.
fn nickname(stage: &Stage, mv: Option<&MatchVeto>, player: PlayerId) -> String {
    stage
        .entrants
        .iter()
        .flat_map(|t| &t.members)
        .find(|m| m.user_id == player.0)
        .map(|m| m.nickname.clone())
        .or_else(|| {
            mv?.rows
                .iter()
                .find(|r| r.player == Some(player.0))
                .and_then(|r| r.player_name.clone())
        })
        .unwrap_or_default()
}

/// O veto da série `m` (pronta ou decidida) para quem vê (`me`), do lado
/// `mine` se joga. `None` sem fase de veto no torneio, antes de a série
/// ficar pronta e na série decidida sem veto.
pub fn view(stage: &Stage, m: &Match, mine: Option<Side>, me: Option<i64>) -> Option<VetoView> {
    let mv = stage.veto_of(m.id);
    let ready_match = matches!(m.state, MatchState::Ready { .. });
    if !stage.veto_rules.has_phases() || !(ready_match || mv.is_some()) {
        return None;
    }
    let state: Cow<Veto> = match mv {
        Some(mv) => Cow::Borrowed(&mv.state),
        None => Cow::Owned(Veto::new(stage.veto_rules.clone(), m.best_of)),
    };
    let ctx = mv.map(|mv| Ctx::of(stage, mv));
    let turn = state.turn();
    let ready = mv.map(MatchVeto::ready).unwrap_or_default();
    let acting = match (&ctx, mine, me) {
        (Some(ctx), Some(side), Some(me)) if ready_match => Some((ctx, side, PlayerId(me))),
        _ => None,
    };
    let options = acting
        .map(|(ctx, side, _)| {
            state
                .options(ev_side(side), &ctx.engine())
                .into_iter()
                .map(|c| match c {
                    ev::Choice::Map(id) => map_ref(stage, id).slug,
                    ev::Choice::Champion(id) => champion_ref(stage, id).slug,
                })
                .collect()
        })
        .unwrap_or_default();
    let status = match turn {
        Turn::NotBegun => VetoStatus::WaitingReady,
        Turn::Act { .. } | Turn::RostersUnknown { .. } => VetoStatus::InProgress,
        Turn::Done { .. } => VetoStatus::Done,
    };
    Some(VetoView {
        map_mode: stage_mode(stage),
        champion_phase: stage.veto_rules.champion_phase,
        step_seconds: stage.veto_seconds,
        team_pick_seconds: stage.veto_rules.teams().then_some(stage.team_pick_seconds),
        pool: stage
            .map_pool
            .iter()
            .map(|p| map_ref(stage, p.id))
            .collect(),
        status,
        ready_a: ready[0],
        ready_b: ready[1],
        game: state.game().map(i16::from),
        turn: match &turn {
            Turn::Act {
                action: a,
                target,
                sides,
                pick_for,
                ..
            } if ready_match => Some(VetoTurn {
                action: action(*a),
                target: match target {
                    ev::Target::Map => VetoTarget::Map,
                    ev::Target::Champion => VetoTarget::Champion,
                },
                sides: sides.iter().copied().map(api_side).collect(),
                pick_for: pick_for.map(i16::from),
            }),
            _ => None,
        },
        deadline: mv
            .filter(|_| ready_match)
            .and_then(|mv| mv.deadline(stage.veto_seconds, stage.team_pick_seconds)),
        map_steps: state
            .map_steps()
            .iter()
            .filter_map(|s| match s.choice {
                ev::Choice::Map(id) => Some(VetoMapStep {
                    side: api_side(s.side),
                    action: action(s.action),
                    map: map_ref(stage, id),
                }),
                ev::Choice::Champion(_) => None,
            })
            .collect(),
        remaining: if stage.veto_rules.map_mode == ev::MapMode::Veto && state.begun() {
            state
                .remaining_maps()
                .into_iter()
                .map(|id| map_ref(stage, id))
                .collect()
        } else {
            Vec::new()
        },
        bans: bans(stage, mv, &state),
        games: (1..=state.best_of())
            .map(|g| game(stage, mv, &state, g))
            .collect(),
        options,
        picks: acting
            .map(|(ctx, side, me)| picks(stage, &state, &turn, ctx, side, me))
            .unwrap_or_default(),
        missing: if ready_match {
            missing(stage, mv, &state)
        } else {
            Vec::new()
        },
        champions: champions(stage, &state),
        undo_request: mv.and_then(MatchVeto::undo_request),
        can_undo: ready_match && state.can_undo(),
        reportable: ready_match && reportable(stage, m, &state),
        history: mv
            .map(|mv| mv.rows.iter().map(|r| event(stage, r)).collect())
            .unwrap_or_default(),
    })
}

/// Os jogadores podem enviar a próxima partida: o veto dela acabou (ou ela
/// já passou no veto, quando foi anulada para ser jogada de novo).
fn reportable(stage: &Stage, m: &Match, state: &Veto) -> bool {
    let next = stage.games_of(m.id).len() + 1;
    let Some(current) = state.game().map(usize::from) else {
        return false;
    };
    next < current || (next == current && matches!(state.turn(), Turn::Done { .. }))
}

fn stage_mode(stage: &Stage) -> crate::tournaments::MapMode {
    use crate::tournaments::MapMode;
    match stage.veto_rules.map_mode {
        ev::MapMode::Veto => MapMode::Veto,
        ev::MapMode::Vote => MapMode::Vote,
        ev::MapMode::Random => MapMode::Random,
        ev::MapMode::Off => MapMode::Off,
    }
}

/// A linha gravada do passo de campeão: o último pick de `player` (ou do
/// lado, no duelo) para a partida `game`, ou o ban do `champion`.
fn timed_out(mv: Option<&MatchVeto>, matches: impl Fn(&VetoRow) -> bool) -> bool {
    mv.and_then(|mv| mv.rows.iter().rev().find(|r| matches(r)))
        .is_some_and(|r| r.timeout)
}

fn bans(stage: &Stage, mv: Option<&MatchVeto>, state: &Veto) -> Vec<VetoBan> {
    state
        .bans()
        .iter()
        .filter_map(|s| {
            let c = s.champion()?;
            Some(VetoBan {
                side: api_side(s.side),
                champion: champion_ref(stage, c),
                timeout: timed_out(mv, |r| r.kind == Kind::Ban && r.champion == Some(c)),
            })
        })
        .collect()
}

fn game(stage: &Stage, mv: Option<&MatchVeto>, state: &Veto, g: u8) -> VetoGame {
    let picks = state.picks(g);
    let votes = state.votes(g);
    let slug = |m: Option<MapId>| m.map(|id| map_ref(stage, id).slug);
    VetoGame {
        game: i16::from(g),
        map: state.map_of(g).map(|id| map_ref(stage, id)),
        picked_by: state.picker(g).map(api_side),
        offered: state
            .offered(g)
            .into_iter()
            .map(|id| map_ref(stage, id))
            .collect(),
        vote_a: slug(votes[0]),
        vote_b: slug(votes[1]),
        pick_a: picks[0].map(|c| champion_ref(stage, c)),
        pick_b: picks[1].map(|c| champion_ref(stage, c)),
        team_picks: team_picks(stage, mv, state, g),
    }
}

/// TDM: a escolha que conta de cada jogador para a partida `g`, com o
/// sorteio do fim da janela marcado (pela linha gravada).
fn team_picks(stage: &Stage, mv: Option<&MatchVeto>, state: &Veto, g: u8) -> Vec<VetoTeamPick> {
    state
        .team_picks(g)
        .into_iter()
        .filter_map(|a| {
            let (Some(c), Some(player)) = (a.champion(), a.player) else {
                return None;
            };
            let timeout = timed_out(mv, |r| {
                r.kind == Kind::Pick
                    && r.pick_game == Some(i16::from(g))
                    && r.player == Some(player.0)
            });
            Some(VetoTeamPick {
                side: api_side(a.side),
                user_id: player.0,
                nickname: nickname(stage, mv, player),
                champion: champion_ref(stage, c),
                timeout,
            })
        })
        .collect()
}

/// TDM: quem falta na janela simultânea, com as partidas de cada um.
fn missing(stage: &Stage, mv: Option<&MatchVeto>, state: &Veto) -> Vec<VetoPlayer> {
    let mut players: Vec<VetoPlayer> = Vec::new();
    for m in state.missing() {
        match players.iter_mut().find(|p| p.user_id == m.player.0) {
            Some(p) => p.games.push(i16::from(m.game)),
            None => players.push(VetoPlayer {
                side: api_side(m.side),
                user_id: m.player.0,
                nickname: nickname(stage, mv, m.player),
                games: vec![i16::from(m.game)],
            }),
        }
    }
    players
}

/// Os picks que quem vê (`me`, do lado `side`) pode fazer agora: no duelo, o
/// da vez do lado dele; no TDM, cada partida em que ele falta.
fn picks(
    stage: &Stage,
    state: &Veto,
    turn: &Turn,
    ctx: &Ctx,
    side: Side,
    me: PlayerId,
) -> Vec<VetoPickSlot> {
    let side = ev_side(side);
    let (picker, games): (ev::Picker, Vec<u8>) = match turn {
        Turn::Act {
            action: ev::Action::Pick,
            target: ev::Target::Champion,
            sides,
            pick_for: Some(game),
            ..
        } if sides.contains(&side) => (ev::Picker { side, player: None }, vec![*game]),
        Turn::Act {
            action: ev::Action::TeamPick,
            ..
        } => (
            ev::Picker {
                side,
                player: Some(me),
            },
            state
                .missing()
                .into_iter()
                .filter(|m| m.side == side && m.player == me)
                .map(|m| m.game)
                .collect(),
        ),
        _ => return Vec::new(),
    };
    let ctx = ctx.engine();
    games
        .into_iter()
        .map(|game| {
            let options = state.pick_options(picker, game, &ctx);
            VetoPickSlot {
                game: i16::from(game),
                options: options
                    .champions
                    .iter()
                    .map(|c| champion_ref(stage, *c).slug)
                    .collect(),
                unavailable: state
                    .unavailable(picker, game, &ctx)
                    .into_iter()
                    .map(|(c, reason)| VetoUnavailable {
                        slug: champion_ref(stage, c).slug,
                        reason: match reason {
                            ev::Unavailable::Banned => VetoUnavailableReason::Banned,
                            ev::Unavailable::Teammate => VetoUnavailableReason::Teammate,
                            ev::Unavailable::Opponent => VetoUnavailableReason::Opponent,
                            ev::Unavailable::Repeat => VetoUnavailableReason::Repeat,
                        },
                    })
                    .collect(),
                gave_way: options.gave_way.map(|g| match g {
                    ev::GaveWay::Rule => VetoGaveWay::Rule,
                    ev::GaveWay::Game => VetoGaveWay::Game,
                }),
            }
        })
        .collect()
}

/// Os campeões ativos do torneio, com quem baniu cada um na série.
fn champions(stage: &Stage, state: &Veto) -> Vec<VetoChampion> {
    if !stage.veto_rules.champion_phase {
        return Vec::new();
    }
    let bans = state.bans();
    stage
        .champions
        .iter()
        .filter(|c| c.active)
        .map(|c| VetoChampion {
            slug: c.slug.clone(),
            name: c.name.clone(),
            banned_by: bans
                .iter()
                .find(|b| b.champion() == Some(c.id))
                .map(|b| api_side(b.side)),
        })
        .collect()
}

fn event(stage: &Stage, r: &VetoRow) -> VetoEvent {
    VetoEvent {
        kind: match r.kind {
            Kind::Ready => VetoEventKind::Ready,
            Kind::Begin => VetoEventKind::Begin,
            Kind::Game => VetoEventKind::Game,
            Kind::Ban => VetoEventKind::Ban,
            Kind::Pick => VetoEventKind::Pick,
            Kind::Vote => VetoEventKind::Vote,
            Kind::Undo => VetoEventKind::Undo,
            Kind::UndoRequest => VetoEventKind::UndoRequest,
            Kind::UndoDeclined => VetoEventKind::UndoDeclined,
        },
        game: r.game,
        pick_game: r.pick_game,
        side: r.side,
        map: r.map.map(|id| map_ref(stage, id)),
        champion: r.champion.map(|id| champion_ref(stage, id)),
        timeout: r.timeout,
        actor: r.actor.and(r.actor_name.clone()),
        player: r.player.map(|_| r.player_name.clone().unwrap_or_default()),
        at: r.created_at,
    }
}
