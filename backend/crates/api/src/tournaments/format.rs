//! Conversão entre os DTOs de formato e regra de campeões e os tipos do
//! `engine`, que são os que ficam gravados (JSON) e os que validam.

use engine::{
    champions as ec,
    formats::{self as ef, elimination as el},
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
        }
    }
}

impl From<ChampionRules> for ec::ChampionRules {
    fn from(rules: ChampionRules) -> Self {
        Self {
            bans_per_game: rules.bans_per_game.into(),
            no_repeat: rules.no_repeat.map(|r| ec::NoRepeatRule {
                who: match r.who {
                    NoRepeatWho::Player => ec::NoRepeatWho::Player,
                    NoRepeatWho::Team => ec::NoRepeatWho::Team,
                },
                scope: match r.scope {
                    NoRepeatScope::Match => ec::NoRepeatScope::Match,
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
            bans_per_game: rules.bans_per_game.try_into().unwrap_or(u8::MAX),
            no_repeat: rules.no_repeat.map(|r| NoRepeatRule {
                who: match r.who {
                    ec::NoRepeatWho::Player => NoRepeatWho::Player,
                    ec::NoRepeatWho::Team => NoRepeatWho::Team,
                },
                scope: match r.scope {
                    ec::NoRepeatScope::Match => NoRepeatScope::Match,
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
    fn champion_rules_round_trip() {
        let dto = ChampionRules {
            bans_per_game: 2,
            no_repeat: Some(NoRepeatRule {
                who: NoRepeatWho::Player,
                scope: NoRepeatScope::Tournament,
            }),
        };
        assert_eq!(ChampionRules::from(ec::ChampionRules::from(dto)), dto);
    }
}
