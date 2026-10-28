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
fn champion_rules_json_is_stable() {
    let with = ChampionRules {
        bans_per_game: 1,
        no_repeat: Some(NoRepeatRule {
            who: NoRepeatWho::Player,
            scope: NoRepeatScope::Tournament,
        }),
    };
    let json = r#"{"bans_per_game":1,"no_repeat":{"who":"player","scope":"tournament"}}"#;
    assert_eq!(serde_json::to_string(&with).unwrap(), json);
    assert_eq!(serde_json::from_str::<ChampionRules>(json).unwrap(), with);
    let without = ChampionRules {
        bans_per_game: 0,
        no_repeat: None,
    };
    let json = r#"{"bans_per_game":0,"no_repeat":null}"#;
    assert_eq!(serde_json::to_string(&without).unwrap(), json);
    assert_eq!(
        serde_json::from_str::<ChampionRules>(json).unwrap(),
        without
    );
}

#[test]
fn longest_series_is_the_max() {
    assert_eq!(kinds()[0].longest_series(), 5);
}

#[test]
fn worst_case_matches_the_built_bracket() {
    for f in kinds() {
        let StageFormat::Elimination(c) = f;
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
