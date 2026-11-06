//! Disputa de posições: a ordem da chave forma os pares e quem vence cada série
//! fica com a melhor das duas posições. Serve de segunda etapa: a organização
//! inscreve os jogadores na ordem da classificação da primeira (ainda não há
//! fases ligadas).
//!
//! Regras:
//! - Pares pela ordem da chave: no par k, o seed 2k − 1 (lado `a`) contra o
//!   2k. Todas as séries ficam prontas no início, numa rodada só (`pl-1-k`).
//! - N ímpar: o último seed não tem par e fica na posição dele, sem jogar. É
//!   uma partida `Bye`, a última da rodada, só para a tela, como a folga do
//!   round robin.
//! - Uma série (melhor de N, ímpar: cada par precisa de vencedor) para todas;
//!   sem final nem desempate.
//! - Colocação: no par k, quem vence fica em 2k − 1 e quem perde em 2k, sem
//!   empate. Cada par define as duas posições quando é decidido (o sem par, a
//!   dele desde o início); a fase termina com todos decididos.
//! - W.O. vale como vitória. Quem está fora (desistiu ou saiu) a API resolve
//!   por W.O., como na eliminação.
//! - De 2 a `MAX_PARTICIPANTS`; cada um joga no máximo uma série.

#[cfg(test)]
mod tests;

use super::check_participants;
use super::table::{pairing, rest};
use crate::core::{
    BracketSide, EngineError, Match, MatchId, MatchReport, MatchState, Occupant, ParticipantId,
    Placement, Slot,
};
use crate::series::{validate_best_of, validate_outcome};

/// Teto de participantes, o mesmo do suíço (64 séries).
pub const MAX_PARTICIPANTS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacementConfig {
    /// Melhor de N de todas as séries: sempre ímpar, sem empate.
    pub best_of: u8,
}

impl PlacementConfig {
    /// Erros: melhor de N par (a série precisa de vencedor).
    pub fn validate(&self) -> Result<(), EngineError> {
        validate_best_of(self.best_of)
    }

    /// Máximo de mapas de um participante com `participants`: uma série.
    /// Erros: poucos ou participantes demais, melhor de N inválido.
    pub fn worst_case_games(&self, participants: usize) -> Result<u32, EngineError> {
        super::check_count(participants, MAX_PARTICIPANTS)?;
        self.validate()?;
        Ok(u32::from(self.best_of))
    }
}

/// Só leitura de fora: para reidratar, `replay` com configuração,
/// participantes e histórico, que revalida tudo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacementBracket {
    config: PlacementConfig,
    participants: Vec<ParticipantId>,
    /// Em ordem de id: o par k é a partida de índice k.
    matches: Vec<Match>,
}

impl PlacementBracket {
    /// Monta os pares. `seeded` está em ordem de seed: o primeiro é o seed 1.
    ///
    /// Erros: menos de 2 ou mais de `MAX_PARTICIPANTS` participantes,
    /// participante repetido, melhor de N par.
    pub fn new(config: PlacementConfig, seeded: Vec<ParticipantId>) -> Result<Self, EngineError> {
        check_participants(&seeded, MAX_PARTICIPANTS)?;
        config.validate()?;
        let matches = seeded
            .chunks(2)
            .zip(1u16..)
            .map(|(pair, index)| {
                let id = MatchId {
                    side: BracketSide::Placement,
                    round: 1,
                    index,
                };
                match *pair {
                    [a, b] => pairing(id, a, b, config.best_of),
                    [alone] => rest(id, alone, config.best_of),
                    _ => unreachable!("chunks(2) dá um ou dois"),
                }
            })
            .collect();
        Ok(Self {
            config,
            participants: seeded,
            matches,
        })
    }

    /// Reconstrói o estado aplicando os resultados em ordem. Corrigir um
    /// resultado é chamar `replay` com o histórico corrigido: só muda as
    /// posições do par dele.
    pub fn replay(
        config: PlacementConfig,
        seeded: Vec<ParticipantId>,
        reports: impl IntoIterator<Item = MatchReport>,
    ) -> Result<Self, EngineError> {
        let mut bracket = Self::new(config, seeded)?;
        for report in reports {
            bracket.report(report)?;
        }
        Ok(bracket)
    }

    pub fn config(&self) -> PlacementConfig {
        self.config
    }

    pub fn participants(&self) -> &[ParticipantId] {
        &self.participants
    }

    /// Os pares, na ordem; com N ímpar, o sem par por último.
    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    fn position(&self, id: MatchId) -> Option<usize> {
        self.matches.binary_search_by_key(&id, |m| m.id).ok()
    }

    pub fn get(&self, id: MatchId) -> Option<&Match> {
        self.position(id).map(|i| &self.matches[i])
    }

    /// Os lados da partida: sempre conhecidos (participante ou, no sem par,
    /// bye).
    pub fn occupants(&self, id: MatchId) -> Option<(Occupant, Occupant)> {
        let m = self.get(id)?;
        let occupant = |slot: Slot| match slot {
            Slot::Participant(p) => Occupant::Participant(p),
            Slot::Bye => Occupant::Bye,
            Slot::WinnerOf(_) | Slot::LoserOf(_) => Occupant::Unknown,
        };
        Some((occupant(m.a), occupant(m.b)))
    }

    /// Registra um resultado.
    ///
    /// Erros: partida inexistente, sem par (bye), já com resultado; vencedor
    /// que não joga a partida; placar inválido para o melhor de N, o empate
    /// inclusive (a série é ímpar). W.O. não tem placar e não é validado.
    pub fn report(&mut self, report: MatchReport) -> Result<(), EngineError> {
        let id = report.match_id;
        let idx = self.position(id).ok_or(EngineError::UnknownMatch(id))?;
        let m = self.matches[idx];
        let (a, b) = match m.state {
            MatchState::Ready { a, b } => (a, b),
            MatchState::Completed { .. } => return Err(EngineError::MatchAlreadyReported(id)),
            _ => return Err(EngineError::MatchNotReady(id)),
        };
        if let Some(winner) = report.outcome.winner()
            && winner != a
            && winner != b
        {
            return Err(EngineError::NotInMatch {
                match_id: id,
                participant: winner,
            });
        }
        validate_outcome(m.best_of, report.outcome)?;
        self.matches[idx].state = MatchState::Completed {
            a,
            b,
            outcome: report.outcome,
        };
        Ok(())
    }

    /// Todas as séries decididas.
    pub fn is_finished(&self) -> bool {
        !self
            .matches
            .iter()
            .any(|m| matches!(m.state, MatchState::Ready { .. }))
    }

    /// As posições já definidas, da melhor para a pior: as dos pares
    /// decididos (quem vence em 2k − 1, quem perde em 2k) e a do sem par.
    /// Uma por participante, sem empate; no fim, todos.
    pub fn placements(&self) -> Vec<Placement> {
        let place = |participant, rank| Placement {
            participant,
            rank,
            tied: 1,
        };
        let mut out = Vec::new();
        for (m, k) in self.matches.iter().zip(1u16..) {
            let best = 2 * k - 1;
            match m.state {
                MatchState::Completed { a, b, outcome } => {
                    // A série é ímpar: sempre tem vencedor.
                    let winner = outcome.winner().unwrap_or(a);
                    let loser = if winner == a { b } else { a };
                    out.push(place(winner, best));
                    out.push(place(loser, best + 1));
                }
                MatchState::Bye {
                    advanced: Some(alone),
                } => out.push(place(alone, best)),
                _ => {}
            }
        }
        out
    }

    /// Máximo de mapas de um participante: uma série.
    pub fn worst_case_games(&self) -> u32 {
        u32::from(self.config.best_of)
    }
}
