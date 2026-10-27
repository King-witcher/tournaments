//! Formatos de chaveamento.

pub mod elimination;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::{EngineError, ParticipantId};
use elimination::{EliminationBracket, EliminationConfig, MAX_PARTICIPANTS};

/// Formato de uma fase do torneio: é o que a fase guarda (em JSON). O estado
/// da fase nunca é guardado; vem do `replay` do formato.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum StageFormat {
    Elimination(EliminationConfig),
}

impl StageFormat {
    /// Confere a configuração sem depender de participantes (ex.: melhor de N
    /// ímpar).
    pub fn validate(&self) -> Result<(), EngineError> {
        match self {
            Self::Elimination(c) => c.series.validate(),
        }
    }

    /// Maior melhor de N de uma partida da fase.
    pub fn longest_series(&self) -> u8 {
        match self {
            Self::Elimination(c) => c.series.default.max(c.series.finals),
        }
    }

    /// Máximo de mapas que um participante pode jogar na fase com
    /// `participants` participantes (o mesmo `worst_case_games` da chave
    /// montada com esse número). Erros: os de montar a fase com esse número
    /// (poucos ou participantes demais, melhor de N inválido).
    pub fn worst_case_games(&self, participants: usize) -> Result<u32, EngineError> {
        match self {
            Self::Elimination(c) => {
                if participants > MAX_PARTICIPANTS {
                    return Err(EngineError::TooManyParticipants {
                        max: MAX_PARTICIPANTS,
                        got: participants,
                    });
                }
                let seeded = (1..=participants as u128)
                    .map(|i| ParticipantId(Uuid::from_u128(i)))
                    .collect();
                Ok(EliminationBracket::new(*c, seeded)?.worst_case_games())
            }
        }
    }
}

#[cfg(test)]
mod tests;
