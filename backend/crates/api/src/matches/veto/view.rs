//! Veto refeito → o que a página da partida mostra, do ponto de vista de
//! quem vê (as opções são só de quem está na vez).

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
        VetoGaveWay, VetoMapStep, VetoPlayer, VetoStatus, VetoTarget, VetoTeamPick, VetoTurn,
        VetoView,
    },
};
use crate::matches::{result::Side, stage::Stage, types::MapRef};

/// Onde o veto da partida pronta `m` está; `None` sem fase de veto no
/// torneio ou fora da partida pronta.
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

/// O veto da partida `m` (pronta ou decidida) para quem vê (`me`), do lado
/// `mine` se joga. `None` sem fase de veto no torneio, antes de a partida
/// ficar pronta e na partida decidida sem veto.
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
    let team_pick = matches!(
        turn,
        Turn::Act {
            action: ev::Action::TeamPick,
            ..
        }
    );
    let captain = |side: Side| {
        let team = match side {
            Side::A => mv.map(|v| v.sides.0),
            Side::B => mv.map(|v| v.sides.1),
        };
        team.and_then(|t| stage.captain(t))
            .is_some_and(|c| Some(c) == me)
    };
    // Na escolha simultânea, as opções são de cada jogador.
    let team_options = match (&ctx, mine, me) {
        (Some(ctx), Some(side), Some(me)) if ready_match && team_pick => {
            Some(state.team_options(ev_side(side), PlayerId(me), &ctx.engine()))
        }
        _ => None,
    };
    let options = match (&team_options, &ctx, mine) {
        (Some(t), _, _) => t
            .champions
            .iter()
            .map(|id| champion_ref(stage, *id).slug)
            .collect(),
        (None, Some(ctx), Some(side)) if ready_match && captain(side) => state
            .options(ev_side(side), &ctx.engine())
            .into_iter()
            .map(|c| match c {
                ev::Choice::Map(id) => map_ref(stage, id).slug,
                ev::Choice::Champion(id) => champion_ref(stage, id).slug,
            })
            .collect(),
        _ => Vec::new(),
    };
    let missing = match (ready_match, state.game()) {
        (true, Some(g)) => state
            .missing(g)
            .into_iter()
            .map(|(side, p)| VetoPlayer {
                side: api_side(side),
                user_id: p.0,
                nickname: nickname(stage, mv, p),
            })
            .collect(),
        _ => Vec::new(),
    };
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
                ..
            } if ready_match => Some(VetoTurn {
                action: action(*a),
                target: match target {
                    ev::Target::Map => VetoTarget::Map,
                    ev::Target::Champion => VetoTarget::Champion,
                },
                sides: sides.iter().copied().map(api_side).collect(),
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
        games: (1..=state.game().unwrap_or(0))
            .map(|g| game(stage, mv, &state, g))
            .collect(),
        options,
        gave_way: team_options.and_then(|t| t.gave_way).map(|g| match g {
            ev::GaveWay::Rule => VetoGaveWay::Rule,
            ev::GaveWay::Team => VetoGaveWay::Team,
        }),
        missing,
        champions: champions(stage, &state, ctx.as_ref(), mine.zip(me)),
        undo_request: mv.and_then(MatchVeto::undo_request),
        can_undo: ready_match && state.can_undo(),
        reportable: ready_match && reportable(stage, m, &state),
        history: mv
            .map(|mv| mv.rows.iter().map(|r| event(stage, r)).collect())
            .unwrap_or_default(),
    })
}

/// Os jogadores podem enviar o próximo mapa: o veto dele acabou (ou ele já
/// passou no veto, quando foi anulado para ser jogado de novo).
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
        starter: state.starter(g).map(api_side),
        bans: state
            .champion_steps(g)
            .iter()
            .filter(|s| s.action == ev::Action::Ban)
            .filter_map(|s| match s.choice {
                ev::Choice::Champion(c) => Some(VetoBan {
                    side: api_side(s.side),
                    champion: champion_ref(stage, c),
                }),
                ev::Choice::Map(_) => None,
            })
            .collect(),
        pick_a: picks[0].map(|c| champion_ref(stage, c)),
        pick_b: picks[1].map(|c| champion_ref(stage, c)),
        team_picks: team_picks(stage, mv, state, g),
    }
}

/// TDM: a escolha de cada jogador no jogo `g`, com o sorteio do fim da
/// janela marcado (pela linha gravada).
fn team_picks(stage: &Stage, mv: Option<&MatchVeto>, state: &Veto, g: u8) -> Vec<VetoTeamPick> {
    state
        .team_picks(g)
        .into_iter()
        .filter_map(|a| {
            let (ev::Choice::Champion(c), Some(player)) = (a.choice, a.player) else {
                return None;
            };
            let timeout = mv
                .and_then(|mv| {
                    mv.rows.iter().rev().find(|r| {
                        r.kind == Kind::Pick
                            && r.game == Some(i16::from(g))
                            && r.player == Some(player.0)
                    })
                })
                .is_some_and(|r| r.timeout);
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

/// Os campeões ativos no jogo em andamento: banidos neste mapa e tirados de
/// cada lado pela regra; no TDM, também de quem vê (`viewer`: o lado dele e
/// o usuário), se ele joga.
fn champions(
    stage: &Stage,
    state: &Veto,
    ctx: Option<&Ctx>,
    viewer: Option<(Side, i64)>,
) -> Vec<VetoChampion> {
    let (Some(ctx), Some(g)) = (ctx, state.game()) else {
        return Vec::new();
    };
    if !stage.veto_rules.champion_phase {
        return Vec::new();
    }
    let bans: Vec<(ev::Applied, ChampionId)> = state
        .champion_steps(g)
        .iter()
        .filter(|s| s.action == ev::Action::Ban)
        .filter_map(|s| match s.choice {
            ev::Choice::Champion(c) => Some((*s, c)),
            ev::Choice::Map(_) => None,
        })
        .collect();
    let used_a = state.forbidden(engine::series::Side::A, &ctx.engine());
    let used_b = state.forbidden(engine::series::Side::B, &ctx.engine());
    let used_you = match viewer {
        Some((side, me)) if stage.veto_rules.teams() => {
            state.forbidden_for(ev_side(side), PlayerId(me), &ctx.engine())
        }
        _ => Default::default(),
    };
    stage
        .champions
        .iter()
        .filter(|c| c.active)
        .map(|c| VetoChampion {
            slug: c.slug.clone(),
            name: c.name.clone(),
            banned_by: bans
                .iter()
                .find(|(_, id)| *id == c.id)
                .map(|(s, _)| api_side(s.side)),
            used_by_a: used_a.contains(&c.id),
            used_by_b: used_b.contains(&c.id),
            used_by_you: used_you.contains(&c.id),
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
        side: r.side,
        map: r.map.map(|id| map_ref(stage, id)),
        champion: r.champion.map(|id| champion_ref(stage, id)),
        timeout: r.timeout,
        actor: r.actor.and(r.actor_name.clone()),
        player: r.player.map(|_| r.player_name.clone().unwrap_or_default()),
        at: r.created_at,
    }
}
