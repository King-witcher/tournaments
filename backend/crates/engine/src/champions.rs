//! Regras de não repetição de campeões.

use serde::{Deserialize, Serialize};

use crate::core::EngineError;

/// Quem não pode repetir campeão.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatWho {
    /// Cada jogador não repete os próprios campeões.
    Player,
    /// O time inteiro não repete: um campeão usado por um membro fica
    /// indisponível para todos.
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

/// Dados para checar se o pool de campeões aguenta a regra no pior caso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChampionBudget {
    /// Campeões disponíveis no torneio.
    pub pool_size: u32,
    /// Bans que o adversário aplica contra um lado, por mapa.
    pub bans_per_game: u32,
    /// Jogadores por lado (1 no duelo).
    pub team_size: u32,
    /// Máximo de mapas que um participante pode jogar dentro do escopo da
    /// regra: o melhor de N mais longo para `Match`; o pior caso do formato
    /// (`worst_case_games`) para `Stage` e `Tournament`.
    pub games_in_scope: u32,
}

/// Campeões necessários no pior caso.
///
/// No último mapa do escopo, quem escolhe já gastou um campeão por mapa
/// anterior (por jogador ou pelo time todo) e ainda enfrenta os bans daquele
/// mapa; precisa sobrar ao menos um campeão para cada escolha.
pub fn required_pool(rule: NoRepeatRule, budget: ChampionBudget) -> u32 {
    let picks_per_game = match rule.who {
        NoRepeatWho::Player => 1,
        NoRepeatWho::Team => budget.team_size,
    };
    budget.games_in_scope * picks_per_game + budget.bans_per_game
}

pub fn check_budget(rule: NoRepeatRule, budget: ChampionBudget) -> Result<(), EngineError> {
    let required = required_pool(rule, budget);
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

    fn budget(pool_size: u32, bans_per_game: u32, team_size: u32, games: u32) -> ChampionBudget {
        ChampionBudget {
            pool_size,
            bans_per_game,
            team_size,
            games_in_scope: games,
        }
    }

    #[test]
    fn duel_without_bans_needs_one_champion_per_game() {
        assert_eq!(required_pool(TOURNAMENT_PLAYER, budget(17, 0, 1, 9)), 9);
        assert!(check_budget(TOURNAMENT_PLAYER, budget(9, 0, 1, 9)).is_ok());
    }

    #[test]
    fn bans_add_to_the_requirement() {
        assert_eq!(required_pool(TOURNAMENT_PLAYER, budget(17, 2, 1, 9)), 11);
    }

    #[test]
    fn team_rule_multiplies_by_team_size() {
        let rule = NoRepeatRule {
            who: NoRepeatWho::Team,
            scope: NoRepeatScope::Tournament,
        };
        assert_eq!(required_pool(rule, budget(17, 1, 2, 6)), 13);
    }

    #[test]
    fn reports_required_and_available_when_short() {
        assert_eq!(
            check_budget(TOURNAMENT_PLAYER, budget(17, 0, 1, 24)),
            Err(EngineError::ChampionPoolExhausted {
                required: 24,
                available: 17
            })
        );
    }
}
