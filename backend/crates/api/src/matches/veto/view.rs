//! Veto refeito → o que a página mostra, do ponto de vista de quem vê (as
//! opções e os picks são só de quem joga e pode agir agora). A montagem
//! (`build`) não sabe de onde o veto veio: a série de um torneio (`view`) e a
//! sala de veto (`veto_rooms`) dizem o que ela precisa (`Board`, `Shown`).
//!
//! **Às cegas**: todo pick que sai daqui passa por `Sealed`, o ponto único que
//! esconde o de um lado de quem não é dele enquanto a janela está aberta. Nada
//! mais expõe um pick: as opções e os motivos só olham o outro lado sem espelho,
//! e às cegas exige espelho.

use std::borrow::Cow;

use engine::{
    core::{ChampionId, MapId, Match, MatchState, PlayerId},
    veto::{self as ev, Turn, Veto},
};

use super::{
    Ctx,
    db::Champion,
    state::{Kind, MatchVeto, VetoLog, VetoRow, api_side, ev_side},
    types::{
        ChampionRef, VetoAction, VetoBan, VetoChampion, VetoEvent, VetoEventKind, VetoGame,
        VetoGaveWay, VetoHiddenPick, VetoMapStep, VetoPickSlot, VetoPlayer, VetoStatus, VetoTarget,
        VetoTeamPick, VetoTurn, VetoUnavailable, VetoUnavailableReason, VetoView,
    },
};
use crate::{
    matches::{games::StoredGame, result::Side, stage::Stage, tasks, types::MapRef},
    tournaments::{MapMode, PoolMap, TournamentStatus},
};

/// O que o veto joga: as regras, os tempos, o pool de mapas e os campeões.
pub struct Board<'a> {
    pub rules: &'a ev::VetoRules,
    pub step_seconds: i32,
    pub team_pick_seconds: i32,
    pub maps: &'a [PoolMap],
    pub champions: &'a [Champion],
}

impl Board<'_> {
    fn map_ref(&self, id: MapId) -> MapRef {
        map_ref(self.maps, id)
    }

    fn champion_ref(&self, id: ChampionId) -> ChampionRef {
        self.champions.iter().find(|c| c.id == id).map_or_else(
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

    const fn mode(&self) -> MapMode {
        match self.rules.map_mode {
            ev::MapMode::Veto => MapMode::Veto,
            ev::MapMode::Vote => MapMode::Vote,
            ev::MapMode::Random => MapMode::Random,
            ev::MapMode::Off => MapMode::Off,
        }
    }
}

/// O veto a mostrar e para quem.
pub struct Shown<'a> {
    /// O histórico; `None` na série pronta em que ninguém agiu.
    pub log: Option<&'a VetoLog>,
    pub state: &'a Veto,
    /// O veto ainda aceita ação: mostra a vez e o prazo.
    pub open: bool,
    pub can_undo: bool,
    pub reportable: bool,
    /// Quem vê, se pode agir: as opções (`Ctx`), o lado e quem é.
    pub acting: Option<(&'a Ctx, Side, PlayerId)>,
    /// O lado de quem vê, se é de um (mesmo sem poder agir agora): é o único
    /// que ele enxerga com a janela às cegas aberta. Espectador, gestão e
    /// organizador que não joga não têm lado.
    pub viewer: Option<Side>,
    /// O nome de um jogador.
    pub names: &'a dyn Fn(PlayerId) -> String,
}

/// O que quem vê não enxerga agora (ver o `//!`): os picks, das partidas
/// `games` (`Veto::sealed_games`), dos lados que não são o dele.
struct Sealed {
    games: Vec<u8>,
    viewer: Option<Side>,
}

impl Sealed {
    fn of(shown: &Shown<'_>) -> Self {
        Self {
            games: shown.state.sealed_games(),
            viewer: shown.viewer,
        }
    }

    /// O pick de `side` para a partida `game` fica escondido de quem vê.
    fn hides(&self, side: Side, game: u8) -> bool {
        self.games.contains(&game) && self.viewer != Some(side)
    }

    /// A linha gravada `r` é um pick escondido de quem vê. Sem o lado ou a
    /// partida, que todo pick de campeão tem, fica escondida também.
    fn hides_row(&self, r: &VetoRow) -> bool {
        if self.games.is_empty() || r.kind != Kind::Pick || r.champion.is_none() {
            return false;
        }
        match (r.side, r.pick_game.and_then(|g| u8::try_from(g).ok())) {
            (Some(side), Some(game)) => self.hides(side, game),
            _ => true,
        }
    }
}

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

fn map_ref(maps: &[PoolMap], id: MapId) -> MapRef {
    maps.iter().find(|m| m.id == id).map_or_else(
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

/// O mapa do próximo jogo que dá para enviar na série `m` (`games`: os
/// enviados), quando o veto, a votação ou o sorteio já o deu. Sai do veto já
/// refeito da fase, sem consulta nova.
pub fn next_map(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    games: &[&StoredGame],
) -> Option<MapRef> {
    let next = u8::try_from(tasks::next_game(status, m, games)?).ok()?;
    let id = stage.veto_of(m.id)?.state.map_of(next)?;
    Some(map_ref(&stage.map_pool, id))
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
    let acting = match (&ctx, mine, me) {
        (Some(ctx), Some(side), Some(me)) if ready_match => Some((ctx, side, PlayerId(me))),
        _ => None,
    };
    let board = Board {
        rules: &stage.veto_rules,
        step_seconds: stage.veto_seconds,
        team_pick_seconds: stage.team_pick_seconds,
        maps: &stage.map_pool,
        champions: &stage.champions,
    };
    Some(build(
        &board,
        &Shown {
            log: mv.map(|mv| &mv.log),
            state: &state,
            open: ready_match,
            can_undo: ready_match && state.can_undo(),
            reportable: ready_match && reportable(stage, m, &state),
            acting,
            viewer: mine,
            names: &|player| nickname(stage, mv, player),
        },
    ))
}

/// O veto `shown`, jogado em `board`, como a página o mostra.
pub fn build(board: &Board<'_>, shown: &Shown<'_>) -> VetoView {
    let state = shown.state;
    let log = shown.log;
    let turn = state.turn();
    let sealed = Sealed::of(shown);
    let ready = log.map(VetoLog::ready).unwrap_or_default();
    let options = shown
        .acting
        .map(|(ctx, side, _)| {
            state
                .options(ev_side(side), &ctx.engine())
                .into_iter()
                .map(|c| match c {
                    ev::Choice::Map(id) => board.map_ref(id).slug,
                    ev::Choice::Champion(id) => board.champion_ref(id).slug,
                })
                .collect()
        })
        .unwrap_or_default();
    let status = match turn {
        Turn::NotBegun => VetoStatus::WaitingReady,
        Turn::Act { .. } | Turn::RostersUnknown { .. } => VetoStatus::InProgress,
        Turn::Done { .. } => VetoStatus::Done,
    };
    VetoView {
        map_mode: board.mode(),
        champion_phase: board.rules.champion_phase,
        pick_order: board.rules.pick_order.into(),
        team_size: i16::from(board.rules.team_size),
        step_seconds: board.step_seconds,
        team_pick_seconds: board
            .rules
            .pick_order
            .window()
            .then_some(board.team_pick_seconds),
        pool: board.maps.iter().map(|p| board.map_ref(p.id)).collect(),
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
            } if shown.open => Some(VetoTurn {
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
        step: log.and_then(VetoLog::step),
        deadline: log
            .filter(|_| shown.open)
            .and_then(|log| log.deadline(board.step_seconds, board.team_pick_seconds)),
        map_steps: state
            .map_steps()
            .iter()
            .filter_map(|s| match s.choice {
                ev::Choice::Map(id) => Some(VetoMapStep {
                    side: api_side(s.side),
                    action: action(s.action),
                    map: board.map_ref(id),
                }),
                ev::Choice::Champion(_) => None,
            })
            .collect(),
        remaining: if board.rules.map_mode == ev::MapMode::Veto && state.begun() {
            state
                .remaining_maps()
                .into_iter()
                .map(|id| board.map_ref(id))
                .collect()
        } else {
            Vec::new()
        },
        bans: bans(board, log, state),
        games: (1..=state.best_of())
            .map(|g| game(board, shown, &sealed, g))
            .collect(),
        options,
        picks: shown
            .acting
            .map(|(ctx, side, me)| picks(board, state, &turn, ctx, side, me))
            .unwrap_or_default(),
        missing: if shown.open {
            missing(shown)
        } else {
            Vec::new()
        },
        champions: champions(board, state),
        undo_request: log.and_then(VetoLog::undo_request),
        can_undo: shown.can_undo,
        reportable: shown.reportable,
        history: log
            .map(|log| {
                log.rows
                    .iter()
                    .filter(|r| !sealed.hides_row(r))
                    .map(|r| event(board, r))
                    .collect()
            })
            .unwrap_or_default(),
    }
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

/// A linha gravada do passo de campeão: o último pick de `player` (ou do
/// lado, no duelo) para a partida `game`, ou o ban do `champion`.
fn timed_out(log: Option<&VetoLog>, matches: impl Fn(&VetoRow) -> bool) -> bool {
    log.and_then(|log| log.rows.iter().rev().find(|r| matches(r)))
        .is_some_and(|r| r.timeout)
}

fn bans(board: &Board<'_>, log: Option<&VetoLog>, state: &Veto) -> Vec<VetoBan> {
    state
        .bans()
        .iter()
        .filter_map(|s| {
            let c = s.champion()?;
            Some(VetoBan {
                side: api_side(s.side),
                champion: board.champion_ref(c),
                timeout: timed_out(log, |r| r.kind == Kind::Ban && r.champion == Some(c)),
            })
        })
        .collect()
}

fn game(board: &Board<'_>, shown: &Shown<'_>, sealed: &Sealed, g: u8) -> VetoGame {
    let state = shown.state;
    let picks = state.picks(g);
    let side_pick = |side: Side, pick: Option<ChampionId>| {
        pick.filter(|_| !sealed.hides(side, g))
            .map(|c| board.champion_ref(c))
    };
    let (team_picks, hidden_picks) = team_picks(board, shown, sealed, g);
    let votes = state.votes(g);
    let slug = |m: Option<MapId>| m.map(|id| board.map_ref(id).slug);
    VetoGame {
        game: i16::from(g),
        map: state.map_of(g).map(|id| board.map_ref(id)),
        picked_by: state.picker(g).map(api_side),
        offered: state
            .offered(g)
            .into_iter()
            .map(|id| board.map_ref(id))
            .collect(),
        vote_a: slug(votes[0]),
        vote_b: slug(votes[1]),
        pick_a: side_pick(Side::A, picks[0]),
        pick_b: side_pick(Side::B, picks[1]),
        team_picks,
        hidden_picks,
    }
}

/// No pick por jogador: a escolha que conta de cada um para a partida `g`, com
/// o sorteio do fim do tempo marcado (pela linha gravada), e, à parte, quem
/// escolheu sem que quem vê possa saber o quê (`Sealed`).
fn team_picks(
    board: &Board<'_>,
    shown: &Shown<'_>,
    sealed: &Sealed,
    g: u8,
) -> (Vec<VetoTeamPick>, Vec<VetoHiddenPick>) {
    let mut seen = Vec::new();
    let mut hidden = Vec::new();
    for a in shown.state.team_picks(g) {
        let (Some(c), Some(player)) = (a.champion(), a.player) else {
            continue;
        };
        let side = api_side(a.side);
        let nickname = (shown.names)(player);
        if sealed.hides(side, g) {
            hidden.push(VetoHiddenPick {
                side,
                user_id: player.0,
                nickname,
            });
            continue;
        }
        let timeout = timed_out(shown.log, |r| {
            r.kind == Kind::Pick && r.pick_game == Some(i16::from(g)) && r.player == Some(player.0)
        });
        seen.push(VetoTeamPick {
            side,
            user_id: player.0,
            nickname,
            champion: board.champion_ref(c),
            timeout,
        });
    }
    (seen, hidden)
}

/// No pick por jogador: quem falta, com as partidas de cada um.
fn missing(shown: &Shown<'_>) -> Vec<VetoPlayer> {
    let mut players: Vec<VetoPlayer> = Vec::new();
    for m in shown.state.missing() {
        match players.iter_mut().find(|p| p.user_id == m.player.0) {
            Some(p) => p.games.push(i16::from(m.game)),
            None => players.push(VetoPlayer {
                side: api_side(m.side),
                user_id: m.player.0,
                nickname: (shown.names)(m.player),
                games: vec![i16::from(m.game)],
            }),
        }
    }
    players
}

/// Os picks que quem vê (`me`, do lado `side`) pode fazer agora: no duelo
/// alternado, o da vez do lado dele; no pick por jogador, as partidas em que ele
/// falta e pode escolher agora (`pickable`).
fn picks(
    board: &Board<'_>,
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
                .pickable()
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
                    .map(|c| board.champion_ref(*c).slug)
                    .collect(),
                unavailable: state
                    .unavailable(picker, game, &ctx)
                    .into_iter()
                    .map(|(c, reason)| VetoUnavailable {
                        slug: board.champion_ref(c).slug,
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

/// Os campeões ativos, com quem baniu cada um na série.
fn champions(board: &Board<'_>, state: &Veto) -> Vec<VetoChampion> {
    if !board.rules.champion_phase {
        return Vec::new();
    }
    let bans = state.bans();
    board
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

fn event(board: &Board<'_>, r: &VetoRow) -> VetoEvent {
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
        map: r.map.map(|id| board.map_ref(id)),
        champion: r.champion.map(|id| board.champion_ref(id)),
        timeout: r.timeout,
        actor: r.actor.and(r.actor_name.clone()),
        player: r.player.map(|_| r.player_name.clone().unwrap_or_default()),
        at: r.created_at,
    }
}
