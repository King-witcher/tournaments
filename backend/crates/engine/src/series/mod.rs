//! Séries melhor de N mapas: tamanho e placar aqui; mapa a mapa em `games`.
//!
//! N é ímpar ou, só no round robin, par (qualquer N par, `even_series`): a série par
//! pode empatar (`Outcome::Drawn`), com todos os N mapas confirmados e metade
//! para cada lado. Se alguém chega à maioria antes, ela fecha como as outras: a
//! md2 termina em 2 x 0 ou 1 x 1, a md4 em 3 x 0, 3 x 1 ou 2 x 2, a md6 em 4 x 0,
//! 4 x 1, 4 x 2 ou 3 x 3.

mod games;

pub use games::{
    Decision, GameStatus, SeriesGame, SeriesState, Side, check_map, check_next, evaluate,
    games_left, put, winner_from_frags,
};

use serde::Serialize;

use crate::core::{EngineError, Outcome, SeriesScore};

/// Quantos mapas vale cada partida.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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

/// Série par (qualquer N par, só no round robin): pode empatar, e o veto de
/// mapas dela só tem bans.
pub const fn even_series(best_of: u8) -> bool {
    best_of >= 2 && best_of.is_multiple_of(2)
}

/// Melhor de N ímpar: a da eliminação e a do suíço.
pub fn validate_best_of(best_of: u8) -> Result<(), EngineError> {
    if best_of % 2 == 1 {
        Ok(())
    } else {
        Err(EngineError::InvalidBestOf(best_of))
    }
}

/// Melhor de N do round robin: ímpar, ou uma das séries pares.
pub fn validate_league_best_of(best_of: u8) -> Result<(), EngineError> {
    if even_series(best_of) {
        Ok(())
    } else {
        validate_best_of(best_of)
    }
}

/// Mapas que o vencedor precisa ganhar numa série melhor de `best_of`: a
/// maioria. É também o que o W.O. vale (2 x 0 na md2 e na md3, 3 x 0 na md4 e na
/// md5, 4 x 0 na md6).
pub const fn wins_needed(best_of: u8) -> u8 {
    best_of / 2 + 1
}

/// O vencedor tem exatamente os mapas necessários, o perdedor menos que isso, e
/// os dois juntos não passam dos mapas da série (a md2 não tem 2 x 1, a md4 não
/// tem 3 x 2).
pub fn validate_score(best_of: u8, score: SeriesScore) -> Result<(), EngineError> {
    let needed = wins_needed(best_of);
    let fits = u16::from(score.winner) + u16::from(score.loser) <= u16::from(best_of);
    if score.winner == needed && score.loser < needed && fits {
        Ok(())
    } else {
        Err(EngineError::InvalidScore { best_of, score })
    }
}

/// O resultado cabe numa melhor de `best_of`: o placar, em `validate_score`; só
/// a série par empata, com metade dos mapas para cada lado; o W.O. não tem
/// placar para conferir.
pub fn validate_outcome(best_of: u8, outcome: Outcome) -> Result<(), EngineError> {
    match outcome {
        Outcome::Played { score, .. } => validate_score(best_of, score),
        Outcome::Walkover { .. } => Ok(()),
        Outcome::Drawn { maps } if even_series(best_of) && maps == best_of / 2 => Ok(()),
        Outcome::Drawn { maps } => Err(EngineError::InvalidScore {
            best_of,
            score: SeriesScore {
                winner: maps,
                loser: maps,
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ParticipantId;

    fn score(winner: u8, loser: u8) -> SeriesScore {
        SeriesScore { winner, loser }
    }

    #[test]
    fn best_of_must_be_odd() {
        assert!(validate_best_of(1).is_ok());
        assert!(validate_best_of(5).is_ok());
        assert_eq!(validate_best_of(0), Err(EngineError::InvalidBestOf(0)));
        for even in [2, 4, 6, 8] {
            assert_eq!(
                validate_best_of(even),
                Err(EngineError::InvalidBestOf(even))
            );
        }
    }

    #[test]
    fn the_league_also_takes_any_even_series() {
        for n in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 254, 255] {
            assert!(validate_league_best_of(n).is_ok(), "{n}");
        }
        assert_eq!(
            validate_league_best_of(0),
            Err(EngineError::InvalidBestOf(0))
        );
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

    #[test]
    fn the_scores_of_each_even_series() {
        let valid: [(u8, &[SeriesScore]); 3] = [
            (2, &[score(2, 0)]),
            (4, &[score(3, 0), score(3, 1)]),
            (6, &[score(4, 0), score(4, 1), score(4, 2)]),
        ];
        for (best_of, scores) in valid {
            for s in scores {
                assert_eq!(validate_score(best_of, *s), Ok(()), "md{best_of} {s:?}");
            }
        }
        // Mais mapas que a série, sem a maioria, ou passando dela.
        let invalid: [(u8, &[SeriesScore]); 3] = [
            (2, &[score(2, 1), score(1, 0), score(1, 1), score(3, 0)]),
            (4, &[score(3, 2), score(2, 2), score(4, 0), score(2, 0)]),
            (6, &[score(4, 3), score(3, 3), score(5, 0), score(3, 0)]),
        ];
        for (best_of, scores) in invalid {
            for s in scores {
                assert!(validate_score(best_of, *s).is_err(), "md{best_of} {s:?}");
            }
        }
        assert_eq!([wins_needed(2), wins_needed(4), wins_needed(6)], [2, 3, 4]);
    }

    #[test]
    fn only_an_even_series_draws_with_half_the_maps_each() {
        for (best_of, maps) in [(2, 1), (4, 2), (6, 3), (8, 4)] {
            assert_eq!(validate_outcome(best_of, Outcome::Drawn { maps }), Ok(()));
            let wrong = maps - 1;
            assert_eq!(
                validate_outcome(best_of, Outcome::Drawn { maps: wrong }),
                Err(EngineError::InvalidScore {
                    best_of,
                    score: score(wrong, wrong),
                })
            );
        }
        for best_of in [1, 3, 5, 9] {
            let maps = best_of / 2;
            assert_eq!(
                validate_outcome(best_of, Outcome::Drawn { maps }),
                Err(EngineError::InvalidScore {
                    best_of,
                    score: score(maps, maps),
                })
            );
        }
        let winner = ParticipantId(1);
        for best_of in [2, 3, 4, 6] {
            assert_eq!(
                validate_outcome(best_of, Outcome::Walkover { winner }),
                Ok(())
            );
        }
    }
}
