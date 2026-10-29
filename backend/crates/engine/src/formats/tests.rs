use super::*;
use crate::champions::{ChampionRules, NoRepeatRule, NoRepeatScope, NoRepeatWho};
use crate::formats::elimination::EliminationKind;
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
fn stage_format_json_is_stable() {
    let f = stage(EliminationKind::Double {
        grand_final_reset: true,
    });
    let json = r#"{"format":"elimination","kind":{"kind":"double","grand_final_reset":true},"series":{"default":3,"finals":5}}"#;
    assert_eq!(serde_json::to_string(&f).unwrap(), json);
    assert_eq!(serde_json::from_str::<StageFormat>(json).unwrap(), f);
}

#[test]
fn table_formats_json_is_stable() {
    let cases = [
        (
            StageFormat::RoundRobin(RoundRobinConfig { best_of: 3 }),
            r#"{"format":"round_robin","best_of":3}"#,
        ),
        (
            StageFormat::Swiss(SwissConfig {
                best_of: 1,
                rounds: None,
            }),
            r#"{"format":"swiss","best_of":1,"rounds":null}"#,
        ),
        (
            StageFormat::Swiss(SwissConfig {
                best_of: 5,
                rounds: Some(6),
            }),
            r#"{"format":"swiss","best_of":5,"rounds":6}"#,
        ),
    ];
    for (f, json) in cases {
        assert_eq!(serde_json::to_string(&f).unwrap(), json);
        assert_eq!(serde_json::from_str::<StageFormat>(json).unwrap(), f);
    }
}

#[test]
fn champion_rules_json_is_stable() {
    let with = ChampionRules {
        bans_per_game: 1,
        no_repeat: Some(NoRepeatRule {
            who: NoRepeatWho::Player,
            scope: NoRepeatScope::Tournament,
        }),
        team_duplicates: false,
    };
    let json = r#"{"bans_per_game":1,"no_repeat":{"who":"player","scope":"tournament"},"team_duplicates":false}"#;
    assert_eq!(serde_json::to_string(&with).unwrap(), json);
    assert_eq!(serde_json::from_str::<ChampionRules>(json).unwrap(), with);
    let without = ChampionRules {
        bans_per_game: 0,
        no_repeat: None,
        team_duplicates: true,
    };
    let json = r#"{"bans_per_game":0,"no_repeat":null,"team_duplicates":true}"#;
    assert_eq!(serde_json::to_string(&without).unwrap(), json);
    assert_eq!(
        serde_json::from_str::<ChampionRules>(json).unwrap(),
        without
    );
    // Gravado antes do TDM, sem o campo: não repete no time.
    let old = r#"{"bans_per_game":0,"no_repeat":null}"#;
    assert!(
        !serde_json::from_str::<ChampionRules>(old)
            .unwrap()
            .team_duplicates
    );
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
        StageFormat::RoundRobin(RoundRobinConfig { best_of: 3 }),
        StageFormat::Swiss(SwissConfig {
            best_of: 3,
            rounds: None,
        }),
        StageFormat::Swiss(SwissConfig {
            best_of: 1,
            rounds: Some(9),
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
    let even = StageFormat::RoundRobin(RoundRobinConfig { best_of: 2 });
    assert_eq!(even.validate(), Err(EngineError::InvalidBestOf(2)));
    let no_rounds = StageFormat::Swiss(SwissConfig {
        best_of: 3,
        rounds: Some(0),
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
