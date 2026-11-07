//! As partidas de uma fase de tabela, com os frags de cada uma e a
//! classificação que sai delas.

use super::ties::{self, FinalTie, PendingTie, TieOrder};
use super::{Rules, TableRow, compute};
use crate::core::{
    EngineError, Match, MatchId, MatchState, Occupant, ParticipantId, Placement, SeriesFrags, Slot,
    StageReport,
};
use crate::series::validate_outcome;

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
    /// `ratings[i]` é o rating congelado de `participants[i]`, para o menor
    /// rating; `None` sem rating.
    ratings: Vec<Option<i32>>,
    table: Vec<TableRow>,
    /// Quem decide o empate que sobra no fim (`ties`).
    final_tie: FinalTie,
    /// As decisões da organização gravadas para a fase.
    orders: Vec<TieOrder>,
    /// Os grupos que esperam decisão; só depois de `close`.
    pending: Vec<PendingTie>,
}

impl League {
    /// `participants` na ordem dos seeds; `matches` em ordem de id, feitas
    /// por `pairing` e `rest`. Sem ratings (`set_ratings`).
    pub fn new(rules: Rules, participants: Vec<ParticipantId>, matches: Vec<Match>) -> Self {
        debug_assert!(matches.is_sorted_by_key(|m| m.id));
        let frags = vec![None; matches.len()];
        let ratings = vec![None; participants.len()];
        let table = compute(rules, &participants, &ratings, &matches, &frags);
        Self {
            rules,
            participants,
            matches,
            frags,
            ratings,
            table,
            final_tie: FinalTie::Seed,
            orders: Vec::new(),
            pending: Vec::new(),
        }
    }

    /// Os ratings congelados, por participante (os de quem não participa
    /// ficam de fora; quem não aparece fica sem), sem refazer a tabela (o
    /// `replay` refaz uma vez, no fim, por `refresh`).
    pub fn set_ratings(&mut self, ratings: &[(ParticipantId, i32)]) {
        self.ratings = self
            .participants
            .iter()
            .map(|p| ratings.iter().find(|(q, _)| q == p).map(|&(_, r)| r))
            .collect();
    }

    /// Quem decide o empate que sobra no fim e as decisões gravadas. Vale a
    /// partir do próximo `close`.
    pub fn with_ties(mut self, final_tie: FinalTie, orders: Vec<TieOrder>) -> Self {
        self.final_tie = final_tie;
        self.orders = orders;
        self
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
    /// melhor de N, o empate inclusive (`validate_outcome`). W.O. não tem
    /// placar e não é validado.
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
        if let Some(winner) = outcome.winner()
            && winner != a
            && winner != b
        {
            return Err(EngineError::NotInMatch {
                match_id: id,
                participant: winner,
            });
        }
        validate_outcome(m.best_of, outcome)?;
        self.matches[idx].state = MatchState::Completed { a, b, outcome };
        self.frags[idx] = report.frags;
        Ok(())
    }

    /// Refaz a tabela, como durante a fase (o seed desempata, nada pendente).
    pub fn refresh(&mut self) {
        self.table = compute(
            self.rules,
            &self.participants,
            &self.ratings,
            &self.matches,
            &self.frags,
        );
        self.pending.clear();
    }

    /// No fim da fase (o formato sabe quando): aplica as decisões da
    /// organização à tabela e acha os empates que ainda esperam uma
    /// (`ties::settle`). Chamado depois de `refresh`.
    pub fn close(&mut self) {
        self.pending = ties::settle(&mut self.table, self.final_tie, &self.orders);
    }

    /// Os empates que esperam a organização; vazio antes de `close`.
    pub fn pending(&self) -> &[PendingTie] {
        &self.pending
    }

    /// `apply` e `refresh`, sem fechar os empates (os formatos sabem quando
    /// a fase acaba e fecham sozinhos).
    #[cfg(test)]
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

    /// A tabela como colocação: uma posição por participante, sem empate,
    /// menos os de um empate que espera a organização.
    pub fn placements(&self) -> Vec<Placement> {
        let waiting = |p: ParticipantId| self.pending.iter().any(|t| t.participants.contains(&p));
        self.table
            .iter()
            .filter(|r| !waiting(r.participant))
            .map(|r| Placement {
                participant: r.participant,
                rank: r.position,
                tied: 1,
            })
            .collect()
    }
}
