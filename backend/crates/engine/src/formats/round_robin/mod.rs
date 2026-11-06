//! Round robin simples: cada par de participantes se enfrenta uma vez, numa
//! fase única (a classificação final sai da tabela).
//!
//! Regras:
//! - Rodadas pelo método do círculo (tabela de Berger, `schedule`): com N
//!   par, N − 1 rodadas de N/2 partidas; com N ímpar, N rodadas e uma folga
//!   por rodada (cada um folga uma vez), que não vale ponto. A folga é uma
//!   partida `Bye` (a última da rodada), só para a tela mostrar quem folga.
//! - Todas as partidas ficam prontas desde o início: a rodada só agrupa, não
//!   trava a ordem. `suggest` indica a cada time qual série jogar agora (a
//!   ordem das rodadas, saltando só quando o adversário da vez ainda vai
//!   demorar), a partir do progresso que a API informa; só indica, nada muda
//!   em `replay`, `report` nem na tabela.
//! - Uma série (melhor de N, ímpar, 2, 4 ou 6) para todas as partidas; não há
//!   final. A série par pode empatar, sem vencedor (`series`).
//! - Classificação e desempates em `formats::table`: saldo de séries, os
//!   desempates da fase (`config.tiebreakers`; o padrão é confronto direto,
//!   saldo de mapas, saldo de frags; sem Buchholz) e o seed.
//! - Termina quando todas as séries estão decididas; só então há colocação
//!   (`placements`), uma posição por participante, sem empate.
//! - Até `MAX_PARTICIPANTS` (são N(N − 1)/2 partidas).

mod schedule;
mod suggest;
#[cfg(test)]
mod tests;

pub use suggest::{SeriesProgress, suggest};

use super::check_count;
use super::table::{League, Rules, TableRow, Tiebreakers};
use crate::core::{EngineError, Match, MatchId, Occupant, ParticipantId, Placement, StageReport};
use crate::series::validate_league_best_of;

/// Teto de participantes: 32 dá 496 partidas e 31 séries por participante.
pub const MAX_PARTICIPANTS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundRobinConfig {
    /// Melhor de N de todas as partidas: ímpar, 2, 4 ou 6.
    pub best_of: u8,
    /// Os desempates depois do saldo de séries, sem Buchholz; o padrão é
    /// `Tiebreakers::ROUND_ROBIN`.
    pub tiebreakers: Tiebreakers,
}

impl RoundRobinConfig {
    /// Erros: melhor de N inválido; `InvalidTiebreakers` com o Buchholz, que
    /// é só do suíço.
    pub fn validate(&self) -> Result<(), EngineError> {
        validate_league_best_of(self.best_of)?;
        self.tiebreakers.check(false)
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
    /// participante repetido, os de `RoundRobinConfig::validate`.
    pub fn new(config: RoundRobinConfig, seeded: Vec<ParticipantId>) -> Result<Self, EngineError> {
        super::check_participants(&seeded, MAX_PARTICIPANTS)?;
        config.validate()?;
        let matches = schedule::build(&seeded, config.best_of);
        Ok(Self {
            config,
            league: League::new(Rules::round_robin(config.tiebreakers), seeded, matches),
        })
    }

    /// Reconstrói o estado aplicando os resultados em ordem, com o rating
    /// congelado de cada participante (`ratings`, para o menor rating; ids que
    /// não são participantes são ignorados). Corrigir um resultado é chamar
    /// `replay` com o histórico corrigido: muda a tabela, e nada mais depende
    /// dele.
    pub fn replay(
        config: RoundRobinConfig,
        seeded: Vec<ParticipantId>,
        reports: impl IntoIterator<Item = StageReport>,
        ratings: &[(ParticipantId, i32)],
    ) -> Result<Self, EngineError> {
        let mut bracket = Self::new(config, seeded)?;
        bracket.league.set_ratings(ratings);
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
