//! Eliminação simples e dupla, para participantes de um contra um (duelo,
//! ou times em 2v2/3v3).
//!
//! A chave inteira é gerada no início: toda partida existe desde a criação,
//! com lados que apontam para participantes, para o vencedor/perdedor de
//! outra partida, ou para bye. Resultados vão resolvendo os lados.
//!
//! Regras:
//! - Tamanho da chave: `seeding::bracket_size(n)`; posições na ordem de
//!   `seeding::bracket_order`. Seeds acima de `n` são byes.
//! - Partida com um lado bye avança o outro sem jogar (`MatchState::Bye`);
//!   com os dois lados bye, propaga bye. `LoserOf` de uma partida bye é bye.
//! - Eliminação dupla: quem perde na chave de cima cai na de baixo; quem perde
//!   na de baixo está eliminado. A chave de baixo tem `2 * (log2(tamanho) - 1)`
//!   rodadas, alternando rodadas só entre sobreviventes da chave de baixo e
//!   rodadas que recebem os perdedores da rodada seguinte da chave de cima.
//!   Os perdedores que descem são posicionados de forma a evitar revanches
//!   cedo (convenção documentada na implementação).
//! - Grande final: campeão da chave de cima contra o da de baixo. Com
//!   `grand_final_reset`, se o campeão da chave de baixo vencer a rodada 1,
//!   há a rodada 2; senão a rodada 2 fica `NotNeeded`. Sem reset, a rodada 2
//!   não existe.
//! - `series.finals` vale para a final da eliminação simples e para a grande
//!   final (rodadas 1 e 2); as demais partidas usam `series.default`.
//! - Colocação: quem cai na mesma rodada de eliminação divide a faixa.
//!   Byes não contam como eliminados.

mod build;
mod resolve;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::core::{
    BracketSide, EngineError, Match, MatchId, MatchReport, MatchState, Occupant, Outcome,
    ParticipantId, Placement, Slot,
};
use crate::seeding::bracket_size;
use crate::series::{SeriesLength, validate_score};
use build::{build_matches, mid};
use resolve::loser_of;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EliminationKind {
    Single,
    Double { grand_final_reset: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EliminationConfig {
    pub kind: EliminationKind,
    pub series: SeriesLength,
}

/// Teto de participantes: muito acima de qualquer torneio real, e longe do
/// limite dos índices `u16` de `MatchId` e `Placement`.
pub const MAX_PARTICIPANTS: usize = 1024;

/// Só `Serialize`: o estado nunca é lido de fora. Para reidratar, use
/// `replay` com configuração, participantes e histórico, que revalida tudo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EliminationBracket {
    config: EliminationConfig,
    participants: Vec<ParticipantId>,
    matches: Vec<Match>,
}

impl EliminationBracket {
    /// Gera a chave. `seeded` está em ordem de seed: o primeiro é o seed 1.
    ///
    /// Erros: menos de 2 ou mais de `MAX_PARTICIPANTS` participantes,
    /// participante repetido, melhor de N inválido.
    pub fn new(config: EliminationConfig, seeded: Vec<ParticipantId>) -> Result<Self, EngineError> {
        if seeded.len() < 2 {
            return Err(EngineError::NotEnoughParticipants {
                min: 2,
                got: seeded.len(),
            });
        }
        if seeded.len() > MAX_PARTICIPANTS {
            return Err(EngineError::TooManyParticipants {
                max: MAX_PARTICIPANTS,
                got: seeded.len(),
            });
        }
        let mut seen = HashSet::new();
        for p in &seeded {
            if !seen.insert(*p) {
                return Err(EngineError::DuplicateParticipant(*p));
            }
        }
        config.series.validate()?;
        let mut matches = build_matches(config, &seeded);
        resolve::resolve(&mut matches);
        Ok(Self {
            config,
            participants: seeded,
            matches,
        })
    }

    /// Reconstrói o estado aplicando os resultados em ordem. Corrigir um
    /// resultado é chamar `replay` com o histórico corrigido; se a correção
    /// contradiz uma partida posterior, retorna o erro dessa partida.
    pub fn replay(
        config: EliminationConfig,
        seeded: Vec<ParticipantId>,
        reports: impl IntoIterator<Item = MatchReport>,
    ) -> Result<Self, EngineError> {
        let mut bracket = Self::new(config, seeded)?;
        for report in reports {
            bracket.report(report)?;
        }
        Ok(bracket)
    }

    pub fn config(&self) -> EliminationConfig {
        self.config
    }

    pub fn participants(&self) -> &[ParticipantId] {
        &self.participants
    }

    /// Todas as partidas, ordenadas por lado da chave, rodada e índice.
    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    pub fn get(&self, id: MatchId) -> Option<&Match> {
        self.matches.iter().find(|m| m.id == id)
    }

    /// Quem ocupa cada lado (`a`, `b`) da partida agora; `None` se ela não
    /// existe. Numa partida `NotNeeded`, os que a jogariam.
    pub fn occupants(&self, id: MatchId) -> Option<(Occupant, Occupant)> {
        let m = self.get(id)?;
        Some((
            resolve::occupant(&self.matches, m.a),
            resolve::occupant(&self.matches, m.b),
        ))
    }

    /// Partidas que podem ser jogadas agora (`MatchState::Ready`).
    pub fn ready_matches(&self) -> impl Iterator<Item = &Match> {
        self.matches
            .iter()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
    }

    /// Registra um resultado e resolve as partidas que dependem dele.
    ///
    /// Erros: partida inexistente, ainda não pronta, já com resultado;
    /// vencedor que não joga a partida; placar inválido para o melhor de N.
    /// W.O. não tem placar e não é validado contra o melhor de N.
    pub fn report(&mut self, report: MatchReport) -> Result<(), EngineError> {
        let id = report.match_id;
        let idx = self
            .matches
            .binary_search_by_key(&id, |m| m.id)
            .map_err(|_| EngineError::UnknownMatch(id))?;
        let m = self.matches[idx];
        let (a, b) = match m.state {
            MatchState::Ready { a, b } => (a, b),
            MatchState::Completed { .. } => return Err(EngineError::MatchAlreadyReported(id)),
            _ => return Err(EngineError::MatchNotReady(id)),
        };
        let winner = report.outcome.winner();
        if winner != a && winner != b {
            return Err(EngineError::NotInMatch {
                match_id: id,
                participant: winner,
            });
        }
        if let Outcome::Played { score, .. } = report.outcome {
            validate_score(m.best_of, score)?;
        }
        self.matches[idx].state = MatchState::Completed {
            a,
            b,
            outcome: report.outcome,
        };
        resolve::resolve(&mut self.matches);
        Ok(())
    }

    fn is_double(&self) -> bool {
        matches!(self.config.kind, EliminationKind::Double { .. })
    }

    /// Partida que decide o campeão, se já foi decidida.
    fn deciding_match(&self) -> Option<&Match> {
        let completed = |m: &&Match| matches!(m.state, MatchState::Completed { .. });
        if self.is_double() {
            let gf1 = self
                .get(mid(BracketSide::GrandFinal, 1, 1))
                .filter(completed)?;
            match self.get(mid(BracketSide::GrandFinal, 2, 1)) {
                Some(gf2) => match gf2.state {
                    MatchState::Completed { .. } => Some(gf2),
                    MatchState::NotNeeded => Some(gf1),
                    _ => None,
                },
                None => Some(gf1),
            }
        } else {
            let k = bracket_size(self.participants.len()).trailing_zeros() as usize;
            self.get(mid(BracketSide::Winners, k, 1)).filter(completed)
        }
    }

    pub fn is_finished(&self) -> bool {
        self.deciding_match().is_some()
    }

    /// Para cada partida, `(vencedor existe, perdedor existe)` de forma
    /// estrutural: depende só do número de participantes, não dos resultados.
    fn realness(&self) -> HashMap<MatchId, (bool, bool)> {
        let mut map: HashMap<MatchId, (bool, bool)> = HashMap::new();
        for m in &self.matches {
            let real = |slot: Slot| match slot {
                Slot::Participant(_) => true,
                Slot::Bye => false,
                Slot::WinnerOf(id) => map[&id].0,
                Slot::LoserOf(id) => map[&id].1,
            };
            let (ra, rb) = (real(m.a), real(m.b));
            map.insert(m.id, (ra || rb, ra && rb));
        }
        map
    }

    /// Colocações já definidas: eliminados (com a faixa final que vão ocupar)
    /// e, quando a chave termina, campeão e vice. Quem ainda está vivo não
    /// aparece.
    ///
    /// A faixa de cada rodada de eliminação é estrutural: começa depois de
    /// todas as rodadas de eliminação posteriores e tem o tamanho do número
    /// real de eliminados na rodada (byes não contam).
    pub fn standings(&self) -> Vec<Placement> {
        let real = self.realness();
        let k = bracket_size(self.participants.len()).trailing_zeros() as usize;
        let round_ids = |side: BracketSide, round: usize| -> Vec<MatchId> {
            self.matches
                .iter()
                .filter(|m| m.id.side == side && m.id.round as usize == round)
                .map(|m| m.id)
                .collect()
        };

        // Grupos de eliminação, do primeiro ao último. Cada grupo é a lista de
        // partidas cujo perdedor sai da chave.
        let mut groups: Vec<Vec<MatchId>> = Vec::new();
        if self.is_double() {
            for rho in 1..=2 * (k - 1) {
                groups.push(round_ids(BracketSide::Losers, rho));
            }
        } else {
            for r in 1..=k {
                groups.push(round_ids(BracketSide::Winners, r));
            }
        }

        let loser = |id: MatchId| match self.get(id).map(|m| m.state) {
            Some(MatchState::Completed { a, b, outcome }) => Some(loser_of(a, b, outcome)),
            _ => None,
        };

        let mut out = Vec::new();
        let decided = self.deciding_match();
        if let Some(Match {
            state: MatchState::Completed { outcome, .. },
            ..
        }) = decided
        {
            out.push(Placement {
                participant: outcome.winner(),
                rank: 1,
                tied: 1,
            });
        }

        let mut next_rank: u16 = 2;
        if self.is_double() {
            if let Some(p) = decided.and_then(|m| loser(m.id)) {
                out.push(Placement {
                    participant: p,
                    rank: 2,
                    tied: 1,
                });
            }
            next_rank = 3;
        }
        for group in groups.iter().rev() {
            let count = group.iter().filter(|id| real[*id].1).count() as u16;
            if count == 0 {
                continue;
            }
            for id in group {
                if let Some(p) = loser(*id) {
                    out.push(Placement {
                        participant: p,
                        rank: next_rank,
                        tied: count,
                    });
                }
            }
            next_rank += count;
        }
        out.sort_by_key(|p| p.rank);
        out
    }

    /// Maior soma de pesos ao longo de um caminho de um participante pela
    /// chave (vencendo ou perdendo). Partidas sem dois participantes reais
    /// (nascem com bye) pesam 0.
    fn worst_case(&self, weight: impl Fn(&Match) -> u32) -> u32 {
        let real = self.realness();
        let mut successors: HashMap<MatchId, Vec<MatchId>> = HashMap::new();
        for m in &self.matches {
            for slot in [m.a, m.b] {
                if let Slot::WinnerOf(src) | Slot::LoserOf(src) = slot {
                    successors.entry(src).or_default().push(m.id);
                }
            }
        }
        let mut best: HashMap<MatchId, u32> = HashMap::new();
        for m in self.matches.iter().rev() {
            let w = if real[&m.id].1 { weight(m) } else { 0 };
            let tail = successors
                .get(&m.id)
                .into_iter()
                .flatten()
                .map(|s| best[s])
                .max()
                .unwrap_or(0);
            best.insert(m.id, w + tail);
        }
        self.matches
            .iter()
            .filter(|m| [m.a, m.b].iter().any(|s| matches!(s, Slot::Participant(_))))
            .map(|m| best[&m.id])
            .max()
            .unwrap_or(0)
    }

    /// Máximo de partidas que um participante pode jogar, no pior caminho.
    /// Partidas que nascem com um lado bye não contam.
    pub fn worst_case_matches(&self) -> u32 {
        self.worst_case(|_| 1)
    }

    /// Máximo de mapas que um participante pode jogar: soma dos melhor de N
    /// ao longo do pior caminho. É o que alimenta a checagem de campeões.
    pub fn worst_case_games(&self) -> u32 {
        self.worst_case(|m| u32::from(m.best_of))
    }
}
