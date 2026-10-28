//! Séries melhor de N mapas.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::core::{EngineError, MapId, SeriesScore};

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

/// Um mapa jogado na série, visto pelo placar: os frags de quem venceu a
/// série e os de quem perdeu (no mapa, qualquer um dos dois pode ter vencido).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameResult {
    pub map: MapId,
    pub winner_frags: u16,
    pub loser_frags: u16,
}

/// Detalhe por mapa de uma série cujo placar `score` já passou por
/// `validate_score`. Vazio passa: o detalhe é opcional. Com detalhe, vem um
/// item por mapa jogado, na ordem, sem empate (o duelo não termina empatado)
/// e sem repetir mapa, e os vencedores dos mapas batem com o placar: o
/// vencedor da série ganha `score.winner` mapas e o último é dele (depois do
/// mapa que fecha a série não se joga outro).
///
/// Erros: `GameCountMismatch`, `MapRepeated`, `GameTied` (com o número do
/// mapa, a partir de 1) e `GamesDisagreeWithScore`, nessa ordem.
pub fn validate_games(score: SeriesScore, games: &[GameResult]) -> Result<(), EngineError> {
    if games.is_empty() {
        return Ok(());
    }
    let expected = usize::from(score.winner) + usize::from(score.loser);
    if games.len() != expected {
        return Err(EngineError::GameCountMismatch {
            expected,
            got: games.len(),
        });
    }
    let mut seen = HashSet::new();
    for game in games {
        if !seen.insert(game.map) {
            return Err(EngineError::MapRepeated(game.map));
        }
    }
    if let Some(i) = games.iter().position(|g| g.winner_frags == g.loser_frags) {
        return Err(EngineError::GameTied { game: i + 1 });
    }
    let won = |g: &GameResult| g.winner_frags > g.loser_frags;
    let winner_maps = games.iter().filter(|g| won(g)).count();
    if winner_maps != usize::from(score.winner) || !games.last().is_some_and(won) {
        return Err(EngineError::GamesDisagreeWithScore { score });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mapa `map` com os frags do vencedor e do perdedor da série.
    fn game(map: i64, winner_frags: u16, loser_frags: u16) -> GameResult {
        GameResult {
            map: MapId(map),
            winner_frags,
            loser_frags,
        }
    }

    #[test]
    fn games_are_optional() {
        assert_eq!(validate_games(score(2, 1), &[]), Ok(()));
    }

    #[test]
    fn games_that_match_the_score() {
        let games = [game(1, 20, 10), game(2, 5, 12), game(3, 30, 29)];
        assert_eq!(validate_games(score(2, 1), &games), Ok(()));
        assert_eq!(validate_games(score(1, 0), &[game(4, 1, 0)]), Ok(()));
    }

    #[test]
    fn game_count_must_be_the_maps_played() {
        let games = [game(1, 20, 10), game(2, 20, 10)];
        assert_eq!(
            validate_games(score(2, 1), &games),
            Err(EngineError::GameCountMismatch {
                expected: 3,
                got: 2
            })
        );
    }

    #[test]
    fn maps_do_not_repeat_nor_tie() {
        let repeated = [game(1, 20, 10), game(1, 20, 10)];
        assert_eq!(
            validate_games(score(2, 0), &repeated),
            Err(EngineError::MapRepeated(MapId(1)))
        );
        let tied = [game(1, 20, 10), game(2, 15, 15)];
        assert_eq!(
            validate_games(score(2, 0), &tied),
            Err(EngineError::GameTied { game: 2 })
        );
    }

    #[test]
    fn map_winners_must_match_the_score() {
        let disagree = Err(EngineError::GamesDisagreeWithScore { score: score(2, 1) });
        // O perdedor da série venceu dois mapas.
        let swapped = [game(1, 10, 20), game(2, 10, 20), game(3, 20, 10)];
        assert_eq!(validate_games(score(2, 1), &swapped), disagree);
        // A série acabou no segundo mapa: o terceiro não existiria.
        let too_long = [game(1, 20, 10), game(2, 20, 10), game(3, 10, 20)];
        assert_eq!(validate_games(score(2, 1), &too_long), disagree);
    }

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
