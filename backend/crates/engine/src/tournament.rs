//! Validação da configuração de um torneio antes de gravar: fases, vagas e
//! regras de campeões.
//!
//! O pior caso de cada fase usa `max_participants`, um teto conservador: as
//! fases seguintes têm no máximo esse número de participantes.

use crate::champions::{self, ChampionBudget, ChampionRules, NoRepeatScope};
use crate::core::EngineError;
use crate::formats::StageFormat;

/// O que o torneio define antes de ter inscritos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TournamentSetup<'a> {
    /// Jogadores por lado (1 no duelo).
    pub team_size: u32,
    /// Vagas. O pior caso de cada fase é calculado com esse número, que é o
    /// teto de participantes de qualquer fase.
    pub max_participants: usize,
    pub champions: ChampionRules,
    /// Campeões disponíveis no torneio.
    pub champion_pool: u32,
    /// Fases, na ordem.
    pub stages: &'a [StageFormat],
}

impl TournamentSetup<'_> {
    /// Erros: `NoStages`; os de cada fase (`StageFormat::validate` e
    /// `StageFormat::worst_case_games` com `max_participants`);
    /// `ChampionPoolExhausted` quando o pool não aguenta os bans e a regra de
    /// não repetição no pior caso.
    pub fn validate(&self) -> Result<(), EngineError> {
        if self.stages.is_empty() {
            return Err(EngineError::NoStages);
        }
        let mut worst = Vec::with_capacity(self.stages.len());
        for stage in self.stages {
            stage.validate()?;
            // Sempre calculado, para as vagas serem validadas.
            worst.push(stage.worst_case_games(self.max_participants)?);
        }
        let Some(rule) = self.champions.no_repeat else {
            let required = champions::required_without_rule(self.champions.bans_per_game);
            return if required > self.champion_pool {
                Err(EngineError::ChampionPoolExhausted {
                    required,
                    available: self.champion_pool,
                })
            } else {
                Ok(())
            };
        };
        let games_in_scope = match rule.scope {
            NoRepeatScope::Match => self
                .stages
                .iter()
                .map(|s| u32::from(s.longest_series()))
                .max(),
            NoRepeatScope::Stage => worst.iter().copied().max(),
            NoRepeatScope::Tournament => Some(worst.iter().sum()),
        }
        .unwrap_or(0);
        champions::check_budget(
            rule,
            ChampionBudget {
                pool_size: self.champion_pool,
                bans_per_game: self.champions.bans_per_game,
                team_size: self.team_size,
                games_in_scope,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::champions::{NoRepeatRule, NoRepeatWho};
    use crate::formats::elimination::{EliminationConfig, EliminationKind};
    use crate::series::SeriesLength;

    fn double(default: u8, finals: u8) -> StageFormat {
        StageFormat::Elimination(EliminationConfig {
            kind: EliminationKind::Double {
                grand_final_reset: true,
            },
            series: SeriesLength { default, finals },
        })
    }

    fn single(default: u8, finals: u8) -> StageFormat {
        StageFormat::Elimination(EliminationConfig {
            kind: EliminationKind::Single,
            series: SeriesLength { default, finals },
        })
    }

    fn rules(bans: u32, scope: Option<NoRepeatScope>) -> ChampionRules {
        ChampionRules {
            bans_per_game: bans,
            no_repeat: scope.map(|scope| NoRepeatRule {
                who: NoRepeatWho::Player,
                scope,
            }),
        }
    }

    fn setup(
        n: usize,
        champions: ChampionRules,
        pool: u32,
        stages: &[StageFormat],
    ) -> Result<(), EngineError> {
        TournamentSetup {
            team_size: 1,
            max_participants: n,
            champions,
            champion_pool: pool,
            stages,
        }
        .validate()
    }

    fn exhausted(required: u32, available: u32) -> Result<(), EngineError> {
        Err(EngineError::ChampionPoolExhausted {
            required,
            available,
        })
    }

    #[test]
    fn no_stages() {
        assert_eq!(
            setup(8, rules(0, None), 17, &[]),
            Err(EngineError::NoStages)
        );
    }

    #[test]
    fn invalid_best_of() {
        assert_eq!(
            setup(8, rules(0, None), 17, &[double(4, 5)]),
            Err(EngineError::InvalidBestOf(4))
        );
    }

    #[test]
    fn slots_are_validated() {
        let s = [double(3, 5)];
        assert_eq!(
            setup(1, rules(0, None), 17, &s),
            Err(EngineError::NotEnoughParticipants { min: 2, got: 1 })
        );
        assert_eq!(
            setup(1025, rules(0, None), 17, &s),
            Err(EngineError::TooManyParticipants {
                max: 1024,
                got: 1025
            })
        );
    }

    #[test]
    fn without_rule_each_map_needs_both_bans_plus_one() {
        let s = [double(3, 5)];
        assert_eq!(setup(8, rules(3, None), 6, &s), exhausted(7, 6));
        assert_eq!(setup(8, rules(1, None), 17, &s), Ok(()));
    }

    #[test]
    fn tournament_scope_uses_worst_case_games() {
        let s = [double(3, 5)];
        let r = rules(1, Some(NoRepeatScope::Tournament));
        assert_eq!(setup(16, r, 17, &s), exhausted(33, 17));
    }

    #[test]
    fn match_scope_uses_longest_series() {
        let s = [double(3, 5)];
        let r = rules(1, Some(NoRepeatScope::Match));
        assert_eq!(setup(16, r, 17, &s), Ok(()));
        assert_eq!(setup(16, r, 5, &s), exhausted(7, 5));
    }

    #[test]
    fn stage_scope_takes_max_and_tournament_scope_sums() {
        let a = single(1, 1);
        let b = double(3, 5);
        let s = [a, b];
        let wa = a.worst_case_games(16).unwrap();
        let wb = b.worst_case_games(16).unwrap();
        assert!(wa > 0 && wb > wa);
        let stage = rules(0, Some(NoRepeatScope::Stage));
        let tournament = rules(0, Some(NoRepeatScope::Tournament));
        assert_eq!(setup(16, stage, wb, &s), Ok(()));
        assert_eq!(setup(16, stage, wb - 1, &s), exhausted(wb, wb - 1));
        assert_eq!(setup(16, tournament, wa + wb, &s), Ok(()));
        assert_eq!(setup(16, tournament, wb, &s), exhausted(wa + wb, wb));
    }
}
