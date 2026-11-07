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
        table::{self as et, Criterion, Tiebreakers},
    },
    series::SeriesLength,
    stages::StageSetup,
};

use super::types::{
    Bracket, ChampionRules, FinalTie, NoRepeatRule, NoRepeatScope, NoRepeatWho, StageFormat,
    Tiebreaker,
};

impl From<FinalTie> for et::FinalTie {
    fn from(f: FinalTie) -> Self {
        match f {
            FinalTie::Seed => Self::Seed,
            FinalTie::Organizer => Self::Organizer,
        }
    }
}

impl From<et::FinalTie> for FinalTie {
    fn from(f: et::FinalTie) -> Self {
        match f {
            et::FinalTie::Seed => Self::Seed,
            et::FinalTie::Organizer => Self::Organizer,
        }
    }
}

impl Tiebreaker {
    fn criterion(self) -> Criterion {
        match self {
            Self::HeadToHead => Criterion::HeadToHead,
            Self::MapDiff => Criterion::MapDiff,
            Self::FragDiff => Criterion::FragDiff,
            Self::Buchholz => Criterion::Buchholz,
            Self::LowerRating => Criterion::LowerRating,
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
                Criterion::LowerRating => Some(Self::LowerRating),
                Criterion::SeriesDiff
                | Criterion::Points
                | Criterion::Seed
                | Criterion::Organizer => None,
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

impl StageFormat {
    /// O corte e o check-in da fase.
    const fn entry(&self) -> (Option<u16>, bool) {
        match *self {
            Self::Elimination {
                advance, check_in, ..
            }
            | Self::RoundRobin {
                advance, check_in, ..
            }
            | Self::Swiss {
                advance, check_in, ..
            }
            | Self::Placement {
                advance, check_in, ..
            } => (advance, check_in),
        }
    }

    /// A mesma fase com o corte e o check-in `entry`.
    fn with_entry(mut self, (advance, check_in): (Option<u16>, bool)) -> Self {
        match &mut self {
            Self::Elimination {
                advance: a,
                check_in: c,
                ..
            }
            | Self::RoundRobin {
                advance: a,
                check_in: c,
                ..
            }
            | Self::Swiss {
                advance: a,
                check_in: c,
                ..
            }
            | Self::Placement {
                advance: a,
                check_in: c,
                ..
            } => {
                *a = advance;
                *c = check_in;
            }
        }
        self
    }
}

impl TryFrom<StageFormat> for StageSetup {
    type Error = EngineError;

    fn try_from(stage: StageFormat) -> Result<Self, EngineError> {
        let (advance, check_in) = stage.entry();
        Ok(Self {
            format: stage.try_into()?,
            advance,
            check_in,
        })
    }
}

impl From<StageSetup> for StageFormat {
    fn from(stage: StageSetup) -> Self {
        Self::from(stage.format).with_entry((stage.advance, stage.check_in))
    }
}

impl TryFrom<StageFormat> for ef::StageFormat {
    type Error = EngineError;

    /// Só o formato (o corte e o check-in ficam de fora).
    fn try_from(stage: StageFormat) -> Result<Self, EngineError> {
        Ok(match stage {
            StageFormat::Elimination {
                bracket,
                grand_final_reset,
                best_of,
                finals_best_of,
                ..
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
                final_tie,
                ..
            } => Self::RoundRobin(RoundRobinConfig {
                best_of,
                tiebreakers: tiebreakers(&list)?,
                final_tie: final_tie.into(),
            }),
            StageFormat::Swiss {
                best_of,
                rounds,
                tiebreakers: list,
                final_tie,
                ..
            } => Self::Swiss(SwissConfig {
                best_of,
                rounds,
                tiebreakers: tiebreakers(&list)?,
                final_tie: final_tie.into(),
            }),
            StageFormat::Placement { best_of, .. } => Self::Placement(PlacementConfig { best_of }),
        })
    }
}

impl From<ef::StageFormat> for StageFormat {
    /// O formato sem corte nem check-in.
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
                    advance: None,
                    check_in: false,
                }
            }
            ef::StageFormat::RoundRobin(c) => Self::RoundRobin {
                best_of: c.best_of,
                tiebreakers: Tiebreaker::list(c.tiebreakers),
                final_tie: c.final_tie.into(),
                advance: None,
                check_in: false,
            },
            ef::StageFormat::Swiss(c) => Self::Swiss {
                best_of: c.best_of,
                rounds: c.rounds,
                tiebreakers: Tiebreaker::list(c.tiebreakers),
                final_tie: c.final_tie.into(),
                advance: None,
                check_in: false,
            },
            ef::StageFormat::Placement(c) => Self::Placement {
                best_of: c.best_of,
                advance: None,
                check_in: false,
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

    fn round_trip(dto: StageFormat) -> StageFormat {
        StageFormat::from(StageSetup::try_from(dto).unwrap())
    }

    #[test]
    fn single_elimination_drops_the_reset() {
        let dto = StageFormat::Elimination {
            bracket: Bracket::Single,
            grand_final_reset: true,
            best_of: 3,
            finals_best_of: 5,
            advance: None,
            check_in: false,
        };
        assert_eq!(
            round_trip(dto),
            StageFormat::Elimination {
                bracket: Bracket::Single,
                grand_final_reset: false,
                best_of: 3,
                finals_best_of: 5,
                advance: None,
                check_in: false,
            }
        );
    }

    #[test]
    fn table_formats_round_trip() {
        for dto in [
            StageFormat::RoundRobin {
                best_of: 3,
                tiebreakers: vec![Tiebreaker::MapDiff, Tiebreaker::HeadToHead],
                final_tie: FinalTie::Organizer,
                advance: None,
                check_in: false,
            },
            StageFormat::RoundRobin {
                best_of: 2,
                tiebreakers: vec![],
                final_tie: FinalTie::Seed,
                advance: Some(4),
                check_in: true,
            },
            StageFormat::Swiss {
                best_of: 1,
                rounds: None,
                tiebreakers: vec![Tiebreaker::FragDiff, Tiebreaker::Buchholz],
                final_tie: FinalTie::Organizer,
                advance: None,
                check_in: true,
            },
            StageFormat::Swiss {
                best_of: 3,
                rounds: None,
                tiebreakers: vec![
                    Tiebreaker::LowerRating,
                    Tiebreaker::Buchholz,
                    Tiebreaker::HeadToHead,
                    Tiebreaker::MapDiff,
                    Tiebreaker::FragDiff,
                ],
                final_tie: FinalTie::Organizer,
                advance: Some(16),
                check_in: false,
            },
            StageFormat::Swiss {
                best_of: 5,
                rounds: Some(4),
                tiebreakers: Tiebreaker::list(Tiebreakers::SWISS),
                final_tie: FinalTie::Seed,
                advance: None,
                check_in: false,
            },
            StageFormat::Placement {
                best_of: 3,
                advance: Some(8),
                check_in: true,
            },
        ] {
            assert_eq!(round_trip(dto.clone()), dto);
        }
    }

    #[test]
    fn the_cut_and_check_in_are_optional_in_the_body() {
        let dto: StageFormat =
            serde_json::from_value(serde_json::json!({ "format": "placement", "best_of": 3 }))
                .unwrap();
        let stage = StageSetup::try_from(dto).unwrap();
        assert_eq!((stage.advance, stage.check_in), (None, false));
        let dto: StageFormat = serde_json::from_value(serde_json::json!({
            "format": "placement", "best_of": 3, "advance": 8, "check_in": true
        }))
        .unwrap();
        let stage = StageSetup::try_from(dto).unwrap();
        assert_eq!((stage.advance, stage.check_in), (Some(8), true));
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
        assert_eq!(rr.final_tie, et::FinalTie::Seed);
        assert_eq!(swiss.final_tie, et::FinalTie::Seed);
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
