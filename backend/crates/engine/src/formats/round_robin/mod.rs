//! Round robin simples: cada par de participantes se enfrenta uma vez, numa
//! fase única (a classificação final sai da tabela).
//!
//! Regras:
//! - Rodadas pelo método do círculo (tabela de Berger, `schedule`): com N
//!   par, N − 1 rodadas de N/2 partidas; com N ímpar, N rodadas e uma folga
//!   por rodada (cada um folga uma vez), que não vale ponto. A folga é uma
//!   partida `Bye` (a última da rodada), só para a tela mostrar quem folga.
//! - Todas as partidas ficam prontas desde o início: a rodada só agrupa, não
//!   trava a ordem.
//! - Uma série (melhor de N, ímpar) para todas as partidas; não há final.
//! - Classificação e desempates em `formats::table`: pontos, confronto
//!   direto, saldo de mapas, saldo de frags, seed.
//! - Termina quando todas as séries estão decididas; só então há colocação
//!   (`placements`), uma posição por participante, sem empate.
//! - Até `MAX_PARTICIPANTS` (são N(N − 1)/2 partidas).

mod schedule;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use super::check_count;
use super::table::{self, League, TableRow};
use crate::core::{EngineError, Match, MatchId, Occupant, ParticipantId, Placement, StageReport};
use crate::series::validate_best_of;

/// Teto de participantes: 32 dá 496 partidas e 31 séries por participante.
pub const MAX_PARTICIPANTS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundRobinConfig {
    /// Melhor de N de todas as partidas.
    pub best_of: u8,
}

impl RoundRobinConfig {
    pub fn validate(&self) -> Result<(), EngineError> {
        validate_best_of(self.best_of)
    }

    /// Máximo de mapas de um participante com `participants`: N − 1 séries.
    /// Erros: poucos ou participantes demais, melhor de N inválido.
    pub fn worst_case_games(&self, participants: usize) -> Result<u32, EngineError> {
        check_count(participants, MAX_PARTICIPANTS)?;
        self.validate()?;
        Ok((participants as u32 - 1) * u32::from(self.best_of))
    }
}

/// Só leitura de fora: para reidratar, `replay` com configuração,
/// participantes e histórico, que revalida tudo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundRobinBracket {
    config: RoundRobinConfig,
    league: League,
}

impl RoundRobinBracket {
    /// Gera todas as rodadas. `seeded` está em ordem de seed: o primeiro é o
    /// seed 1.
    ///
    /// Erros: menos de 2 ou mais de `MAX_PARTICIPANTS` participantes,
    /// participante repetido, melhor de N inválido.
    pub fn new(config: RoundRobinConfig, seeded: Vec<ParticipantId>) -> Result<Self, EngineError> {
        super::check_participants(&seeded, MAX_PARTICIPANTS)?;
        config.validate()?;
        let matches = schedule::build(&seeded, config.best_of);
        Ok(Self {
            config,
            league: League::new(table::ROUND_ROBIN, seeded, matches),
        })
    }

    /// Reconstrói o estado aplicando os resultados em ordem. Corrigir um
    /// resultado é chamar `replay` com o histórico corrigido: muda a tabela,
    /// e nada mais depende dele.
    pub fn replay(
        config: RoundRobinConfig,
        seeded: Vec<ParticipantId>,
        reports: impl IntoIterator<Item = StageReport>,
    ) -> Result<Self, EngineError> {
        let mut bracket = Self::new(config, seeded)?;
        for report in reports {
            bracket.league.apply(report)?;
        }
        bracket.league.refresh();
        Ok(bracket)
    }

    pub fn config(&self) -> RoundRobinConfig {
        self.config
    }

    pub fn participants(&self) -> &[ParticipantId] {
        self.league.participants()
    }

    /// Todas as partidas, por rodada e índice.
    pub fn matches(&self) -> &[Match] {
        self.league.matches()
    }

    pub fn get(&self, id: MatchId) -> Option<&Match> {
        self.league.get(id)
    }

    pub fn occupants(&self, id: MatchId) -> Option<(Occupant, Occupant)> {
        self.league.occupants(id)
    }

    /// Número de rodadas: N − 1 com N par, N com N ímpar.
    pub fn rounds(&self) -> u16 {
        let n = self.participants().len();
        (n + n % 2 - 1) as u16
    }

    /// Registra um resultado. Erros: os de `MatchReport` numa partida (ver
    /// `EliminationBracket::report`); a folga não recebe resultado.
    pub fn report(&mut self, report: StageReport) -> Result<(), EngineError> {
        self.league.report(report)
    }

    pub fn is_finished(&self) -> bool {
        self.league.all_decided()
    }

    /// A tabela atual, em ordem de posição.
    pub fn table(&self) -> &[TableRow] {
        self.league.table()
    }

    /// A colocação final, quando termina; antes, vazia (ninguém sai antes do
    /// fim).
    pub fn placements(&self) -> Vec<Placement> {
        if self.is_finished() {
            self.league.placements()
        } else {
            Vec::new()
        }
    }

    /// Máximo de mapas de um participante: N − 1 séries.
    pub fn worst_case_games(&self) -> u32 {
        (self.participants().len() as u32 - 1) * u32::from(self.config.best_of)
    }
}
