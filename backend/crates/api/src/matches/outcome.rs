//! Placar que chega pelos lados (mapas de `a` e de `b`, frags de `a` e de `b`
//! por mapa) → o resultado que o `engine` valida e que se grava. As regras de
//! série (placar do melhor de N, detalhe batendo com ele) são do `engine`;
//! aqui ficam os limites do site e a troca de orientação.

use std::collections::HashMap;

use engine::{
    core::{MapId, Outcome, ParticipantId, SeriesScore},
    series::{self, GameResult},
};

use super::{result::Side, types::GameInput};
use crate::shared::ApiError;

/// Teto de frags num mapa (o mesmo check está na tabela).
pub const MAX_FRAGS: u16 = 999;

/// Um mapa do detalhe, pronto para gravar.
pub struct NewGame {
    pub map: MapId,
    pub a_frags: i16,
    pub b_frags: i16,
}

/// O resultado que uma ação propõe para a partida.
pub struct Proposed {
    pub outcome: Outcome,
    /// Mapas de `a` e de `b`; `None` no W.O.
    pub score: Option<(u8, u8)>,
    pub games: Vec<NewGame>,
}

/// Série jogada entre `a` e `b` numa melhor de `best_of`, com o detalhe
/// opcional por mapa (`maps`: ids dos mapas ativos pelo slug). Erros: 422
/// `match.invalid_score` `{best_of}`, `match.frags_invalid` `{max}`,
/// `match.map_unknown` e os do detalhe (`match.games_count_mismatch`
/// `{expected, got}`, `match.map_repeated`, `match.game_tied` `{game}`,
/// `match.games_disagree`).
pub fn played(
    best_of: u8,
    (a, b): (ParticipantId, ParticipantId),
    (a_wins, b_wins): (u8, u8),
    games: &[GameInput],
    maps: &HashMap<String, MapId>,
) -> Result<Proposed, ApiError> {
    let a_won = a_wins > b_wins;
    let (winner, score) = if a_won {
        (
            a,
            SeriesScore {
                winner: a_wins,
                loser: b_wins,
            },
        )
    } else {
        (
            b,
            SeriesScore {
                winner: b_wins,
                loser: a_wins,
            },
        )
    };
    series::validate_score(best_of, score)?;

    let mut detail = Vec::with_capacity(games.len());
    let mut checked = Vec::with_capacity(games.len());
    for g in games {
        if g.a_frags > MAX_FRAGS || g.b_frags > MAX_FRAGS {
            return Err(ApiError::invalid("match.frags_invalid")
                .with_params([("max", i64::from(MAX_FRAGS))]));
        }
        let map = *maps
            .get(&g.map)
            .ok_or(ApiError::invalid("match.map_unknown"))?;
        let (winner_frags, loser_frags) = if a_won {
            (g.a_frags, g.b_frags)
        } else {
            (g.b_frags, g.a_frags)
        };
        checked.push(GameResult {
            map,
            winner_frags,
            loser_frags,
        });
        // Cabem em i16: no máximo MAX_FRAGS.
        detail.push(NewGame {
            map,
            a_frags: g.a_frags as i16,
            b_frags: g.b_frags as i16,
        });
    }
    series::validate_games(score, &checked)?;
    Ok(Proposed {
        outcome: Outcome::Played { winner, score },
        score: Some((a_wins, b_wins)),
        games: detail,
    })
}

/// W.O. para o lado `winner`.
pub fn walkover((a, b): (ParticipantId, ParticipantId), winner: Side) -> Proposed {
    Proposed {
        outcome: Outcome::Walkover {
            winner: match winner {
                Side::A => a,
                Side::B => b,
            },
        },
        score: None,
        games: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: ParticipantId = ParticipantId(10);
    const B: ParticipantId = ParticipantId(20);

    fn maps() -> HashMap<String, MapId> {
        ["awoken", "exile", "insomnia"]
            .into_iter()
            .zip(1..)
            .map(|(slug, id)| (slug.to_owned(), MapId(id)))
            .collect()
    }

    fn game(map: &str, a_frags: u16, b_frags: u16) -> GameInput {
        GameInput {
            map: map.into(),
            a_frags,
            b_frags,
        }
    }

    fn code(r: Result<Proposed, ApiError>) -> &'static str {
        r.err().map_or("ok", |e| e.code)
    }

    #[test]
    fn b_can_win_and_the_detail_is_turned_around() {
        let games = [
            game("awoken", 20, 10),
            game("exile", 5, 15),
            game("insomnia", 9, 30),
        ];
        let p = played(3, (A, B), (1, 2), &games, &maps()).unwrap();
        assert_eq!(
            p.outcome,
            Outcome::Played {
                winner: B,
                score: SeriesScore {
                    winner: 2,
                    loser: 1
                }
            }
        );
        assert_eq!(p.score, Some((1, 2)));
        assert_eq!(p.games[2].b_frags, 30);
    }

    #[test]
    fn site_limits_and_engine_rules() {
        let m = maps();
        assert_eq!(
            code(played(3, (A, B), (2, 2), &[], &m)),
            "match.invalid_score"
        );
        assert_eq!(
            code(played(3, (A, B), (3, 0), &[], &m)),
            "match.invalid_score"
        );
        let too_many = [game("awoken", 1000, 0), game("exile", 1, 0)];
        assert_eq!(
            code(played(3, (A, B), (2, 0), &too_many, &m)),
            "match.frags_invalid"
        );
        let unknown = [game("q3dm17", 10, 0), game("exile", 1, 0)];
        assert_eq!(
            code(played(3, (A, B), (2, 0), &unknown, &m)),
            "match.map_unknown"
        );
        let short = [game("awoken", 10, 0)];
        assert_eq!(
            code(played(3, (A, B), (2, 0), &short, &m)),
            "match.games_count_mismatch"
        );
        let wrong = [game("awoken", 0, 10), game("exile", 1, 0)];
        assert_eq!(
            code(played(3, (A, B), (2, 0), &wrong, &m)),
            "match.games_disagree"
        );
    }

    #[test]
    fn walkover_has_no_score() {
        let p = walkover((A, B), Side::B);
        assert_eq!(p.outcome, Outcome::Walkover { winner: B });
        assert_eq!(p.score, None);
    }
}
