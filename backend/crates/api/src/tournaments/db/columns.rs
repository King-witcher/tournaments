//! A configuração do torneio nas colunas do banco: as regras de campeões em
//! `tournaments` e cada fase em `tournament_stages`, onde `format` diz quais
//! colunas valem (check `tournament_stages_shape`). Único lugar que traduz
//! entre as colunas e os tipos do `engine`: um campo novo neles não compila
//! aqui até ganhar coluna.

use engine::{
    champions as ec,
    formats::{
        self as ef, elimination as el,
        placement::PlacementConfig,
        round_robin::RoundRobinConfig,
        swiss::SwissConfig,
        table::{Criterion, Tiebreakers},
    },
    series::SeriesLength,
};
use serde::Deserialize;

use crate::shared::ApiError;

/// O banco confere estas colunas: valor fora delas é erro nosso (500).
fn invalid(what: &'static str, value: &impl std::fmt::Debug) -> ApiError {
    tracing::error!(what, ?value, "configuração do torneio inválida no banco");
    ApiError::internal()
}

/// As regras de campeões como ficam em `tournaments`.
pub(super) struct ChampionColumns {
    pub bans_per_series: i16,
    pub mirror: bool,
    pub same_team: bool,
    pub no_repeat_who: Option<String>,
    pub no_repeat_scope: Option<String>,
}

impl From<ec::ChampionRules> for ChampionColumns {
    fn from(rules: ec::ChampionRules) -> Self {
        let (who, scope) = rules.no_repeat.map_or((None, None), |rule| {
            let who = match rule.who {
                ec::NoRepeatWho::Player => "player",
                ec::NoRepeatWho::Team => "team",
            };
            let scope = match rule.scope {
                ec::NoRepeatScope::Series => "series",
                ec::NoRepeatScope::Stage => "stage",
                ec::NoRepeatScope::Tournament => "tournament",
            };
            (Some(who.to_owned()), Some(scope.to_owned()))
        });
        Self {
            // Fora da faixa, o check do banco recusa.
            bans_per_series: i16::try_from(rules.bans_per_series).unwrap_or(i16::MAX),
            mirror: rules.mirror,
            same_team: rules.same_team,
            no_repeat_who: who,
            no_repeat_scope: scope,
        }
    }
}

impl ChampionColumns {
    pub(super) fn read(self) -> Result<ec::ChampionRules, ApiError> {
        let no_repeat = match (
            self.no_repeat_who.as_deref(),
            self.no_repeat_scope.as_deref(),
        ) {
            (None, None) => None,
            (Some(who), Some(scope)) => Some(ec::NoRepeatRule {
                who: match who {
                    "player" => ec::NoRepeatWho::Player,
                    "team" => ec::NoRepeatWho::Team,
                    other => return Err(invalid("no_repeat_who", &other)),
                },
                scope: match scope {
                    "series" => ec::NoRepeatScope::Series,
                    "stage" => ec::NoRepeatScope::Stage,
                    "tournament" => ec::NoRepeatScope::Tournament,
                    other => return Err(invalid("no_repeat_scope", &other)),
                },
            }),
            pair => return Err(invalid("no_repeat", &pair)),
        };
        Ok(ec::ChampionRules {
            bans_per_series: u32::try_from(self.bans_per_series)
                .map_err(|_| invalid("bans_per_series", &self.bans_per_series))?,
            mirror: self.mirror,
            same_team: self.same_team,
            no_repeat,
        })
    }
}

/// Uma fase como fica em `tournament_stages`, e como vem do JSON que as
/// queries montam na hora a partir das colunas (`find` e
/// `matches::db::stage_rows`, que leem a fase junto do resto).
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct StageRow {
    pub(super) format: String,
    pub(super) best_of: i16,
    pub(super) finals_best_of: Option<i16>,
    pub(super) bracket: Option<String>,
    pub(super) grand_final_reset: Option<bool>,
    pub(super) rounds: Option<i16>,
    /// Fases de tabela: os desempates, em ordem. Nulo na eliminação e na
    /// disputa de posições.
    pub(super) tiebreakers: Option<Vec<String>>,
}

/// Um desempate na coluna `tiebreakers` (check `tournament_stages_tiebreakers`).
fn tiebreaker_column(criterion: Criterion) -> &'static str {
    match criterion {
        Criterion::HeadToHead => "head_to_head",
        Criterion::MapDiff => "map_diff",
        Criterion::FragDiff => "frag_diff",
        Criterion::Buchholz => "buchholz",
        Criterion::LowerRating => "lower_rating",
        // `Tiebreakers` só guarda desempates; o check recusaria.
        Criterion::SeriesDiff => "series_diff",
        Criterion::Points => "points",
        Criterion::Seed => "seed",
    }
}

fn tiebreaker_columns(tiebreakers: Tiebreakers) -> Option<Vec<String>> {
    Some(
        tiebreakers
            .as_slice()
            .iter()
            .map(|&c| tiebreaker_column(c).to_owned())
            .collect(),
    )
}

impl From<ef::StageFormat> for StageRow {
    fn from(stage: ef::StageFormat) -> Self {
        match stage {
            ef::StageFormat::Elimination(config) => {
                let (bracket, grand_final_reset) = match config.kind {
                    el::EliminationKind::Single => ("single", None),
                    el::EliminationKind::Double { grand_final_reset } => {
                        ("double", Some(grand_final_reset))
                    }
                };
                Self {
                    format: "elimination".to_owned(),
                    best_of: config.series.default.into(),
                    finals_best_of: Some(config.series.finals.into()),
                    bracket: Some(bracket.to_owned()),
                    grand_final_reset,
                    rounds: None,
                    tiebreakers: None,
                }
            }
            ef::StageFormat::RoundRobin(config) => Self {
                format: "round_robin".to_owned(),
                best_of: config.best_of.into(),
                finals_best_of: None,
                bracket: None,
                grand_final_reset: None,
                rounds: None,
                tiebreakers: tiebreaker_columns(config.tiebreakers),
            },
            ef::StageFormat::Swiss(config) => Self {
                format: "swiss".to_owned(),
                best_of: config.best_of.into(),
                finals_best_of: None,
                bracket: None,
                grand_final_reset: None,
                // Fora da faixa, o check do banco recusa.
                rounds: config.rounds.map(|n| i16::try_from(n).unwrap_or(i16::MAX)),
                tiebreakers: tiebreaker_columns(config.tiebreakers),
            },
            ef::StageFormat::Placement(config) => Self {
                format: "placement".to_owned(),
                best_of: config.best_of.into(),
                finals_best_of: None,
                bracket: None,
                grand_final_reset: None,
                rounds: None,
                tiebreakers: None,
            },
        }
    }
}

impl StageRow {
    /// Os desempates de uma fase de tabela.
    fn tiebreakers(&self) -> Result<Tiebreakers, ApiError> {
        let Some(columns) = &self.tiebreakers else {
            return Err(invalid("tiebreakers", &self));
        };
        let criteria = columns
            .iter()
            .map(|c| match c.as_str() {
                "head_to_head" => Ok(Criterion::HeadToHead),
                "map_diff" => Ok(Criterion::MapDiff),
                "frag_diff" => Ok(Criterion::FragDiff),
                "buchholz" => Ok(Criterion::Buchholz),
                "lower_rating" => Ok(Criterion::LowerRating),
                _ => Err(invalid("tiebreakers", &self)),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Tiebreakers::new(&criteria).map_err(|_| invalid("tiebreakers", &self))
    }

    pub fn read(&self) -> Result<ef::StageFormat, ApiError> {
        let best_of = |n: i16| u8::try_from(n).map_err(|_| invalid("best_of", &self));
        match self.format.as_str() {
            "elimination" => {
                let finals = self
                    .finals_best_of
                    .ok_or_else(|| invalid("finals_best_of", &self))?;
                let kind = match (self.bracket.as_deref(), self.grand_final_reset) {
                    (Some("single"), None) => el::EliminationKind::Single,
                    (Some("double"), Some(grand_final_reset)) => {
                        el::EliminationKind::Double { grand_final_reset }
                    }
                    _ => return Err(invalid("bracket", &self)),
                };
                if self.tiebreakers.is_some() {
                    return Err(invalid("tiebreakers", &self));
                }
                Ok(ef::StageFormat::Elimination(el::EliminationConfig {
                    kind,
                    series: SeriesLength {
                        default: best_of(self.best_of)?,
                        finals: best_of(finals)?,
                    },
                }))
            }
            "round_robin" => Ok(ef::StageFormat::RoundRobin(RoundRobinConfig {
                best_of: best_of(self.best_of)?,
                tiebreakers: self.tiebreakers()?,
            })),
            "swiss" => Ok(ef::StageFormat::Swiss(SwissConfig {
                best_of: best_of(self.best_of)?,
                rounds: self
                    .rounds
                    .map(u16::try_from)
                    .transpose()
                    .map_err(|_| invalid("rounds", &self))?,
                tiebreakers: self.tiebreakers()?,
            })),
            "placement" if self.tiebreakers.is_none() => {
                Ok(ef::StageFormat::Placement(PlacementConfig {
                    best_of: best_of(self.best_of)?,
                }))
            }
            "placement" => Err(invalid("tiebreakers", &self)),
            _ => Err(invalid("format", &self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elimination(kind: el::EliminationKind) -> ef::StageFormat {
        ef::StageFormat::Elimination(el::EliminationConfig {
            kind,
            series: SeriesLength {
                default: 3,
                finals: 5,
            },
        })
    }

    #[test]
    fn every_format_reads_back_what_it_writes() {
        for stage in [
            elimination(el::EliminationKind::Single),
            elimination(el::EliminationKind::Double {
                grand_final_reset: true,
            }),
            elimination(el::EliminationKind::Double {
                grand_final_reset: false,
            }),
            ef::StageFormat::RoundRobin(RoundRobinConfig {
                best_of: 3,
                tiebreakers: Tiebreakers::ROUND_ROBIN,
            }),
            ef::StageFormat::RoundRobin(RoundRobinConfig {
                best_of: 2,
                tiebreakers: Tiebreakers::NONE,
            }),
            ef::StageFormat::RoundRobin(RoundRobinConfig {
                best_of: 4,
                tiebreakers: Tiebreakers::new(&[Criterion::LowerRating, Criterion::HeadToHead])
                    .unwrap(),
            }),
            ef::StageFormat::Swiss(SwissConfig {
                best_of: 1,
                rounds: None,
                tiebreakers: Tiebreakers::SWISS,
            }),
            ef::StageFormat::Swiss(SwissConfig {
                best_of: 255,
                rounds: Some(127),
                tiebreakers: Tiebreakers::new(&[Criterion::FragDiff, Criterion::Buchholz]).unwrap(),
            }),
            ef::StageFormat::Placement(PlacementConfig { best_of: 5 }),
        ] {
            assert_eq!(StageRow::from(stage).read().unwrap(), stage);
        }
    }

    #[test]
    fn champion_rules_read_back_what_they_write() {
        let rules = |no_repeat| ec::ChampionRules {
            bans_per_series: 2,
            mirror: true,
            same_team: false,
            no_repeat,
        };
        let cases = [
            None,
            Some(ec::NoRepeatRule {
                who: ec::NoRepeatWho::Player,
                scope: ec::NoRepeatScope::Series,
            }),
            Some(ec::NoRepeatRule {
                who: ec::NoRepeatWho::Team,
                scope: ec::NoRepeatScope::Stage,
            }),
            Some(ec::NoRepeatRule {
                who: ec::NoRepeatWho::Player,
                scope: ec::NoRepeatScope::Tournament,
            }),
        ];
        for no_repeat in cases {
            let rules = rules(no_repeat);
            assert_eq!(ChampionColumns::from(rules).read().unwrap(), rules);
        }
    }

    #[test]
    fn a_row_outside_the_check_is_an_error_not_a_guess() {
        let row = |format: &str, bracket: Option<&str>, reset: Option<bool>| StageRow {
            format: format.to_owned(),
            best_of: 3,
            finals_best_of: Some(5),
            bracket: bracket.map(str::to_owned),
            grand_final_reset: reset,
            rounds: None,
            tiebreakers: None,
        };
        assert!(row("ladder", None, None).read().is_err());
        assert!(row("elimination", None, None).read().is_err());
        assert!(row("elimination", Some("double"), None).read().is_err());
        assert!(
            row("elimination", Some("single"), Some(true))
                .read()
                .is_err()
        );
        // Os desempates: só nas fases de tabela, e só os que existem.
        let tiebreakers = |format: &str, list: Option<&[&str]>| StageRow {
            tiebreakers: list.map(|l| l.iter().map(|s| (*s).to_owned()).collect()),
            ..row(format, None, None)
        };
        assert!(tiebreakers("round_robin", None).read().is_err());
        assert!(tiebreakers("swiss", Some(&["seed"])).read().is_err());
        assert!(
            tiebreakers("swiss", Some(&["map_diff", "map_diff"]))
                .read()
                .is_err()
        );
        let single = StageRow {
            tiebreakers: Some(vec![]),
            ..row("elimination", Some("single"), None)
        };
        assert!(single.read().is_err());
        assert!(tiebreakers("placement", Some(&[])).read().is_err());
        let half_rule = ChampionColumns {
            bans_per_series: 1,
            mirror: true,
            same_team: false,
            no_repeat_who: Some("player".to_owned()),
            no_repeat_scope: None,
        };
        assert!(half_rule.read().is_err());
    }
}
