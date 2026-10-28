//! Séries melhor de N mapas: tamanho e placar aqui; mapa a mapa em `games`.

mod games;

pub use games::{
    Decision, GameStatus, SeriesGame, SeriesState, Side, check_frags, check_map, check_next,
    evaluate, put,
};

use serde::{Deserialize, Serialize};

use crate::core::{EngineError, SeriesScore};

/// Quantos mapas vale cada partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesLength {
    /// Melhor de N das partidas comuns.
    pub default: u8,
    /// Melhor de N da partida decisiva: a final na eliminação simples; a
    /// grande final (e o reset) na eliminação dupla.
    pub finals: u8,
}

impl SeriesLength {
    pub fn validate(&self) -> Result<(), EngineError> {
        validate_best_of(self.default)?;
        validate_best_of(self.finals)
    }
}

pub fn validate_best_of(best_of: u8) -> Result<(), EngineError> {
    if best_of % 2 == 1 {
        Ok(())
    } else {
        Err(EngineError::InvalidBestOf(best_of))
    }
}

/// Mapas que o vencedor precisa ganhar numa série melhor de `best_of`.
pub fn wins_needed(best_of: u8) -> u8 {
    best_of / 2 + 1
}

/// O vencedor tem exatamente os mapas necessários e o perdedor, menos que isso.
pub fn validate_score(best_of: u8, score: SeriesScore) -> Result<(), EngineError> {
    let needed = wins_needed(best_of);
    if score.winner == needed && score.loser < needed {
        Ok(())
    } else {
        Err(EngineError::InvalidScore { best_of, score })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(winner: u8, loser: u8) -> SeriesScore {
        SeriesScore { winner, loser }
    }

    #[test]
    fn best_of_must_be_odd() {
        assert!(validate_best_of(1).is_ok());
        assert!(validate_best_of(5).is_ok());
        assert_eq!(validate_best_of(0), Err(EngineError::InvalidBestOf(0)));
        assert_eq!(validate_best_of(4), Err(EngineError::InvalidBestOf(4)));
    }

    #[test]
    fn valid_scores_for_best_of_three() {
        assert!(validate_score(3, score(2, 0)).is_ok());
        assert!(validate_score(3, score(2, 1)).is_ok());
    }

    #[test]
    fn invalid_scores_for_best_of_three() {
        for s in [score(1, 0), score(3, 0), score(2, 2), score(1, 2)] {
            assert!(validate_score(3, s).is_err(), "{s:?}");
        }
    }

    #[test]
    fn best_of_one() {
        assert!(validate_score(1, score(1, 0)).is_ok());
        assert!(validate_score(1, score(1, 1)).is_err());
    }
}
