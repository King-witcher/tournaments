//! As partidas de uma fase de tabela, com os frags de cada uma e a
//! classificação que sai delas.

use super::{Rules, TableRow, compute};
use crate::core::{
    EngineError, Match, MatchId, MatchState, Occupant, Outcome, ParticipantId, Placement,
    SeriesFrags, Slot, StageReport,
};
use crate::series::validate_score;

/// Partida pronta entre `a` e `b`.
pub(crate) fn pairing(id: MatchId, a: ParticipantId, b: ParticipantId, best_of: u8) -> Match {
    Match {
        id,
        a: Slot::Participant(a),
        b: Slot::Participant(b),
        best_of,
        state: MatchState::Ready { a, b },
    }
}

/// A rodada sem jogo de `p`: a folga do round robin, o bye do suíço.
pub(crate) fn rest(id: MatchId, p: ParticipantId, best_of: u8) -> Match {
    Match {
        id,
        a: Slot::Participant(p),
        b: Slot::Bye,
        best_of,
        state: MatchState::Bye { advanced: Some(p) },
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct League {
    rules: Rules,
    participants: Vec<ParticipantId>,
    /// Em ordem de id.
    matches: Vec<Match>,
    /// `frags[i]` é da partida `matches[i]`.
    frags: Vec<Option<SeriesFrags>>,
    table: Vec<TableRow>,
}

impl League {
    /// `participants` na ordem dos seeds; `matches` em ordem de id, feitas
    /// por `pairing` e `rest`.
    pub fn new(rules: Rules, participants: Vec<ParticipantId>, matches: Vec<Match>) -> Self {
        debug_assert!(matches.is_sorted_by_key(|m| m.id));
        let frags = vec![None; matches.len()];
        let table = compute(rules, &participants, &matches, &frags);
        Self {
            rules,
            participants,
            matches,
            frags,
            table,
        }
    }

    pub fn participants(&self) -> &[ParticipantId] {
        &self.participants
    }

    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    fn position(&self, id: MatchId) -> Option<usize> {
        self.matches.binary_search_by_key(&id, |m| m.id).ok()
    }

    pub fn get(&self, id: MatchId) -> Option<&Match> {
        self.position(id).map(|i| &self.matches[i])
    }

    /// Os lados da partida: sempre conhecidos (participante ou bye).
    pub fn occupants(&self, id: MatchId) -> Option<(Occupant, Occupant)> {
        let m = self.get(id)?;
        let occupant = |slot: Slot| match slot {
            Slot::Participant(p) => Occupant::Participant(p),
            Slot::Bye => Occupant::Bye,
            Slot::WinnerOf(_) | Slot::LoserOf(_) => Occupant::Unknown,
        };
        Some((occupant(m.a), occupant(m.b)))
    }

    /// Registra o resultado sem refazer a tabela (o `replay` refaz uma vez,
    /// no fim, por `refresh`).
    ///
    /// Erros: partida inexistente, sem os dois lados (bye), já com
    /// resultado; vencedor que não joga a partida; placar inválido para o
    /// melhor de N. W.O. não tem placar e não é validado.
    pub fn apply(&mut self, report: StageReport) -> Result<(), EngineError> {
        let id = report.report.match_id;
        let idx = self.position(id).ok_or(EngineError::UnknownMatch(id))?;
        let m = self.matches[idx];
        let (a, b) = match m.state {
            MatchState::Ready { a, b } => (a, b),
            MatchState::Completed { .. } => return Err(EngineError::MatchAlreadyReported(id)),
            _ => return Err(EngineError::MatchNotReady(id)),
        };
        let outcome = report.report.outcome;
        let winner = outcome.winner();
        if winner != a && winner != b {
            return Err(EngineError::NotInMatch {
                match_id: id,
                participant: winner,
            });
        }
        if let Outcome::Played { score, .. } = outcome {
            validate_score(m.best_of, score)?;
        }
        self.matches[idx].state = MatchState::Completed { a, b, outcome };
        self.frags[idx] = report.frags;
        Ok(())
    }

    /// Refaz a tabela.
    pub fn refresh(&mut self) {
        self.table = compute(self.rules, &self.participants, &self.matches, &self.frags);
    }

    pub fn report(&mut self, report: StageReport) -> Result<(), EngineError> {
        self.apply(report)?;
        self.refresh();
        Ok(())
    }

    /// Nenhuma partida esperando resultado.
    pub fn all_decided(&self) -> bool {
        !self
            .matches
            .iter()
            .any(|m| matches!(m.state, MatchState::Ready { .. }))
    }

    /// A tabela em ordem de posição.
    pub fn table(&self) -> &[TableRow] {
        &self.table
    }

    /// A tabela como colocação: uma posição por participante, sem empate.
    pub fn placements(&self) -> Vec<Placement> {
        self.table
            .iter()
            .map(|r| Placement {
                participant: r.participant,
                rank: r.position,
                tied: 1,
            })
            .collect()
    }
}
