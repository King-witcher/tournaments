//! O que uma partida, e cada mapa dela, espera de quem vê: enviar o próximo
//! mapa, responder ao mapa do adversário, esperar, ou (organizador) resolver
//! uma contestação. Só com o torneio em andamento e a partida pronta.

use engine::{
    core::{Match, MatchState},
    series,
};

use super::{
    games::{GameStatus, StoredGame, series_games},
    result::Side,
    types::ViewerTask,
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

/// O que a partida espera de quem vê, pela ordem de urgência: resolver
/// (organizador), responder a um mapa, enviar o próximo, esperar o
/// organizador, esperar o adversário.
pub fn match_task(
    status: TournamentStatus,
    m: &Match,
    games: &[&StoredGame],
    mine: Option<Side>,
    viewer: &Viewer,
) -> Option<ViewerTask> {
    let tasks: Vec<ViewerTask> = games
        .iter()
        .filter_map(|g| game_task(status, m, g, mine, viewer))
        .collect();
    let has = |t: ViewerTask| tasks.contains(&t);
    if has(ViewerTask::Resolve) {
        return Some(ViewerTask::Resolve);
    }
    mine?;
    if has(ViewerTask::Respond) {
        Some(ViewerTask::Respond)
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
