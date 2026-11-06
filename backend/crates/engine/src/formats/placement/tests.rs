//! Testes da disputa de posições: pares, colocação, W.O. e recusas.

use proptest::prelude::*;

use super::*;
use crate::core::{Outcome, SeriesScore};
use crate::series::wins_needed;

fn p(n: i64) -> ParticipantId {
    ParticipantId(n)
}

fn seeds(n: i64) -> Vec<ParticipantId> {
    (1..=n).map(p).collect()
}

fn id(index: u16) -> MatchId {
    MatchId {
        side: BracketSide::Placement,
        round: 1,
        index,
    }
}

fn config() -> PlacementConfig {
    PlacementConfig { best_of: 3 }
}

fn played(index: u16, winner: i64, loser_maps: u8) -> MatchReport {
    MatchReport {
        match_id: id(index),
        outcome: Outcome::Played {
            winner: p(winner),
            score: SeriesScore {
                winner: wins_needed(3),
                loser: loser_maps,
            },
        },
    }
}

/// `(rank, participante)` de cada colocação, na ordem.
fn ranks(b: &PlacementBracket) -> Vec<(u16, i64)> {
    b.placements()
        .iter()
        .map(|x| {
            assert_eq!(x.tied, 1);
            (x.rank, x.participant.0)
        })
        .collect()
}

#[test]
fn pairs_follow_the_seed_order_in_one_round() {
    let b = PlacementBracket::new(config(), seeds(6)).unwrap();
    let pairs: Vec<_> = b
        .matches()
        .iter()
        .map(|m| match m.state {
            MatchState::Ready { a, b } => (m.id, m.best_of, a.0, b.0),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        pairs,
        [(id(1), 3, 1, 2), (id(2), 3, 3, 4), (id(3), 3, 5, 6)]
    );
    assert_eq!(b.ready_count(), 3);
    assert!(!b.is_finished());
    assert!(b.placements().is_empty());
    assert_eq!(
        b.occupants(id(2)),
        Some((Occupant::Participant(p(3)), Occupant::Participant(p(4))))
    );
    assert_eq!(b.get(id(4)), None);
}

#[test]
fn odd_count_leaves_the_last_seed_in_place() {
    let b = PlacementBracket::new(config(), seeds(5)).unwrap();
    assert_eq!(b.matches().len(), 3);
    let last = b.matches()[2];
    assert_eq!(last.id, id(3));
    assert_eq!(last.a, Slot::Participant(p(5)));
    assert_eq!(last.b, Slot::Bye);
    assert_eq!(
        last.state,
        MatchState::Bye {
            advanced: Some(p(5))
        }
    );
    assert_eq!(
        b.occupants(id(3)),
        Some((Occupant::Participant(p(5)), Occupant::Bye))
    );
    // Sem jogar, fica na posição dela desde o início.
    assert_eq!(ranks(&b), [(5, 5)]);
    assert_eq!(b.ready_count(), 2);
}

#[test]
fn the_winner_takes_the_better_place() {
    let mut b = PlacementBracket::new(config(), seeds(5)).unwrap();
    // O 4º vence o 3º e termina em 3º.
    b.report(played(2, 4, 1)).unwrap();
    assert_eq!(ranks(&b), [(3, 4), (4, 3), (5, 5)]);
    assert!(!b.is_finished());
    b.report(played(1, 1, 0)).unwrap();
    assert!(b.is_finished());
    assert_eq!(ranks(&b), [(1, 1), (2, 2), (3, 4), (4, 3), (5, 5)]);
}

#[test]
fn walkover_counts_as_a_win() {
    let mut b = PlacementBracket::new(config(), seeds(4)).unwrap();
    b.report(MatchReport {
        match_id: id(1),
        outcome: Outcome::Walkover { winner: p(2) },
    })
    .unwrap();
    b.report(played(2, 3, 1)).unwrap();
    assert!(b.is_finished());
    assert_eq!(ranks(&b), [(1, 2), (2, 1), (3, 3), (4, 4)]);
}

#[test]
fn replay_matches_the_reports_and_a_correction_moves_only_its_pair() {
    let reports = [played(2, 4, 0), played(1, 2, 1)];
    let direct = {
        let mut b = PlacementBracket::new(config(), seeds(4)).unwrap();
        for r in reports {
            b.report(r).unwrap();
        }
        b
    };
    assert_eq!(
        PlacementBracket::replay(config(), seeds(4), reports).unwrap(),
        direct
    );
    let corrected =
        PlacementBracket::replay(config(), seeds(4), [played(2, 3, 0), reports[1]]).unwrap();
    assert_eq!(ranks(&corrected), [(1, 2), (2, 1), (3, 3), (4, 4)]);
}

#[test]
fn even_series_are_refused() {
    for best_of in [0, 2, 4, 6] {
        let config = PlacementConfig { best_of };
        let invalid = EngineError::InvalidBestOf(best_of);
        assert_eq!(config.validate(), Err(invalid.clone()));
        assert_eq!(
            PlacementBracket::new(config, seeds(4)),
            Err(invalid.clone())
        );
        assert_eq!(config.worst_case_games(4), Err(invalid));
    }
    for best_of in [1, 3, 5, 7, 9] {
        assert_eq!(PlacementConfig { best_of }.validate(), Ok(()));
    }
}

#[test]
fn from_two_to_the_ceiling() {
    assert_eq!(
        PlacementBracket::new(config(), seeds(1)),
        Err(EngineError::NotEnoughParticipants { min: 2, got: 1 })
    );
    assert_eq!(
        PlacementBracket::new(config(), seeds(129)),
        Err(EngineError::TooManyParticipants { max: 128, got: 129 })
    );
    assert_eq!(
        config().worst_case_games(129),
        Err(EngineError::TooManyParticipants { max: 128, got: 129 })
    );
    assert_eq!(
        config().worst_case_games(usize::MAX),
        Err(EngineError::TooManyParticipants {
            max: 128,
            got: usize::MAX
        })
    );
    assert_eq!(
        PlacementBracket::new(config(), vec![p(1), p(2), p(1)]),
        Err(EngineError::DuplicateParticipant(p(1)))
    );
    let two = PlacementBracket::new(config(), seeds(2)).unwrap();
    assert_eq!(two.matches().len(), 1);
    assert_eq!(
        PlacementBracket::new(config(), seeds(128))
            .unwrap()
            .matches()
            .len(),
        64
    );
}

#[test]
fn worst_case_is_one_series() {
    for n in [2, 3, 64, 128] {
        assert_eq!(PlacementConfig { best_of: 5 }.worst_case_games(n), Ok(5));
    }
    let b = PlacementBracket::new(PlacementConfig { best_of: 7 }, seeds(9)).unwrap();
    assert_eq!(b.worst_case_games(), 7);
}

#[test]
fn bad_reports_are_refused() {
    let mut b = PlacementBracket::new(config(), seeds(5)).unwrap();
    assert_eq!(
        b.report(played(4, 1, 0)),
        Err(EngineError::UnknownMatch(id(4)))
    );
    // O sem par não joga.
    assert_eq!(
        b.report(MatchReport {
            match_id: id(3),
            outcome: Outcome::Walkover { winner: p(5) },
        }),
        Err(EngineError::MatchNotReady(id(3)))
    );
    assert_eq!(
        b.report(played(1, 3, 0)),
        Err(EngineError::NotInMatch {
            match_id: id(1),
            participant: p(3)
        })
    );
    let score = SeriesScore {
        winner: 3,
        loser: 0,
    };
    assert_eq!(
        b.report(MatchReport {
            match_id: id(1),
            outcome: Outcome::Played {
                winner: p(1),
                score
            },
        }),
        Err(EngineError::InvalidScore { best_of: 3, score })
    );
    // Série ímpar não empata.
    assert!(matches!(
        b.report(MatchReport {
            match_id: id(1),
            outcome: Outcome::Drawn { maps: 1 },
        }),
        Err(EngineError::InvalidScore { .. })
    ));
    b.report(played(1, 1, 1)).unwrap();
    assert_eq!(
        b.report(played(1, 2, 0)),
        Err(EngineError::MatchAlreadyReported(id(1)))
    );
}

impl PlacementBracket {
    fn ready_count(&self) -> usize {
        self.matches
            .iter()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
            .count()
    }
}

proptest! {
    /// Com qualquer N e quaisquer vencedores: cada um joga no máximo uma
    /// série, todos as do início, e no fim as posições são 1..N, uma cada,
    /// com o par k em 2k − 1 e 2k.
    #[test]
    fn everyone_gets_one_place(n in 2i64..=128, picks in proptest::collection::vec(any::<bool>(), 64)) {
        let mut b = PlacementBracket::new(config(), seeds(n)).unwrap();
        let ready: Vec<Match> = b.matches().iter().copied()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
            .collect();
        prop_assert_eq!(ready.len() as i64, n / 2);
        let mut seen = std::collections::HashSet::new();
        for m in b.matches() {
            for slot in [m.a, m.b] {
                if let Slot::Participant(x) = slot {
                    prop_assert!(seen.insert(x));
                }
            }
        }
        prop_assert_eq!(seen.len() as i64, n);
        for (m, upset) in ready.iter().zip(&picks) {
            let MatchState::Ready { a, b: other } = m.state else { unreachable!() };
            let winner = if *upset { other } else { a };
            b.report(MatchReport {
                match_id: m.id,
                outcome: Outcome::Played {
                    winner,
                    score: SeriesScore { winner: 2, loser: 1 },
                },
            }).unwrap();
        }
        prop_assert!(b.is_finished());
        let mut placements = b.placements();
        placements.sort_by_key(|x| x.rank);
        let ranks: Vec<u16> = placements.iter().map(|x| x.rank).collect();
        prop_assert_eq!(ranks, (1..=n as u16).collect::<Vec<_>>());
        for x in &placements {
            // O par k tem os seeds 2k − 1 e 2k e as posições 2k − 1 e 2k.
            let seed = x.participant.0 as u16;
            prop_assert_eq!(seed.div_ceil(2), x.rank.div_ceil(2));
        }
    }
}
