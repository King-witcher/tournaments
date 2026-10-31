//! Conversão entre os DTOs de formato e regra de campeões e os tipos do
//! `engine`, que são os que ficam gravados (JSON) e os que validam.

use engine::{
    champions as ec,
    formats::{self as ef, elimination as el, round_robin::RoundRobinConfig, swiss::SwissConfig},
    series::SeriesLength,
};

use super::types::{Bracket, ChampionRules, NoRepeatRule, NoRepeatScope, NoRepeatWho, StageFormat};

impl From<StageFormat> for ef::StageFormat {
    fn from(stage: StageFormat) -> Self {
        match stage {
            StageFormat::Elimination {
                bracket,
                grand_final_reset,
                best_of,
                finals_best_of,
            } => Self::Elimination(el::EliminationConfig {
                kind: match bracket {
                    Bracket::Single => el::EliminationKind::Single,
                    Bracket::Double => el::EliminationKind::Double { grand_final_reset },
                },
                series: SeriesLength {
                    default: best_of,
                    finals: finals_best_of,
                },
            }),
            StageFormat::RoundRobin { best_of } => Self::RoundRobin(RoundRobinConfig { best_of }),
            StageFormat::Swiss { best_of, rounds } => Self::Swiss(SwissConfig { best_of, rounds }),
        }
    }
}

impl From<ef::StageFormat> for StageFormat {
    fn from(stage: ef::StageFormat) -> Self {
        match stage {
            ef::StageFormat::Elimination(config) => {
                let (bracket, grand_final_reset) = match config.kind {
                    el::EliminationKind::Single => (Bracket::Single, false),
                    el::EliminationKind::Double { grand_final_reset } => {
                        (Bracket::Double, grand_final_reset)
                    }
                };
                Self::Elimination {
                    bracket,
                    grand_final_reset,
                    best_of: config.series.default,
                    finals_best_of: config.series.finals,
                }
            }
            ef::StageFormat::RoundRobin(c) => Self::RoundRobin { best_of: c.best_of },
            ef::StageFormat::Swiss(c) => Self::Swiss {
                best_of: c.best_of,
                rounds: c.rounds,
            },
        }
    }
}

impl From<ChampionRules> for ec::ChampionRules {
    fn from(rules: ChampionRules) -> Self {
        Self {
            bans_per_series: rules.bans_per_series.into(),
            mirror: rules.mirror,
            same_team: rules.same_team,
            no_repeat: rules.no_repeat.map(|r| ec::NoRepeatRule {
                who: match r.who {
                    NoRepeatWho::Player => ec::NoRepeatWho::Player,
                    NoRepeatWho::Team => ec::NoRepeatWho::Team,
                },
                scope: match r.scope {
                    NoRepeatScope::Series => ec::NoRepeatScope::Series,
                    NoRepeatScope::Stage => ec::NoRepeatScope::Stage,
                    NoRepeatScope::Tournament => ec::NoRepeatScope::Tournament,
                },
            }),
        }
    }
}

impl From<ec::ChampionRules> for ChampionRules {
    fn from(rules: ec::ChampionRules) -> Self {
        Self {
            // Gravado só depois de validado (0 a 3).
            bans_per_series: rules.bans_per_series.try_into().unwrap_or(u8::MAX),
            mirror: rules.mirror,
            same_team: rules.same_team,
            no_repeat: rules.no_repeat.map(|r| NoRepeatRule {
                who: match r.who {
                    ec::NoRepeatWho::Player => NoRepeatWho::Player,
                    ec::NoRepeatWho::Team => NoRepeatWho::Team,
                },
                scope: match r.scope {
                    ec::NoRepeatScope::Series => NoRepeatScope::Series,
                    ec::NoRepeatScope::Stage => NoRepeatScope::Stage,
                    ec::NoRepeatScope::Tournament => NoRepeatScope::Tournament,
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_elimination_drops_the_reset() {
        let dto = StageFormat::Elimination {
            bracket: Bracket::Single,
            grand_final_reset: true,
            best_of: 3,
            finals_best_of: 5,
        };
        let back = StageFormat::from(ef::StageFormat::from(dto));
        assert_eq!(
            back,
            StageFormat::Elimination {
                bracket: Bracket::Single,
                grand_final_reset: false,
                best_of: 3,
                finals_best_of: 5,
            }
        );
    }

    #[test]
    fn table_formats_round_trip() {
        for dto in [
            StageFormat::RoundRobin { best_of: 3 },
            StageFormat::Swiss {
                best_of: 1,
                rounds: None,
            },
            StageFormat::Swiss {
                best_of: 5,
                rounds: Some(4),
            },
        ] {
            assert_eq!(StageFormat::from(ef::StageFormat::from(dto)), dto);
        }
    }

    #[test]
    fn champion_rules_round_trip() {
        let dto = ChampionRules {
            bans_per_series: 2,
            mirror: false,
            same_team: true,
            no_repeat: Some(NoRepeatRule {
                who: NoRepeatWho::Player,
                scope: NoRepeatScope::Tournament,
            }),
        };
        assert_eq!(ChampionRules::from(ec::ChampionRules::from(dto)), dto);
    }
}
