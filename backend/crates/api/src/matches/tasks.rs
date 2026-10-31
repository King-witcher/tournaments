//! O que uma partida, e cada mapa dela, espera de quem vê: enviar o próximo
//! mapa, responder ao mapa do adversário, o veto ("Pronto" ou o passo da
//! vez), esperar, ou (organizador) resolver uma contestação. Só com o torneio em andamento e a partida pronta.

use engine::{
    core::{Match, MatchState, PlayerId},
    series,
    veto::{Action, Turn},
};

use super::{
    games::{GameStatus, StoredGame, series_games},
    result::Side,
    stage::Stage,
    types::ViewerTask,
    veto,
    view::Viewer,
};
use crate::tournaments::TournamentStatus;

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
        GameStatus::Contested => mine.map(|_| ViewerTask::AwaitOrganizer),
        GameStatus::Pending if mine.is_some() && mine == g.reported_side => {
            Some(ViewerTask::AwaitOpponent)
        }
        GameStatus::Pending => mine.map(|_| ViewerTask::Respond),
        GameStatus::Confirmed => None,
    }
}

/// O que o veto da próxima partida espera de quem joga (do lado `mine`):
/// "Pronto" antes de começar, o passo da vez do lado dele (qualquer jogador
/// do time age pelo time) ou esperar o adversário; na janela simultânea do
/// TDM, cada um escolhe os seus. `None` sem veto no torneio, ou com o veto
/// da próxima partida concluído.
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
        let ready = stage.veto_of(m.id).map(|v| v.ready()).unwrap_or_default();
        let me = usize::from(mine == Side::B);
        return Some(if ready[me] {
            ViewerTask::AwaitOpponent
        } else {
            ViewerTask::Ready
        });
    };
    match mv.state.turn() {
        Turn::Act {
            game,
            action: Action::TeamPick,
            ..
        } if i16::from(game) == next => {
            let me = PlayerId(viewer.user_id);
            let missing = mv.state.missing();
            Some(if missing.iter().any(|m| m.player == me) {
                ViewerTask::Veto
            } else {
                ViewerTask::AwaitOpponent
            })
        }
        Turn::Act { game, sides, .. } if i16::from(game) == next => {
            Some(if sides.contains(&veto::ev_side(mine)) {
                ViewerTask::Veto
            } else {
                ViewerTask::AwaitOpponent
            })
        }
        _ => None,
    }
}

/// O que a partida espera de quem vê, pela ordem de urgência: resolver
/// (organizador), responder a um mapa, o veto ("Pronto" ou o passo da vez),
/// enviar o próximo, esperar o organizador, esperar o adversário.
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
        .filter_map(|g| game_task(status, m, g, mine, viewer))
        .collect();
    let has = |t: ViewerTask| tasks.contains(&t);
    if has(ViewerTask::Resolve) {
        return Some(ViewerTask::Resolve);
    }
    let side = mine?;
    if has(ViewerTask::Respond) {
        Some(ViewerTask::Respond)
    } else if let Some(task) = veto_task(status, stage, m, games, side, viewer) {
        Some(task)
    } else if next_game(status, m, games).is_some() {
        Some(ViewerTask::Report)
    } else if has(ViewerTask::AwaitOrganizer) {
        Some(ViewerTask::AwaitOrganizer)
    } else if has(ViewerTask::AwaitOpponent) {
        Some(ViewerTask::AwaitOpponent)
    } else {
        None
    }
}
