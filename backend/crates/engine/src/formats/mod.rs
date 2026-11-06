//! Formatos de chaveamento.
//!
//! Cada formato é uma pasta (`elimination/`, `round_robin/`, `swiss/`,
//! `placement/`); `table/` é o que os formatos de tabela (round robin e
//! suíço) dividem: partidas de lados fixos e a classificação com os
//! desempates (a disputa de posições usa só as partidas). `stage.rs` é a fase
//! de qualquer formato (`StageBracket`), o que a API usa.

pub mod elimination;
pub mod placement;
pub mod round_robin;
mod stage;
pub mod swiss;
pub mod table;

pub use stage::{StageBracket, StageHistory};

use std::collections::HashSet;

use crate::core::{EngineError, ParticipantId};
use elimination::{EliminationBracket, EliminationConfig};
use placement::PlacementConfig;
use round_robin::RoundRobinConfig;
use swiss::SwissConfig;

/// Formato de uma fase do torneio: é o que a fase guarda (a API grava em
/// colunas). O estado da fase nunca é guardado; vem do `replay` do formato.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageFormat {
    Elimination(EliminationConfig),
    RoundRobin(RoundRobinConfig),
    Swiss(SwissConfig),
    /// Disputa de posições.
    Placement(PlacementConfig),
}

impl StageFormat {
    /// Confere a configuração sem depender de participantes (ex.: melhor de N
    /// que o formato aceita, rodadas do suíço na faixa).
    pub fn validate(&self) -> Result<(), EngineError> {
        match self {
            Self::Elimination(c) => c.series.validate(),
            Self::RoundRobin(c) => c.validate(),
            Self::Swiss(c) => c.validate(),
            Self::Placement(c) => c.validate(),
        }
    }

    /// Maior melhor de N de uma partida da fase.
    pub fn longest_series(&self) -> u8 {
        match self {
            Self::Elimination(c) => c.series.default.max(c.series.finals),
            Self::RoundRobin(c) => c.best_of,
            Self::Swiss(c) => c.best_of,
            Self::Placement(c) => c.best_of,
        }
    }

    /// Máximo de mapas que um participante pode jogar na fase com
    /// `participants` participantes (o mesmo `worst_case_games` da fase
    /// montada com esse número). Erros: os de montar a fase com esse número
    /// (poucos ou participantes demais, melhor de N inválido, rodadas fora da
    /// faixa).
    pub fn worst_case_games(&self, participants: usize) -> Result<u32, EngineError> {
        match self {
            Self::Elimination(c) => {
                check_count(participants, elimination::MAX_PARTICIPANTS)?;
                let seeded = (1..=participants as i64).map(ParticipantId).collect();
                Ok(EliminationBracket::new(*c, seeded)?.worst_case_games())
            }
            Self::RoundRobin(c) => c.worst_case_games(participants),
            Self::Swiss(c) => c.worst_case_games(participants),
            Self::Placement(c) => c.worst_case_games(participants),
        }
    }
}

/// De 2 a `max` participantes.
pub(crate) fn check_count(got: usize, max: usize) -> Result<(), EngineError> {
    if got < 2 {
        return Err(EngineError::NotEnoughParticipants { min: 2, got });
    }
    if got > max {
        return Err(EngineError::TooManyParticipants { max, got });
    }
    Ok(())
}

/// De 2 a `max` participantes, sem repetir.
pub(crate) fn check_participants(seeded: &[ParticipantId], max: usize) -> Result<(), EngineError> {
    check_count(seeded.len(), max)?;
    let mut seen = HashSet::new();
    for p in seeded {
        if !seen.insert(*p) {
            return Err(EngineError::DuplicateParticipant(*p));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
