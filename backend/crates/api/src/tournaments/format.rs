//! Conversão entre os DTOs de formato e regra de campeões e os tipos do
//! `engine`, que são os que validam. Como eles ficam gravados: `db::columns`.

use engine::{
    champions as ec,
    core::EngineError,
    formats::{
        self as ef, elimination as el,
        placement::PlacementConfig,
        round_robin::RoundRobinConfig,
        swiss::SwissConfig,
        table::{Criterion, Tiebreakers},
    },
    series::SeriesLength,
};

use super::types::{
    Bracket, ChampionRules, NoRepeatRule, NoRepeatScope, NoRepeatWho, StageFormat, Tiebreaker,
};

impl Tiebreaker {
    fn criterion(self) -> Criterion {
        match self {
            Self::HeadToHead => Criterion::HeadToHead,
            Self::MapDiff => Criterion::MapDiff,
            Self::FragDiff => Criterion::FragDiff,
            Self::Buchholz => Criterion::Buchholz,
        }
    }

    /// Os desempates do `engine` como a API os mostra. A lista só tem
    /// desempates (`Tiebreakers::new`), então nada fica de fora.
    pub(super) fn list(tiebreakers: Tiebreakers) -> Vec<Self> {
        tiebreakers
            .as_slice()
            .iter()
            .filter_map(|c| match c {
                Criterion::HeadToHead => Some(Self::HeadToHead),
                Criterion::MapDiff => Some(Self::MapDiff),
                Criterion::FragDiff => Some(Self::FragDiff),
                Criterion::Buchholz => Some(Self::Buchholz),
                Criterion::SeriesDiff | Criterion::Points | Criterion::Seed => None,
            })
            .collect()
    }
}

/// Erro: `InvalidTiebreakers` com desempate repetido (o Buchholz fora do
/// suíço é do `validate` do formato, como o resto da configuração).
fn tiebreakers(list: &[Tiebreaker]) -> Result<Tiebreakers, EngineError> {
    let criteria: Vec<Criterion> = list.iter().map(|t| t.criterion()).collect();
    Tiebreakers::new(&criteria)
}

impl TryFrom<StageFormat> for ef::StageFormat {
    type Error = EngineError;

    fn try_from(stage: StageFormat) -> Result<Self, EngineError> {
        Ok(match stage {
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
            StageFormat::RoundRobin {
                best_of,
                tiebreakers: list,
            } => Self::RoundRobin(RoundRobinConfig {
                best_of,
                tiebreakers: tiebreakers(&list)?,
            }),
            StageFormat::Swiss {
                best_of,
                rounds,
                tiebreakers: list,
            } => Self::Swiss(SwissConfig {
                best_of,
                rounds,
                tiebreakers: tiebreakers(&list)?,
            }),
            StageFormat::Placement { best_of } => Self::Placement(PlacementConfig { best_of }),
        })
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
            ef::StageFormat::RoundRobin(c) => Self::RoundRobin {
                best_of: c.best_of,
                tiebreakers: Tiebreaker::list(c.tiebreakers),
            },
            ef::StageFormat::Swiss(c) => Self::Swiss {
                best_of: c.best_of,
                rounds: c.rounds,
                tiebreakers: Tiebreaker::list(c.tiebreakers),
            },
            ef::StageFormat::Placement(c) => Self::Placement { best_of: c.best_of },
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

    fn round_trip(dto: StageFormat) -> StageFormat {
        StageFormat::from(ef::StageFormat::try_from(dto).unwrap())
    }

    #[test]
    fn single_elimination_drops_the_reset() {
        let dto = StageFormat::Elimination {
            bracket: Bracket::Single,
            grand_final_reset: true,
            best_of: 3,
            finals_best_of: 5,
        };
        assert_eq!(
            round_trip(dto),
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
            StageFormat::RoundRobin {
                best_of: 3,
                tiebreakers: vec![Tiebreaker::MapDiff, Tiebreaker::HeadToHead],
            },
            StageFormat::RoundRobin {
                best_of: 2,
                tiebreakers: vec![],
            },
            StageFormat::Swiss {
                best_of: 1,
                rounds: None,
                tiebreakers: vec![Tiebreaker::FragDiff, Tiebreaker::Buchholz],
            },
            StageFormat::Swiss {
                best_of: 5,
                rounds: Some(4),
                tiebreakers: Tiebreaker::list(Tiebreakers::SWISS),
            },
            StageFormat::Placement { best_of: 3 },
        ] {
            assert_eq!(round_trip(dto.clone()), dto);
        }
    }

    #[test]
    fn absent_tiebreakers_are_the_format_default() {
        let read = |json: serde_json::Value| -> ef::StageFormat {
            serde_json::from_value::<StageFormat>(json)
                .unwrap()
                .try_into()
                .unwrap()
        };
        let rr = read(serde_json::json!({ "format": "round_robin", "best_of": 3 }));
        let swiss = read(serde_json::json!({ "format": "swiss", "best_of": 3 }));
        let (ef::StageFormat::RoundRobin(rr), ef::StageFormat::Swiss(swiss)) = (rr, swiss) else {
            panic!("formato trocado");
        };
        assert_eq!(rr.tiebreakers, Tiebreakers::ROUND_ROBIN);
        assert_eq!(swiss.tiebreakers, Tiebreakers::SWISS);
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
