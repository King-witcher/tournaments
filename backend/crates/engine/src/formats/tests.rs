use super::*;
use crate::formats::elimination::EliminationKind;
use crate::formats::table::Tiebreakers;
use crate::series::SeriesLength;

fn stage(kind: EliminationKind) -> StageFormat {
    StageFormat::Elimination(EliminationConfig {
        kind,
        series: SeriesLength {
            default: 3,
            finals: 5,
        },
    })
}

fn kinds() -> [StageFormat; 3] {
    [
        stage(EliminationKind::Single),
        stage(EliminationKind::Double {
            grand_final_reset: true,
        }),
        stage(EliminationKind::Double {
            grand_final_reset: false,
        }),
    ]
}

#[test]
fn longest_series_is_the_max() {
    assert_eq!(kinds()[0].longest_series(), 5);
}

#[test]
fn worst_case_matches_the_built_bracket() {
    for f in kinds() {
        let StageFormat::Elimination(c) = f else {
            unreachable!()
        };
        for n in [2usize, 5, 8, 16] {
            let ids = (1..=n as i64).map(ParticipantId).collect();
            let expected = EliminationBracket::new(c, ids).unwrap().worst_case_games();
            assert_eq!(f.worst_case_games(n), Ok(expected));
        }
    }
}

#[test]
fn worst_case_errors_propagate() {
    let f = kinds()[0];
    assert_eq!(
        f.worst_case_games(1),
        Err(EngineError::NotEnoughParticipants { min: 2, got: 1 })
    );
    assert_eq!(
        f.worst_case_games(1025),
        Err(EngineError::TooManyParticipants {
            max: 1024,
            got: 1025
        })
    );
}

#[test]
fn worst_case_rejects_huge_counts_without_allocating() {
    assert_eq!(
        kinds()[0].worst_case_games(usize::MAX),
        Err(EngineError::TooManyParticipants {
            max: 1024,
            got: usize::MAX
        })
    );
}

#[test]
fn worst_case_games_is_monotonic_in_participants() {
    let mut points: Vec<usize> = (2..=130).collect();
    points.extend([255, 256, 257, 1023, 1024]);
    for f in kinds() {
        let mut prev: Option<(usize, u32)> = None;
        for &n in &points {
            let w = f.worst_case_games(n).unwrap();
            if let Some((pn, pw)) = prev
                && n == pn + 1
            {
                assert!(pw <= w, "{f:?}: n={pn} -> {pw}, n={n} -> {w}");
            }
            prev = Some((n, w));
        }
    }
}

fn table_formats() -> [StageFormat; 3] {
    [
        StageFormat::RoundRobin(RoundRobinConfig {
            best_of: 3,
            tiebreakers: Tiebreakers::ROUND_ROBIN,
        }),
        StageFormat::Swiss(SwissConfig {
            best_of: 3,
            rounds: None,
            tiebreakers: Tiebreakers::SWISS,
        }),
        StageFormat::Swiss(SwissConfig {
            best_of: 1,
            rounds: Some(9),
            tiebreakers: Tiebreakers::SWISS,
        }),
    ]
}

#[test]
fn table_formats_validate_and_have_one_series_length() {
    for f in table_formats() {
        assert_eq!(f.validate(), Ok(()));
    }
    assert_eq!(table_formats()[0].longest_series(), 3);
    assert_eq!(table_formats()[2].longest_series(), 1);
    let none = StageFormat::RoundRobin(RoundRobinConfig {
        best_of: 0,
        tiebreakers: Tiebreakers::ROUND_ROBIN,
    });
    assert_eq!(none.validate(), Err(EngineError::InvalidBestOf(0)));
    let no_rounds = StageFormat::Swiss(SwissConfig {
        best_of: 3,
        rounds: Some(0),
        tiebreakers: Tiebreakers::SWISS,
    });
    assert_eq!(
        no_rounds.validate(),
        Err(EngineError::InvalidRounds { max: 127, got: 0 })
    );
    assert_eq!(
        no_rounds.worst_case_games(8),
        Err(EngineError::InvalidRounds { max: 127, got: 0 })
    );
}

#[test]
fn even_series_are_only_for_the_round_robin() {
    let single = |default, finals| {
        StageFormat::Elimination(EliminationConfig {
            kind: EliminationKind::Single,
            series: SeriesLength { default, finals },
        })
    };
    // N − 1 séries, com todos os mapas no pior caso.
    for (best_of, worst) in [(2u8, 14), (4, 28), (6, 42)] {
        let league = StageFormat::RoundRobin(RoundRobinConfig {
            best_of,
            tiebreakers: Tiebreakers::ROUND_ROBIN,
        });
        assert_eq!(league.validate(), Ok(()));
        assert_eq!(league.longest_series(), best_of);
        assert_eq!(league.worst_case_games(8), Ok(worst));

        let invalid = Err(EngineError::InvalidBestOf(best_of));
        let swiss = StageFormat::Swiss(SwissConfig {
            best_of,
            rounds: None,
            tiebreakers: Tiebreakers::SWISS,
        });
        assert_eq!(swiss.validate(), invalid);
        assert_eq!(single(best_of, 3).validate(), invalid);
        assert_eq!(single(3, best_of).validate(), invalid);
    }
    // Qualquer N par no round robin, como no ímpar; md0 não existe.
    for (best_of, result) in [(8u8, Ok(())), (0, Err(EngineError::InvalidBestOf(0)))] {
        let league = StageFormat::RoundRobin(RoundRobinConfig {
            best_of,
            tiebreakers: Tiebreakers::ROUND_ROBIN,
        });
        assert_eq!(league.validate(), result, "md{best_of}");
    }
}

#[test]
fn table_formats_worst_case() {
    let [rr, swiss, long] = table_formats();
    // Round robin: N − 1 séries; suíço: uma por rodada.
    assert_eq!(rr.worst_case_games(8), Ok(21));
    assert_eq!(swiss.worst_case_games(8), Ok(9));
    assert_eq!(long.worst_case_games(8), Ok(7));
    assert_eq!(long.worst_case_games(16), Ok(9));
    assert_eq!(
        rr.worst_case_games(33),
        Err(EngineError::TooManyParticipants { max: 32, got: 33 })
    );
    assert_eq!(
        swiss.worst_case_games(129),
        Err(EngineError::TooManyParticipants { max: 128, got: 129 })
    );
    assert_eq!(
        swiss.worst_case_games(usize::MAX),
        Err(EngineError::TooManyParticipants {
            max: 128,
            got: usize::MAX
        })
    );
    for f in table_formats() {
        assert_eq!(
            f.worst_case_games(1),
            Err(EngineError::NotEnoughParticipants { min: 2, got: 1 })
        );
        let mut prev = 0;
        let max = if matches!(f, StageFormat::RoundRobin(_)) {
            32
        } else {
            128
        };
        for n in 2..=max {
            let w = f.worst_case_games(n).unwrap();
            assert!(prev <= w, "{f:?}: n={n}");
            prev = w;
            let seeded = (1..=n as i64).map(ParticipantId).collect();
            assert_eq!(StageBracket::new(f, seeded).unwrap().worst_case_games(), w);
        }
    }
}
