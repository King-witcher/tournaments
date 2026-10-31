//! Regras de campeões de um torneio (bans da série, repetição na partida e
//! entre partidas) e o orçamento do pool.
//!
//! O draft é da série inteira, antes da partida 1 (`veto`): os bans valem
//! para os dois lados em todas as partidas dela, e cada um escolhe o campeão
//! de cada partida. Três regras dizem o que pode repetir:
//! - **na partida, entre adversários** (`mirror`): os dois lados com o mesmo
//!   campeão. Sem ela, fica com o campeão quem escolhe primeiro.
//! - **na partida, no time** (`same_team`, só nos times): dois colegas com o
//!   mesmo campeão.
//! - **entre partidas** (`no_repeat`): quem (`who`) não repete o campeão de
//!   outra partida, e até onde (`scope`). Pick de partida que não chega a
//!   ser jogada não conta fora da série.

use serde::{Deserialize, Serialize};

use crate::core::EngineError;

/// Bans de campeão de cada lado na série, no máximo.
pub const MAX_BANS_PER_SERIES: u32 = 3;

/// Quem não pode repetir campeão entre partidas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatWho {
    /// Cada jogador não repete os próprios campeões.
    Player,
    /// O time inteiro não repete: um campeão que um membro escolheu para uma
    /// partida fica indisponível para todos nas outras.
    Team,
}

/// Até onde a regra vale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoRepeatScope {
    /// Entre as partidas de uma mesma série.
    Series,
    /// Em todas as séries de uma fase.
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
    /// Bans de cada lado na série. O ban tira o campeão dos dois lados em
    /// todas as partidas dela: a série perde `2 * bans_per_series`.
    pub bans_per_series: u32,
    /// Adversários podem ter o mesmo campeão na mesma partida.
    pub mirror: bool,
    /// Nos times (TDM): colegas podem ter o mesmo campeão na mesma partida.
    /// No duelo não se aplica.
    pub same_team: bool,
    /// `None`: pode repetir entre partidas.
    pub no_repeat: Option<NoRepeatRule>,
}

/// Dados para checar se o pool de campeões aguenta as regras no pior caso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChampionBudget {
    /// Campeões disponíveis no torneio.
    pub pool_size: u32,
    /// Bans de cada lado na série (valem para os dois lados).
    pub bans_per_series: u32,
    /// Jogadores por lado (1 no duelo).
    pub team_size: u32,
    pub mirror: bool,
    pub same_team: bool,
    /// Máximo de partidas que um participante pode jogar dentro do escopo da
    /// regra: a série mais longa para `Series`; o pior caso do formato
    /// (`worst_case_games`) para `Stage` e `Tournament`. Sem a regra, não
    /// conta.
    pub games_in_scope: u32,
}

/// Campeões necessários no pior caso, com as regras.
///
/// O último a escolher numa partida enfrenta os bans da série (os dois
/// lados), o que a regra de não repetição já tirou dele nas outras partidas
/// do escopo (um por partida, o dele; ou o time todo, um por jogador por
/// partida, com a regra do time), sem repetir no time o que os colegas
/// escolheram na partida e, sem espelho, o que os adversários escolheram
/// nela; precisa sobrar um:
/// `2·bans + antes + colegas + adversários + 1`, com `antes` = 0 sem regra,
/// `partidas − 1` pela do jogador e `(partidas − 1)·tamanho` pela do time;
/// `colegas` = `tamanho − 1` sem repetir no time; `adversários` = `tamanho`
/// sem espelho. No duelo com a regra, `2·bans + partidas` (+ 1 sem espelho).
pub fn required_pool(rule: Option<NoRepeatRule>, budget: ChampionBudget) -> u32 {
    let size = budget.team_size.max(1);
    let earlier = budget.games_in_scope.saturating_sub(1);
    let before = match rule.map(|r| r.who) {
        None => 0,
        Some(NoRepeatWho::Player) => earlier,
        Some(NoRepeatWho::Team) => earlier.saturating_mul(size),
    };
    let mates = if budget.same_team { 0 } else { size - 1 };
    let opponents = if budget.mirror { 0 } else { size };
    budget
        .bans_per_series
        .saturating_mul(2)
        .saturating_add(before)
        .saturating_add(mates)
        .saturating_add(opponents)
        .saturating_add(1)
}

/// Erro: `ChampionPoolExhausted` quando o pool não aguenta `required_pool`.
pub fn check_budget(rule: Option<NoRepeatRule>, budget: ChampionBudget) -> Result<(), EngineError> {
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

    const fn rule(who: NoRepeatWho, scope: NoRepeatScope) -> Option<NoRepeatRule> {
        Some(NoRepeatRule { who, scope })
    }

    const PLAYER: Option<NoRepeatRule> = rule(NoRepeatWho::Player, NoRepeatScope::Tournament);
    const TEAM: Option<NoRepeatRule> = rule(NoRepeatWho::Team, NoRepeatScope::Tournament);

    fn budget(bans: u32, team_size: u32, games: u32) -> ChampionBudget {
        ChampionBudget {
            pool_size: 17,
            bans_per_series: bans,
            team_size,
            mirror: true,
            same_team: false,
            games_in_scope: games,
        }
    }

    #[test]
    fn duel_with_the_rule_needs_one_champion_per_game() {
        assert_eq!(required_pool(PLAYER, budget(0, 1, 9)), 9);
        assert_eq!(required_pool(TEAM, budget(0, 1, 9)), 9);
        assert!(
            check_budget(
                PLAYER,
                ChampionBudget {
                    pool_size: 9,
                    ..budget(0, 1, 9)
                }
            )
            .is_ok()
        );
    }

    #[test]
    fn bans_of_both_sides_add_to_the_requirement() {
        assert_eq!(required_pool(PLAYER, budget(2, 1, 9)), 13);
        assert_eq!(required_pool(None, budget(3, 1, 9)), 7);
    }

    #[test]
    fn without_mirror_the_opponents_of_the_game_count() {
        let no_mirror = |b: ChampionBudget| ChampionBudget { mirror: false, ..b };
        // Duelo md3 na série, 1 ban: 2 bans, os 2 dele e o do adversário.
        assert_eq!(required_pool(PLAYER, budget(1, 1, 3)), 5);
        assert_eq!(required_pool(PLAYER, no_mirror(budget(1, 1, 3))), 6);
        // 3v3 sem regra: 2 bans, 2 colegas, 3 adversários e o dele.
        assert_eq!(required_pool(None, no_mirror(budget(1, 3, 1))), 8);
    }

    #[test]
    fn the_team_rule_multiplies_by_team_size() {
        assert_eq!(required_pool(TEAM, budget(1, 2, 6)), 2 + 5 * 2 + 1 + 1);
        // Podendo repetir no time, os colegas da partida não contam.
        let same = ChampionBudget {
            same_team: true,
            ..budget(1, 2, 6)
        };
        assert_eq!(required_pool(TEAM, same), 2 + 5 * 2 + 1);
    }

    #[test]
    fn the_player_rule_in_teams_counts_the_teammates_of_the_game() {
        // 3v3, md3, 1 ban: 2 bans + 2 partidas dele + 2 colegas + o dele.
        assert_eq!(required_pool(PLAYER, budget(1, 3, 3)), 7);
        let same = ChampionBudget {
            same_team: true,
            ..budget(1, 3, 3)
        };
        assert_eq!(required_pool(PLAYER, same), 5);
    }

    #[test]
    fn the_three_scenarios_of_the_user() {
        // Todos contra todos com personagens diferentes: cada jogador não
        // repete no torneio. 2v2, 7 séries md1, sem bans: as 6 dele, o
        // colega e o dele.
        let round_robin = budget(0, 2, 7);
        assert_eq!(required_pool(PLAYER, round_robin), 6 + 1 + 1);
        // Pode repetir entre times mas não no time: o padrão.
        assert_eq!(required_pool(None, budget(1, 2, 1)), 2 + 1 + 1);
        // Ninguém repete na partida: os 4 da partida e os bans.
        let nobody = ChampionBudget {
            mirror: false,
            ..budget(1, 2, 1)
        };
        assert_eq!(required_pool(None, nobody), 2 + 4);
    }

    #[test]
    fn reports_required_and_available_when_short() {
        assert_eq!(
            check_budget(PLAYER, budget(0, 1, 24)),
            Err(EngineError::ChampionPoolExhausted {
                required: 24,
                available: 17
            })
        );
    }
}
