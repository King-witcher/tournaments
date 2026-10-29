//! Validação da configuração de um torneio antes de gravar: fases, vagas e
//! regras de campeões.
//!
//! O pior caso de cada fase usa os lados que as vagas comportam, um teto
//! conservador: as fases seguintes têm no máximo esse número de participantes.

use crate::champions::{self, ChampionBudget, ChampionRules, NoRepeatScope};
use crate::core::EngineError;
use crate::formats::StageFormat;

/// Times na chave, no mínimo.
const MIN_SIDES: usize = 2;

/// O que o torneio define antes de ter inscritos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TournamentSetup<'a> {
    /// Jogadores por lado (1 no duelo; 0 conta como 1).
    pub team_size: u32,
    /// Vagas, em jogadores (no duelo, os lados). Os lados da chave no pior
    /// caso são os times que elas comportam, `⌊vagas/tamanho⌋`: o pior caso
    /// de cada fase é calculado com esse número, que é o teto de
    /// participantes de qualquer fase.
    pub slots: usize,
    pub champions: ChampionRules,
    /// Campeões disponíveis no torneio.
    pub champion_pool: u32,
    /// Fases, na ordem.
    pub stages: &'a [StageFormat],
}

impl TournamentSetup<'_> {
    /// Erros: `NoStages`; `NotEnoughPlayers` nos times, quando as vagas não
    /// formam dois times; os de cada fase (`StageFormat::validate` e
    /// `StageFormat::worst_case_games` com os lados);
    /// `ChampionPoolExhausted` quando o pool não aguenta os bans, a regra de
    /// não repetição e, nos times, a escolha de cada jogador no pior caso
    /// (`champions::required_pool`).
    pub fn validate(&self) -> Result<(), EngineError> {
        if self.stages.is_empty() {
            return Err(EngineError::NoStages);
        }
        let size = usize::try_from(self.team_size.max(1)).unwrap_or(usize::MAX);
        let sides = self.slots / size;
        if size > 1 && sides < MIN_SIDES {
            return Err(EngineError::NotEnoughPlayers {
                min: MIN_SIDES * size,
                got: self.slots,
            });
        }
        let mut worst = Vec::with_capacity(self.stages.len());
        for stage in self.stages {
            stage.validate()?;
            // Sempre calculado, para as vagas serem validadas.
            worst.push(stage.worst_case_games(sides)?);
        }
        let rule = self.champions.no_repeat;
        let games_in_scope = match rule.map(|r| r.scope) {
            None => 1,
            Some(NoRepeatScope::Match) => self
                .stages
                .iter()
                .map(|s| u32::from(s.longest_series()))
                .max()
                .unwrap_or(0),
            Some(NoRepeatScope::Stage) => worst.iter().copied().max().unwrap_or(0),
            Some(NoRepeatScope::Tournament) => worst.iter().sum(),
        };
        champions::check_budget(
            rule,
            ChampionBudget {
                pool_size: self.champion_pool,
                bans_per_game: self.champions.bans_per_game,
                team_size: self.team_size,
                team_duplicates: self.champions.team_duplicates,
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
            team_duplicates: false,
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
            slots: n,
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
    fn teams_need_a_champion_per_player() {
        // 4v4, 3 bans, sem regra: 6 bans e 4 escolhas no mapa.
        let s = [double(3, 5)];
        let tdm = |champions, pool| {
            TournamentSetup {
                team_size: 4,
                slots: 16,
                champions,
                champion_pool: pool,
                stages: &s,
            }
            .validate()
        };
        assert_eq!(tdm(rules(3, None), 9), exhausted(10, 9));
        let dup = ChampionRules {
            team_duplicates: true,
            ..rules(3, None)
        };
        assert_eq!(tdm(dup, 9), Ok(()));
        // Na série (md5): 6 bans, 4 mapas dele e 4 no último.
        assert_eq!(tdm(rules(3, Some(NoRepeatScope::Match)), 17), Ok(()));
        assert_eq!(
            tdm(rules(3, Some(NoRepeatScope::Match)), 13),
            exhausted(14, 13)
        );
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

    #[test]
    fn table_formats_use_their_worst_case_and_limits() {
        use crate::formats::{round_robin::RoundRobinConfig, swiss::SwissConfig};
        let rr = [StageFormat::RoundRobin(RoundRobinConfig { best_of: 3 })];
        let swiss = [StageFormat::Swiss(SwissConfig {
            best_of: 3,
            rounds: None,
        })];
        let r = rules(0, Some(NoRepeatScope::Tournament));
        // Round robin com 8: 7 séries md3; suíço com 16: 4 rodadas md3.
        assert_eq!(setup(8, r, 21, &rr), Ok(()));
        assert_eq!(setup(8, r, 17, &rr), exhausted(21, 17));
        assert_eq!(setup(16, r, 12, &swiss), Ok(()));
        assert_eq!(
            setup(33, rules(0, None), 17, &rr),
            Err(EngineError::TooManyParticipants { max: 32, got: 33 })
        );
    }

    fn teams(
        size: u32,
        slots: usize,
        champions: ChampionRules,
        pool: u32,
        stages: &[StageFormat],
    ) -> Result<(), EngineError> {
        TournamentSetup {
            team_size: size,
            slots,
            champions,
            champion_pool: pool,
            stages,
        }
        .validate()
    }

    #[test]
    fn slots_below_two_teams_are_counted_in_players() {
        let s = [double(3, 5)];
        // 3 vagas com time de 4: nem um time; 7, um só. O mínimo vem em
        // jogadores, não em lados.
        for slots in [3, 7] {
            assert_eq!(
                teams(4, slots, rules(0, None), 17, &s),
                Err(EngineError::NotEnoughPlayers { min: 8, got: slots })
            );
        }
        assert_eq!(teams(4, 8, rules(0, None), 17, &s), Ok(()));
        // No duelo, continua o erro do formato.
        assert_eq!(
            teams(1, 1, rules(0, None), 17, &s),
            Err(EngineError::NotEnoughParticipants { min: 2, got: 1 })
        );
    }

    #[test]
    fn team_rule_with_duplicates_in_md5_by_stage_and_by_tournament() {
        // 3v3, 1 ban, regra do time podendo repetir no mapa, séries md5, 4
        // times: a cada mapa anterior o time gasta 3; no último, basta 1.
        let a = single(5, 5);
        let b = double(5, 5);
        let s = [a, b];
        let wa = a.worst_case_games(4).unwrap();
        let wb = b.worst_case_games(4).unwrap();
        assert!(wb > wa);
        let rule = |scope| ChampionRules {
            bans_per_game: 1,
            no_repeat: Some(NoRepeatRule {
                who: NoRepeatWho::Team,
                scope,
            }),
            team_duplicates: true,
        };
        let required = |games: u32| 2 + (games - 1) * 3 + 1;
        for (scope, games) in [
            (NoRepeatScope::Stage, wb),
            (NoRepeatScope::Tournament, wa + wb),
        ] {
            let need = required(games);
            assert_eq!(teams(3, 12, rule(scope), need, &s), Ok(()));
            assert_eq!(
                teams(3, 12, rule(scope), need - 1, &s),
                exhausted(need, need - 1)
            );
        }
    }

    #[test]
    fn sixteen_teams() {
        // 4v4 com 64 vagas, ou 67 (os 3 a mais não formam time): 16 times,
        // o mesmo pior caso de 16 no duelo. Regra do jogador no torneio, 1
        // ban: 2 bans, um por mapa anterior e 4 no último.
        let s = [double(3, 5)];
        let w = s[0].worst_case_games(16).unwrap();
        let r = rules(1, Some(NoRepeatScope::Tournament));
        let need = 2 + (w - 1) + 4;
        for slots in [64, 67] {
            assert_eq!(teams(4, slots, r, need, &s), Ok(()));
            assert_eq!(teams(4, slots, r, need - 1, &s), exhausted(need, need - 1));
        }
        // 68 vagas: 17 times, a chave dobra.
        assert_ne!(teams(4, 68, r, need, &s), Ok(()));
    }
}
