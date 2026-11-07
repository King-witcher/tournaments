//! Validação da configuração de um torneio antes de gravar: fases, vagas e
//! regras de campeões.
//!
//! O pior caso de cada fase usa os lados que as vagas comportam, um teto
//! conservador: as fases seguintes têm no máximo esse número de participantes
//! (ou o corte delas, se for menor). As fases são isoladas na regra de não
//! repetição: nos escopos fase e torneio, o orçamento é o da fase de maior
//! pior caso, não a soma.

use crate::champions::{self, ChampionBudget, ChampionRules, NoRepeatScope};
use crate::core::EngineError;
use crate::stages::{MIN_ADVANCE, StageSetup};

/// Times na chave, no mínimo.
const MIN_SIDES: usize = 2;
/// Jogadores por lado, no máximo (4v4).
pub const MAX_TEAM_SIZE: u32 = 4;

/// O que o torneio define antes de ter inscritos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TournamentSetup<'a> {
    /// Jogadores por lado: 1 no duelo, até `MAX_TEAM_SIZE`.
    pub team_size: u32,
    /// Vagas, em jogadores (no duelo, os lados). Os lados da chave no pior
    /// caso são os times que elas comportam, `⌊vagas/tamanho⌋`: o pior caso
    /// de cada fase é calculado com esse número (ou com o corte dela, se for
    /// menor), que é o teto de participantes de qualquer fase.
    pub slots: usize,
    pub champions: ChampionRules,
    /// Campeões disponíveis no torneio.
    pub champion_pool: u32,
    /// Fases, na ordem.
    pub stages: &'a [StageSetup],
}

impl TournamentSetup<'_> {
    /// Erros: `InvalidTeamSize` fora de 1 a `MAX_TEAM_SIZE`; `TooManyBans`
    /// acima de `champions::MAX_BANS_PER_SERIES`; `NoStages`;
    /// `NotEnoughPlayers` nos times, quando as vagas não formam dois times;
    /// `EntryRulesFirst` com corte ou check-in na fase 1; `InvalidAdvance` com
    /// o corte fora de `MIN_ADVANCE` ao teto do formato; `EliminationNotLast`;
    /// os de cada fase dentro de `InStage` (`StageFormat::validate` e
    /// `StageFormat::worst_case_games` com os lados);
    /// `ChampionPoolExhausted` quando o pool não aguenta os bans da série,
    /// as regras de repetição e, nos times, a escolha de cada jogador no pior
    /// caso (`champions::required_pool`).
    pub fn validate(&self) -> Result<(), EngineError> {
        if !(1..=MAX_TEAM_SIZE).contains(&self.team_size) {
            return Err(EngineError::InvalidTeamSize {
                max: MAX_TEAM_SIZE,
                got: self.team_size,
            });
        }
        if self.champions.bans_per_series > champions::MAX_BANS_PER_SERIES {
            return Err(EngineError::TooManyBans {
                max: champions::MAX_BANS_PER_SERIES,
                got: self.champions.bans_per_series,
            });
        }
        if self.stages.is_empty() {
            return Err(EngineError::NoStages);
        }
        let size = usize::try_from(self.team_size).unwrap_or(usize::MAX);
        let sides = self.slots / size;
        if size > 1 && sides < MIN_SIDES {
            return Err(EngineError::NotEnoughPlayers {
                min: MIN_SIDES * size,
                got: self.slots,
            });
        }
        let mut worst = Vec::with_capacity(self.stages.len());
        let mut participants = sides;
        for (i, stage) in self.stages.iter().enumerate() {
            let number = i + 1;
            participants = entry_rules(stage, number, self.stages.len(), participants)?;
            stage.format.validate().map_err(|e| e.in_stage(number))?;
            // Sempre calculado, para as vagas serem validadas.
            let games = stage
                .format
                .worst_case_games(participants)
                .map_err(|e| e.in_stage(number))?;
            worst.push(games);
        }
        self.budget(&worst)
    }

    /// Ao iniciar a fase de índice `index` (a partir de 0) com `sides` lados
    /// de verdade: o formato com esse número e o orçamento só dessa fase
    /// (as fases são isoladas na regra de não repetição). Erros: os de
    /// `validate` para a fase.
    pub fn validate_stage(&self, index: usize, sides: usize) -> Result<(), EngineError> {
        let stage = self.stages.get(index).ok_or(EngineError::NoStages)?;
        let number = index + 1;
        stage.format.validate().map_err(|e| e.in_stage(number))?;
        let games = stage
            .format
            .worst_case_games(sides)
            .map_err(|e| e.in_stage(number))?;
        self.budget(&[games])
    }

    /// O orçamento de campeões com o pior caso de cada fase (`worst`).
    fn budget(&self, worst: &[u32]) -> Result<(), EngineError> {
        let rule = self.champions.no_repeat;
        let games_in_scope = match rule.map(|r| r.scope) {
            None => 1,
            Some(NoRepeatScope::Series) => self
                .stages
                .iter()
                .map(|s| u32::from(s.format.longest_series()))
                .max()
                .unwrap_or(0),
            // Cada fase recomeça: com uma fase só, os dois são o mesmo.
            Some(NoRepeatScope::Stage | NoRepeatScope::Tournament) => {
                worst.iter().copied().max().unwrap_or(0)
            }
        };
        champions::check_budget(
            rule,
            ChampionBudget {
                pool_size: self.champion_pool,
                bans_per_series: self.champions.bans_per_series,
                team_size: self.team_size,
                mirror: self.champions.mirror,
                same_team: self.champions.same_team,
                games_in_scope,
            },
        )
    }
}

/// Confere o corte e o check-in da fase `number` (de `total`) e a
/// eliminação só na última, e devolve quantos participantes ela tem no
/// máximo, saindo de `previous` (os da anterior, ou os lados das vagas).
fn entry_rules(
    stage: &StageSetup,
    number: usize,
    total: usize,
    previous: usize,
) -> Result<usize, EngineError> {
    let at = u16::try_from(number).unwrap_or(u16::MAX);
    if number == 1 && (stage.advance.is_some() || stage.check_in) {
        return Err(EngineError::EntryRulesFirst { stage: at });
    }
    if matches!(stage.format, crate::formats::StageFormat::Elimination(_)) && number < total {
        return Err(EngineError::EliminationNotLast { stage: at });
    }
    let Some(advance) = stage.advance else {
        return Ok(previous);
    };
    let max = u16::try_from(stage.format.max_participants()).unwrap_or(u16::MAX);
    if !(MIN_ADVANCE..=max).contains(&advance) {
        return Err(EngineError::InvalidAdvance {
            stage: at,
            min: MIN_ADVANCE,
            max,
        });
    }
    Ok(previous.min(usize::from(advance)))
}

#[cfg(test)]
mod tests;
