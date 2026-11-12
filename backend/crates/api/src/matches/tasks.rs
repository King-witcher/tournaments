//! O que uma partida, e cada mapa dela, espera de quem vê: enviar o próximo
//! mapa, responder ao mapa do adversário, o veto ("Pronto" ou o passo da
//! vez), esperar, aceitar uma contestação (o capitão adversário) ou (organizador)
//! resolvê-la. Só com o torneio em andamento e a partida pronta. Não cobra de quem
//! a regra do torneio não deixa agir: para ele, é esperar o capitão.

use engine::{
    core::{Match, MatchState, PlayerId},
    series,
    veto::{Action, Target, Turn},
};

use super::{
    contest,
    games::{GameStatus, StoredGame, series_games},
    result::Side,
    stage::Stage,
    types::{TaskDetail, VetoStep, ViewerTask},
    veto,
    view::Viewer,
};
use crate::tournaments::{TeamAction, TeamPicker, TournamentStatus};

/// A partida está aberta para mapas: torneio em andamento e partida pronta
/// (a série ainda sem decisão).
fn open(status: TournamentStatus, m: &Match) -> bool {
    status == TournamentStatus::InProgress && matches!(m.state, MatchState::Ready { .. })
}

/// O número do próximo mapa, se a série aberta aceita mais um.
pub fn next_game(status: TournamentStatus, m: &Match, games: &[&StoredGame]) -> Option<i16> {
    if !open(status, m) {
        return None;
    }
    series::check_next(m.best_of, &series_games(games), None).ok()?;
    i16::try_from(games.len() + 1).ok()
}

/// O que o mapa `g` espera de quem vê (do lado `mine`, se joga).
pub fn game_task(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    g: &StoredGame,
    mine: Option<Side>,
    viewer: &Viewer,
) -> Option<ViewerTask> {
    if !open(status, m) {
        return None;
    }
    match g.status {
        GameStatus::Contested if viewer.organizer => Some(ViewerTask::Resolve),
        GameStatus::Contested if contest::accepter(stage, g) == Some(viewer.user_id) => {
            Some(ViewerTask::AcceptContest)
        }
        GameStatus::Contested => mine.map(|_| ViewerTask::AwaitOrganizer),
        GameStatus::Pending if mine.is_some() && mine == g.reported_side => {
            Some(ViewerTask::AwaitOpponent)
        }
        GameStatus::Pending if viewer.may(TeamAction::Report) => mine.map(|_| ViewerTask::Respond),
        GameStatus::Pending => mine.map(|_| ViewerTask::AwaitCaptain),
        GameStatus::Confirmed => None,
    }
}

/// O que o veto da próxima partida espera de quem joga (do lado `mine`):
/// "Pronto" antes de começar, o passo da vez do lado dele (quem a regra deixa)
/// ou esperar; no pick por jogador, cada um escolhe os seus, quando pode, ou o
/// capitão, os de todos. `None` sem veto no torneio, ou com o veto da próxima
/// partida concluído.
fn veto_task(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    games: &[&StoredGame],
    mine: Side,
    viewer: &Viewer,
) -> Option<ViewerTask> {
    if !stage.veto_rules.has_phases() {
        return None;
    }
    let next = next_game(status, m, games)?;
    let Some(mv) = stage.veto_of(m.id).filter(|v| v.state.begun()) else {
        let MatchState::Ready { a, b } = m.state else {
            return None;
        };
        let readiness = stage.readiness(m.id, (a, b));
        return Some(if readiness.awaits(viewer.user_id) {
            ViewerTask::Ready
        } else if readiness.side(mine) {
            ViewerTask::AwaitOpponent
        } else if stage.captain_acts_alone() {
            ViewerTask::AwaitCaptain
        } else {
            ViewerTask::AwaitTeam
        });
    };
    let side = veto::ev_side(mine);
    match mv.state.turn() {
        Turn::Act {
            game,
            action: Action::TeamPick,
            ..
        } if i16::from(game) == next => {
            let me = PlayerId(viewer.user_id);
            let pickable = mv.state.pickable();
            let team_open = pickable.iter().any(|m| m.side == side);
            Some(match viewer.rules.pick {
                TeamPicker::EachPlayer if pickable.iter().any(|m| m.player == me) => {
                    ViewerTask::Veto
                }
                TeamPicker::EachPlayer => ViewerTask::AwaitOpponent,
                TeamPicker::Captain if !team_open => ViewerTask::AwaitOpponent,
                TeamPicker::Captain if viewer.captain => ViewerTask::Veto,
                TeamPicker::Captain => ViewerTask::AwaitCaptain,
            })
        }
        ref turn @ Turn::Act {
            game, ref sides, ..
        } if i16::from(game) == next => Some(if !sides.contains(&side) {
            ViewerTask::AwaitOpponent
        } else if veto::turn_action(turn).is_none_or(|a| viewer.may(a)) {
            ViewerTask::Veto
        } else {
            ViewerTask::AwaitCaptain
        }),
        _ => None,
    }
}

/// O que a partida espera de quem vê, pela ordem de urgência: resolver
/// (organizador), a contestação que ele pode aceitar, responder a um mapa, o veto
/// ("Pronto" ou o passo da vez), enviar o próximo, esperar o organizador, esperar
/// o adversário.
pub fn match_task(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    mine: Option<Side>,
    viewer: &Viewer,
) -> Option<ViewerTask> {
    let games = stage.games_of(m.id);
    let games = games.as_slice();
    let tasks: Vec<ViewerTask> = games
        .iter()
        .filter_map(|g| game_task(status, stage, m, g, mine, viewer))
        .collect();
    let has = |t: ViewerTask| tasks.contains(&t);
    if has(ViewerTask::Resolve) {
        return Some(ViewerTask::Resolve);
    }
    let side = mine?;
    if has(ViewerTask::AcceptContest) {
        Some(ViewerTask::AcceptContest)
    } else if has(ViewerTask::Respond) {
        Some(ViewerTask::Respond)
    } else if let Some(task) = veto_task(status, stage, m, games, side, viewer) {
        Some(task)
    } else if next_game(status, m, games).is_some() {
        Some(if viewer.may(TeamAction::Report) {
            ViewerTask::Report
        } else {
            ViewerTask::AwaitCaptain
        })
    } else if has(ViewerTask::AwaitOrganizer) {
        Some(ViewerTask::AwaitOrganizer)
    } else if has(ViewerTask::AwaitOpponent) {
        Some(ViewerTask::AwaitOpponent)
    } else if has(ViewerTask::AwaitCaptain) {
        Some(ViewerTask::AwaitCaptain)
    } else {
        None
    }
}

/// O que o aviso da tarefa `task` da partida `m` (`match_task`) diz além dela,
/// para quem vê do lado `mine`: a partida de que ela fala e, conforme a
/// tarefa, o prazo, quem enviou, quem contestou, o passo do veto ou se o
/// adversário já marcou "Pronto". `None` nas tarefas de esperar.
pub fn task_detail(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    task: Option<ViewerTask>,
    mine: Option<Side>,
    viewer: &Viewer,
) -> Option<TaskDetail> {
    let task = task?;
    let games = stage.games_of(m.id);
    let name = |user: Option<i64>| user.and_then(|u| stage.nickname(u)).map(str::to_owned);
    let detail = |game: i16| TaskDetail {
        game,
        confirm_by: None,
        reported_by: None,
        contested_by: None,
        veto_step: None,
        opponent_ready: false,
        viewer_side: mine,
    };
    // A primeira partida em que a tarefa é `task` (responder, aceitar ou
    // resolver a contestação).
    let of_game = || {
        games
            .iter()
            .find(|g| game_task(status, stage, m, g, mine, viewer) == Some(task))
            .map(|g| TaskDetail {
                confirm_by: (task == ViewerTask::Respond)
                    .then_some(g.confirm_by)
                    .flatten(),
                reported_by: name(g.reported_by),
                contested_by: (task != ViewerTask::Respond)
                    .then(|| name(g.contested_by))
                    .flatten(),
                ..detail(g.number)
            })
    };
    match task {
        ViewerTask::Respond | ViewerTask::AcceptContest | ViewerTask::Resolve => of_game(),
        ViewerTask::Report => next_game(status, m, &games).map(detail),
        ViewerTask::Ready => {
            let game = next_game(status, m, &games)?;
            let MatchState::Ready { a, b } = m.state else {
                return None;
            };
            let other = match mine? {
                Side::A => Side::B,
                Side::B => Side::A,
            };
            Some(TaskDetail {
                opponent_ready: stage.readiness(m.id, (a, b)).side(other),
                ..detail(game)
            })
        }
        ViewerTask::Veto => {
            let game = next_game(status, m, &games)?;
            let step = stage.veto_of(m.id).and_then(|v| match v.state.turn() {
                Turn::Act { action, target, .. } => Some(match (target, action) {
                    (Target::Map, Action::Ban) => VetoStep::BanMap,
                    (Target::Map, _) => VetoStep::PickMap,
                    (Target::Champion, Action::Ban) => VetoStep::BanChampion,
                    (Target::Champion, _) => VetoStep::PickChampion,
                }),
                _ => None,
            });
            Some(TaskDetail {
                veto_step: step,
                ..detail(game)
            })
        }
        ViewerTask::AwaitOpponent
        | ViewerTask::AwaitOrganizer
        | ViewerTask::AwaitCaptain
        | ViewerTask::AwaitTeam => None,
    }
}
