//! Testes do rating: as contas do Elo (`elo`) e o reprocessamento do
//! histórico (`replay`).

mod elo;
mod replay;

use super::*;

pub const A: PlayerId = PlayerId(1);
pub const B: PlayerId = PlayerId(2);
pub const C: PlayerId = PlayerId(3);

pub fn elo() -> Elo {
    Elo::new(EloConfig::default()).unwrap()
}

pub fn beats(winner: PlayerId, loser: PlayerId) -> Event {
    Event::Game { winner, loser }
}

pub fn ratings(replay: &Replay) -> Vec<(PlayerId, i32)> {
    replay
        .standings
        .iter()
        .map(|(p, s)| (*p, s.rating))
        .collect()
}

pub fn game(winner: (i32, i32), loser: (i32, i32)) -> Effect {
    let change = |(delta, after)| Change { delta, after };
    Effect::Game {
        winner: change(winner),
        loser: change(loser),
    }
}

pub fn replay_of(events: &[Event]) -> Replay {
    replay(&elo(), events).unwrap()
}
