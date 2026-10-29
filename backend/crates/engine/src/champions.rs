//! Regras de não repetição de campeões e o orçamento do pool.

use serde::{Deserialize, Serialize};

use crate::core::EngineError;

/// Quem não pode repetir campeão.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatWho {
    /// Cada jogador não repete os próprios campeões.
    Player,
    /// O time inteiro não repete: um campeão usado por um membro fica
    /// indisponível para todos nos mapas seguintes.
    Team,
}

/// Até onde a regra vale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatScope {
    /// Entre os mapas de uma mesma partida.
    Match,
    /// Em todas as partidas de uma fase.
    Stage,
    /// No torneio inteiro.
    Tournament,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoRepeatRule {
    pub who: NoRepeatWho,
    pub scope: NoRepeatScope,
}

/// Regras de campeões de um torneio (guardadas em JSON no torneio).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChampionRules {
    /// Bans de cada lado por mapa. O ban tira o campeão dos dois lados
    /// naquele mapa (veto): cada mapa perde `2 * bans_per_game` campeões.
    pub bans_per_game: u32,
    /// `None`: sem regra de não repetição.
    pub no_repeat: Option<NoRepeatRule>,
    /// Nos times (TDM): dois jogadores do mesmo time podem escolher o mesmo
    /// campeão no mesmo mapa. Padrão não (ausente no JSON gravado antes do
    /// TDM: falso). Entre times o espelho vale sempre.
    #[serde(default)]
    pub team_duplicates: bool,
}

/// Dados para checar se o pool de campeões aguenta a regra no pior caso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChampionBudget {
    /// Campeões disponíveis no torneio.
    pub pool_size: u32,
    /// Bans de cada lado por mapa (valem para os dois lados).
    pub bans_per_game: u32,
    /// Jogadores por lado (1 no duelo).
    pub team_size: u32,
    /// Dois do mesmo time podem escolher o mesmo campeão no mesmo mapa.
    pub team_duplicates: bool,
    /// Máximo de mapas que um participante pode jogar dentro do escopo da
    /// regra: o melhor de N mais longo para `Match`; o pior caso do formato
    /// (`worst_case_games`) para `Stage` e `Tournament`.
    pub games_in_scope: u32,
}

impl ChampionBudget {
    /// Campeões diferentes que um time escolhe num mapa: um por jogador, ou
    /// um só se o time pode repetir no mapa (no duelo, 1).
    const fn per_map(&self) -> u32 {
        if self.team_duplicates || self.team_size == 0 {
            1
        } else {
            self.team_size
        }
    }
}

/// Campeões necessários no pior caso, com a regra.
///
/// No último mapa do escopo, o último a escolher enfrenta os bans daquele
/// mapa (os dois lados: o ban vale para os dois), o que a regra já tirou dele
/// nos mapas anteriores (um por mapa, o dele; ou o time todo, um por jogador
/// por mapa, com a regra do time) e, sem repetir no time, o que os colegas
/// escolheram no mapa; precisa sobrar um:
/// `2·bans + (jogos − 1)·(1 ou tamanho) + (tamanho ou 1)`. No duelo dá
/// `jogos + 2·bans`.
pub fn required_pool(rule: NoRepeatRule, budget: ChampionBudget) -> u32 {
    let before = match rule.who {
        NoRepeatWho::Player => 1,
        NoRepeatWho::Team => budget.team_size.max(1),
    };
    2 * budget.bans_per_game + budget.games_in_scope.saturating_sub(1) * before + budget.per_map()
}

/// Campeões necessários sem a regra de não repetição: os bans dos dois lados
/// e um para cada escolha que não pode repetir no mapa (no duelo, um).
pub fn required_without_rule(budget: ChampionBudget) -> u32 {
    2 * budget.bans_per_game + budget.per_map()
}

pub fn check_budget(rule: Option<NoRepeatRule>, budget: ChampionBudget) -> Result<(), EngineError> {
    let required = match rule {
        Some(rule) => required_pool(rule, budget),
        None => required_without_rule(budget),
    };
    if required <= budget.pool_size {
        Ok(())
    } else {
        Err(EngineError::ChampionPoolExhausted {
            required,
            available: budget.pool_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOURNAMENT_PLAYER: NoRepeatRule = NoRepeatRule {
        who: NoRepeatWho::Player,
        scope: NoRepeatScope::Tournament,
    };

    const TOURNAMENT_TEAM: NoRepeatRule = NoRepeatRule {
        who: NoRepeatWho::Team,
        scope: NoRepeatScope::Tournament,
    };

    fn budget(pool_size: u32, bans_per_game: u32, team_size: u32, games: u32) -> ChampionBudget {
        ChampionBudget {
            pool_size,
            bans_per_game,
            team_size,
            team_duplicates: false,
            games_in_scope: games,
        }
    }

    #[test]
    fn duel_without_bans_needs_one_champion_per_game() {
        assert_eq!(required_pool(TOURNAMENT_PLAYER, budget(17, 0, 1, 9)), 9);
        assert!(check_budget(Some(TOURNAMENT_PLAYER), budget(9, 0, 1, 9)).is_ok());
    }

    #[test]
    fn bans_add_to_the_requirement() {
        assert_eq!(required_pool(TOURNAMENT_PLAYER, budget(17, 2, 1, 9)), 13);
    }

    #[test]
    fn team_rule_multiplies_by_team_size() {
        assert_eq!(required_pool(TOURNAMENT_TEAM, budget(17, 1, 2, 6)), 14);
        // Podendo repetir no mapa, o último mapa pede um só.
        let dup = ChampionBudget {
            team_duplicates: true,
            ..budget(17, 1, 2, 6)
        };
        assert_eq!(required_pool(TOURNAMENT_TEAM, dup), 13);
    }

    #[test]
    fn team_rule_with_duplicates_in_md5() {
        // 3v3, md5, 2 bans: cada mapa anterior gasta 3 do time (um por
        // jogador, mesmo podendo repetir no mapa: no pior caso, os três são
        // distintos); no último, quem escolhe por último pode repetir o de um
        // colega e só precisa de 1.
        let dup = ChampionBudget {
            team_duplicates: true,
            ..budget(17, 2, 3, 5)
        };
        assert_eq!(required_pool(TOURNAMENT_TEAM, dup), 4 + 4 * 3 + 1);
        assert!(check_budget(Some(TOURNAMENT_TEAM), dup).is_ok());
        let short = ChampionBudget {
            pool_size: 16,
            ..dup
        };
        assert_eq!(
            check_budget(Some(TOURNAMENT_TEAM), short),
            Err(EngineError::ChampionPoolExhausted {
                required: 17,
                available: 16
            })
        );
    }

    #[test]
    fn player_rule_in_teams_counts_the_teammates_of_the_map() {
        // 3v3, md3, 1 ban: 2 bans + 2 mapas dele + 3 no mapa.
        assert_eq!(required_pool(TOURNAMENT_PLAYER, budget(17, 1, 3, 3)), 7);
        let dup = ChampionBudget {
            team_duplicates: true,
            ..budget(17, 1, 3, 3)
        };
        assert_eq!(required_pool(TOURNAMENT_PLAYER, dup), 5);
    }

    #[test]
    fn without_rule_a_team_needs_one_per_player() {
        assert_eq!(required_without_rule(budget(17, 3, 1, 1)), 7);
        assert_eq!(required_without_rule(budget(17, 3, 4, 1)), 10);
        let dup = ChampionBudget {
            team_duplicates: true,
            ..budget(17, 3, 4, 1)
        };
        assert_eq!(required_without_rule(dup), 7);
    }

    #[test]
    fn reports_required_and_available_when_short() {
        assert_eq!(
            check_budget(Some(TOURNAMENT_PLAYER), budget(17, 0, 1, 24)),
            Err(EngineError::ChampionPoolExhausted {
                required: 24,
                available: 17
            })
        );
    }
}
