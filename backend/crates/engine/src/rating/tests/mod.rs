//! Testes do rating: as contas do Elo (`elo`), o reprocessamento do
//! histórico (`replay`) e os jogos entre times (`teams`).

mod elo;
mod replay;
mod teams;

use super::*;

pub const A: PlayerId = PlayerId(1);
pub const B: PlayerId = PlayerId(2);
pub const C: PlayerId = PlayerId(3);
pub const D: PlayerId = PlayerId(4);

pub fn elo() -> Elo {
    Elo::new(EloConfig::default()).unwrap()
}

/// Um mapa de duelo: um jogador de cada lado.
pub fn beats(winner: PlayerId, loser: PlayerId) -> Event {
    Event::Game {
        winners: vec![winner],
        losers: vec![loser],
    }
}

/// Um mapa entre times, com a escalação de cada lado.
pub fn team_beats(winners: &[PlayerId], losers: &[PlayerId]) -> Event {
    Event::Game {
        winners: winners.to_vec(),
        losers: losers.to_vec(),
    }
}

pub fn ratings(replay: &Replay) -> Vec<(PlayerId, i32)> {
    replay
        .standings
        .iter()
        .map(|(p, s)| (*p, s.rating))
        .collect()
}

/// O efeito de um mapa de duelo.
pub fn game(winner: (i32, i32), loser: (i32, i32)) -> Effect {
    let change = |(delta, after)| Change { delta, after };
    Effect::Game {
        winners: vec![change(winner)],
        losers: vec![change(loser)],
    }
}

pub fn replay_of(events: &[Event]) -> Replay {
    replay(&elo(), events).unwrap()
}
