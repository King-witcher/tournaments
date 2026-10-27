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

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::core::{
    BracketSide, EngineError, Match, MatchId, MatchReport, MatchState, Outcome, ParticipantId,
    Placement, Slot,
};
use crate::seeding::{bracket_order, bracket_size};
use crate::series::{SeriesLength, validate_score};

fn mid(side: BracketSide, round: usize, index: usize) -> MatchId {
    MatchId {
        side,
        round: round as u16,
        index: index as u16,
    }
}

/// Posição (1-based) da chave de cima, na rodada `j + 1`, cujo perdedor
/// enfrenta o sobrevivente `i` na rodada `2j` da chave de baixo (`c` partidas).
///
/// Convenção: em `j` ímpar a ordem dos que descem é invertida (o perdedor da
/// última partida da rodada enfrenta o primeiro sobrevivente); em `j` par a
/// ordem é mantida. Com resultados de favorito em chaves cheias de 8, 16 e 32,
/// isso evita que quem desce reencontre, na primeira partida da chave de
/// baixo, alguém que já enfrentou. Exceção inevitável: a última descida (perdedor
/// da final de cima contra o campeão da chave de baixo), que com favoritos é
/// sempre o seed 2 contra o seed 3, que já se enfrentaram na chave de cima.
fn drop_position(j: usize, i: usize, c: usize) -> usize {
    if !j.is_multiple_of(2) { c + 1 - i } else { i }
}

fn build_matches(config: EliminationConfig, seeded: &[ParticipantId]) -> Vec<Match> {
    let n = seeded.len();
    let size = bracket_size(n);
    let k = size.trailing_zeros() as usize;
    let double = matches!(config.kind, EliminationKind::Double { .. });
    let SeriesLength { default, finals } = config.series;
    let w = BracketSide::Winners;
    let l = BracketSide::Losers;
    let mut out = Vec::new();
    let push = |out: &mut Vec<Match>, id: MatchId, a: Slot, b: Slot, best_of: u8| {
        out.push(Match {
            id,
            a,
            b,
            best_of,
            state: MatchState::Pending,
        });
    };

    let order = bracket_order(size);
    let seat = |seed: usize| {
        if seed <= n {
            Slot::Participant(seeded[seed - 1])
        } else {
            Slot::Bye
        }
    };
    for i in 1..=size / 2 {
        let best_of = if !double && k == 1 { finals } else { default };
        push(
            &mut out,
            mid(w, 1, i),
            seat(order[2 * i - 2]),
            seat(order[2 * i - 1]),
            best_of,
        );
    }
    for r in 2..=k {
        let best_of = if !double && r == k { finals } else { default };
        for i in 1..=size >> r {
            push(
                &mut out,
                mid(w, r, i),
                Slot::WinnerOf(mid(w, r - 1, 2 * i - 1)),
                Slot::WinnerOf(mid(w, r - 1, 2 * i)),
                best_of,
            );
        }
    }

    if double {
        for rho in 1..=2 * (k - 1) {
            if rho == 1 {
                for i in 1..=size / 4 {
                    push(
                        &mut out,
                        mid(l, 1, i),
                        Slot::LoserOf(mid(w, 1, 2 * i - 1)),
                        Slot::LoserOf(mid(w, 1, 2 * i)),
                        default,
                    );
                }
            } else if rho.is_multiple_of(2) {
                let j = rho / 2;
                let c = size >> (j + 1);
                for i in 1..=c {
                    push(
                        &mut out,
                        mid(l, rho, i),
                        Slot::WinnerOf(mid(l, rho - 1, i)),
                        Slot::LoserOf(mid(w, j + 1, drop_position(j, i, c))),
                        default,
                    );
                }
            } else {
                let j = rho / 2;
                for i in 1..=size >> (j + 2) {
                    push(
                        &mut out,
                        mid(l, rho, i),
                        Slot::WinnerOf(mid(l, rho - 1, 2 * i - 1)),
                        Slot::WinnerOf(mid(l, rho - 1, 2 * i)),
                        default,
                    );
                }
            }
        }
        let gf = BracketSide::GrandFinal;
        let upper = Slot::WinnerOf(mid(w, k, 1));
        let lower = if k == 1 {
            Slot::LoserOf(mid(w, 1, 1))
        } else {
            Slot::WinnerOf(mid(l, 2 * (k - 1), 1))
        };
        push(&mut out, mid(gf, 1, 1), upper, lower, finals);
        if matches!(
            config.kind,
            EliminationKind::Double {
                grand_final_reset: true
            }
        ) {
            push(
                &mut out,
                mid(gf, 2, 1),
                Slot::LoserOf(mid(gf, 1, 1)),
                Slot::WinnerOf(mid(gf, 1, 1)),
                finals,
            );
        }
    }
    out.sort_by_key(|m| m.id);
    out
}

#[derive(Clone, Copy)]
enum Side {
    Unknown,
    Known(ParticipantId),
    Bye,
}

fn loser_of(a: ParticipantId, b: ParticipantId, outcome: Outcome) -> ParticipantId {
    if outcome.winner() == a { b } else { a }
}

fn lookup(matches: &[Match], id: MatchId) -> Option<&Match> {
    matches
        .binary_search_by_key(&id, |m| m.id)
        .ok()
        .map(|i| &matches[i])
}

fn resolve_slot(matches: &[Match], slot: Slot) -> Side {
    match slot {
        Slot::Participant(p) => Side::Known(p),
        Slot::Bye => Side::Bye,
        Slot::WinnerOf(id) => match lookup(matches, id).map(|m| m.state) {
            Some(MatchState::Completed { outcome, .. }) => Side::Known(outcome.winner()),
            Some(MatchState::Bye {
                advanced: Some(p), ..
            }) => Side::Known(p),
            Some(MatchState::Bye { advanced: None }) => Side::Bye,
            _ => Side::Unknown,
        },
        Slot::LoserOf(id) => match lookup(matches, id).map(|m| m.state) {
            Some(MatchState::Completed { a, b, outcome }) => Side::Known(loser_of(a, b, outcome)),
            Some(MatchState::Bye { .. }) => Side::Bye,
            _ => Side::Unknown,
        },
    }
}

/// Resolve os estados pendentes. Toda dependência aponta para uma partida
/// anterior na ordem de `matches`, então uma passada basta (os byes cascateiam).
fn resolve(matches: &mut [Match]) {
    for i in 0..matches.len() {
        if !matches!(matches[i].state, MatchState::Pending) {
            continue;
        }
        let m = matches[i];
        if m.id.side == BracketSide::GrandFinal
            && m.id.round == 2
            && let Some(Match {
                state: MatchState::Completed { a, outcome, .. },
                ..
            }) = lookup(matches, mid(BracketSide::GrandFinal, 1, 1))
            && outcome.winner() == *a
        {
            matches[i].state = MatchState::NotNeeded;
            continue;
        }
        let new_state = match (resolve_slot(matches, m.a), resolve_slot(matches, m.b)) {
            (Side::Known(a), Side::Known(b)) => MatchState::Ready { a, b },
            (Side::Bye, Side::Bye) => MatchState::Bye { advanced: None },
            (Side::Bye, Side::Known(x)) | (Side::Known(x), Side::Bye) => {
                MatchState::Bye { advanced: Some(x) }
            }
            _ => continue,
        };
        matches[i].state = new_state;
    }
}

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
        resolve(&mut matches);
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
        resolve(&mut self.matches);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::SeriesScore;
    use crate::series::wins_needed;
    use proptest::prelude::*;
    use uuid::Uuid;

    const W: BracketSide = BracketSide::Winners;
    const L: BracketSide = BracketSide::Losers;
    const GF: BracketSide = BracketSide::GrandFinal;

    fn p(n: u128) -> ParticipantId {
        ParticipantId(Uuid::from_u128(n))
    }

    fn seeds(n: u128) -> Vec<ParticipantId> {
        (1..=n).map(p).collect()
    }

    fn id(side: BracketSide, round: u16, index: u16) -> MatchId {
        MatchId { side, round, index }
    }

    fn single(best_of: u8, finals: u8) -> EliminationConfig {
        EliminationConfig {
            kind: EliminationKind::Single,
            series: SeriesLength {
                default: best_of,
                finals,
            },
        }
    }

    fn double(reset: bool) -> EliminationConfig {
        EliminationConfig {
            kind: EliminationKind::Double {
                grand_final_reset: reset,
            },
            series: SeriesLength {
                default: 3,
                finals: 5,
            },
        }
    }

    #[test]
    fn participant_cap() {
        let max = MAX_PARTICIPANTS as u128;
        let full = EliminationBracket::new(double(true), seeds(max)).unwrap();
        assert!(full.worst_case_games() > 0);
        assert!(full.standings().is_empty());

        assert_eq!(
            EliminationBracket::new(double(true), seeds(max + 1)),
            Err(EngineError::TooManyParticipants {
                max: MAX_PARTICIPANTS,
                got: MAX_PARTICIPANTS + 1,
            })
        );
    }

    fn played(m: &Match, winner: ParticipantId) -> MatchReport {
        MatchReport {
            match_id: m.id,
            outcome: Outcome::Played {
                winner,
                score: SeriesScore {
                    winner: wins_needed(m.best_of),
                    loser: 0,
                },
            },
        }
    }

    fn ready_pair(b: &EliminationBracket, mid: MatchId) -> (ParticipantId, ParticipantId) {
        match b.get(mid).unwrap().state {
            MatchState::Ready { a, b } => (a, b),
            other => panic!("{mid:?} não está pronta: {other:?}"),
        }
    }

    /// Vence quem tem o menor id (o seed menor). Para antes da grande final
    /// se `stop_at_gf`. Retorna a ordem dos resultados.
    fn play_favorites(b: &mut EliminationBracket, stop_at_gf: bool) -> Vec<MatchReport> {
        let mut reports = Vec::new();
        loop {
            let next = b
                .ready_matches()
                .find(|m| !(stop_at_gf && m.id.side == GF))
                .copied();
            let Some(m) = next else { break };
            let MatchState::Ready { a, b: other } = m.state else {
                unreachable!()
            };
            let r = played(&m, a.min(other));
            b.report(r).unwrap();
            reports.push(r);
        }
        reports
    }

    fn count(b: &EliminationBracket, side: BracketSide) -> usize {
        b.matches().iter().filter(|m| m.id.side == side).count()
    }

    // ---- validação ----

    #[test]
    fn new_validates_input() {
        assert_eq!(
            EliminationBracket::new(single(3, 3), seeds(1)).unwrap_err(),
            EngineError::NotEnoughParticipants { min: 2, got: 1 }
        );
        assert_eq!(
            EliminationBracket::new(single(3, 3), vec![p(1), p(2), p(1)]).unwrap_err(),
            EngineError::DuplicateParticipant(p(1))
        );
        assert_eq!(
            EliminationBracket::new(single(2, 3), seeds(4)).unwrap_err(),
            EngineError::InvalidBestOf(2)
        );
        assert_eq!(
            EliminationBracket::new(single(3, 0), seeds(4)).unwrap_err(),
            EngineError::InvalidBestOf(0)
        );
    }

    // ---- eliminação simples ----

    #[test]
    fn single_two_participants() {
        let mut b = EliminationBracket::new(single(3, 5), seeds(2)).unwrap();
        assert_eq!(b.matches().len(), 1);
        assert_eq!(b.get(id(W, 1, 1)).unwrap().best_of, 5);
        assert!(b.standings().is_empty());
        let m = *b.get(id(W, 1, 1)).unwrap();
        b.report(played(&m, p(2))).unwrap();
        assert!(b.is_finished());
        assert_eq!(
            b.standings(),
            vec![
                Placement {
                    participant: p(2),
                    rank: 1,
                    tied: 1
                },
                Placement {
                    participant: p(1),
                    rank: 2,
                    tied: 1
                },
            ]
        );
        assert_eq!(b.worst_case_matches(), 1);
        assert_eq!(b.worst_case_games(), 5);
    }

    #[test]
    fn single_three_gives_seed_one_a_bye() {
        let b = EliminationBracket::new(single(3, 5), seeds(3)).unwrap();
        assert_eq!(b.matches().len(), 3);
        assert_eq!(
            b.get(id(W, 1, 1)).unwrap().state,
            MatchState::Bye {
                advanced: Some(p(1))
            }
        );
        assert_eq!(
            b.get(id(W, 1, 2)).unwrap().state,
            MatchState::Ready { a: p(2), b: p(3) }
        );
        assert_eq!(b.get(id(W, 2, 1)).unwrap().state, MatchState::Pending);
        assert_eq!(b.worst_case_matches(), 2);
        assert_eq!(b.worst_case_games(), 3 + 5);
    }

    #[test]
    fn single_four_five_and_eight_shapes() {
        let b4 = EliminationBracket::new(single(3, 5), seeds(4)).unwrap();
        assert_eq!(b4.matches().len(), 3);
        assert_eq!(b4.ready_matches().count(), 2);
        assert_eq!(b4.get(id(W, 1, 1)).unwrap().a, Slot::Participant(p(1)));
        assert_eq!(b4.get(id(W, 1, 1)).unwrap().b, Slot::Participant(p(4)));
        assert_eq!(b4.get(id(W, 2, 1)).unwrap().a, Slot::WinnerOf(id(W, 1, 1)));
        assert_eq!(b4.get(id(W, 2, 1)).unwrap().b, Slot::WinnerOf(id(W, 1, 2)));

        let b5 = EliminationBracket::new(single(3, 5), seeds(5)).unwrap();
        assert_eq!(b5.matches().len(), 7);
        assert_eq!(b5.ready_matches().count(), 2);
        assert_eq!(b5.worst_case_matches(), 3);

        let b8 = EliminationBracket::new(single(3, 5), seeds(8)).unwrap();
        assert_eq!(b8.matches().len(), 7);
        assert_eq!(b8.ready_matches().count(), 4);
        assert_eq!(b8.worst_case_matches(), 3);
        assert_eq!(b8.worst_case_games(), 3 + 3 + 5);
        let ids: Vec<_> = b8.matches().iter().map(|m| m.id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn six_participants_give_seeds_one_and_two_a_bye() {
        for config in [single(3, 5), double(true)] {
            let b = EliminationBracket::new(config, seeds(6)).unwrap();
            assert_eq!(
                b.get(id(W, 1, 1)).unwrap().state,
                MatchState::Bye {
                    advanced: Some(p(1))
                }
            );
            assert_eq!(
                b.get(id(W, 1, 3)).unwrap().state,
                MatchState::Bye {
                    advanced: Some(p(2))
                }
            );
            assert_eq!(
                b.get(id(W, 1, 2)).unwrap().state,
                MatchState::Ready { a: p(4), b: p(5) }
            );
            assert_eq!(
                b.get(id(W, 1, 4)).unwrap().state,
                MatchState::Ready { a: p(3), b: p(6) }
            );
        }
    }

    #[test]
    fn single_standings_partial_and_final() {
        let mut b = EliminationBracket::new(single(3, 5), seeds(8)).unwrap();
        let first = *b.get(id(W, 1, 1)).unwrap();
        b.report(played(&first, p(1))).unwrap();
        assert_eq!(
            b.standings(),
            vec![Placement {
                participant: p(8),
                rank: 5,
                tied: 4
            }]
        );
        play_favorites(&mut b, false);
        assert!(b.is_finished());
        let s = b.standings();
        assert_eq!(s.len(), 8);
        assert_eq!(
            s[0],
            Placement {
                participant: p(1),
                rank: 1,
                tied: 1
            }
        );
        assert_eq!(
            s[1],
            Placement {
                participant: p(2),
                rank: 2,
                tied: 1
            }
        );
        assert!(s[2..4].iter().all(|x| x.rank == 3 && x.tied == 2));
        assert!(s[4..].iter().all(|x| x.rank == 5 && x.tied == 4));
    }

    #[test]
    fn single_standings_ignore_byes() {
        let mut b = EliminationBracket::new(single(1, 1), seeds(5)).unwrap();
        play_favorites(&mut b, false);
        let s = b.standings();
        assert_eq!(s.len(), 5);
        // 5 vs 4 na primeira rodada: único eliminado dela, faixa de tamanho 1.
        assert_eq!(
            s[4],
            Placement {
                participant: p(5),
                rank: 5,
                tied: 1
            }
        );
        assert!(s[2..4].iter().all(|x| x.rank == 3 && x.tied == 2));
    }

    // ---- eliminação dupla ----

    #[test]
    fn double_two_participants() {
        let mut b = EliminationBracket::new(double(true), seeds(2)).unwrap();
        assert_eq!(count(&b, L), 0);
        assert_eq!(count(&b, GF), 2);
        assert_eq!(b.get(id(GF, 1, 1)).unwrap().a, Slot::WinnerOf(id(W, 1, 1)));
        assert_eq!(b.get(id(GF, 1, 1)).unwrap().b, Slot::LoserOf(id(W, 1, 1)));
        let m = *b.get(id(W, 1, 1)).unwrap();
        b.report(played(&m, p(1))).unwrap();
        assert_eq!(
            b.get(id(GF, 1, 1)).unwrap().state,
            MatchState::Ready { a: p(1), b: p(2) }
        );
        let gf = *b.get(id(GF, 1, 1)).unwrap();
        b.report(played(&gf, p(2))).unwrap();
        assert!(!b.is_finished());
        assert_eq!(
            b.get(id(GF, 2, 1)).unwrap().state,
            MatchState::Ready { a: p(1), b: p(2) }
        );
        let gf2 = *b.get(id(GF, 2, 1)).unwrap();
        b.report(played(&gf2, p(2))).unwrap();
        assert!(b.is_finished());
        assert_eq!(
            b.standings(),
            vec![
                Placement {
                    participant: p(2),
                    rank: 1,
                    tied: 1
                },
                Placement {
                    participant: p(1),
                    rank: 2,
                    tied: 1
                },
            ]
        );
        assert_eq!(b.worst_case_matches(), 3);
        assert_eq!(b.worst_case_games(), 3 + 5 + 5);
    }

    #[test]
    fn double_shapes() {
        // (participantes, partidas das chaves de cima e de baixo)
        for (n, upper, lower) in [(3, 3, 2), (4, 3, 2), (5, 7, 6), (8, 7, 6)] {
            let b = EliminationBracket::new(double(true), seeds(n)).unwrap();
            assert_eq!(count(&b, W), upper, "n={n}");
            assert_eq!(count(&b, L), lower, "n={n}");
            assert_eq!(count(&b, GF), 2, "n={n}");
            let b = EliminationBracket::new(double(false), seeds(n)).unwrap();
            assert_eq!(count(&b, GF), 1, "n={n}");
            assert_eq!(b.matches().last().unwrap().id, id(GF, 1, 1));
        }
        let b = EliminationBracket::new(double(true), seeds(3)).unwrap();
        // L1 junta o perdedor do bye com o de 2 x 3: fica pendente até 2 x 3 acabar.
        assert_eq!(b.get(id(L, 1, 1)).unwrap().state, MatchState::Pending);
        let b = EliminationBracket::new(double(true), seeds(4)).unwrap();
        assert_eq!(b.get(id(L, 1, 1)).unwrap().a, Slot::LoserOf(id(W, 1, 1)));
        assert_eq!(b.get(id(L, 1, 1)).unwrap().b, Slot::LoserOf(id(W, 1, 2)));
        assert_eq!(b.get(id(L, 2, 1)).unwrap().a, Slot::WinnerOf(id(L, 1, 1)));
        assert_eq!(b.get(id(L, 2, 1)).unwrap().b, Slot::LoserOf(id(W, 2, 1)));
        // melhor de N: só a grande final usa `finals`
        assert_eq!(b.get(id(W, 2, 1)).unwrap().best_of, 3);
        assert_eq!(b.get(id(L, 2, 1)).unwrap().best_of, 3);
        assert_eq!(b.get(id(GF, 1, 1)).unwrap().best_of, 5);
        assert_eq!(b.get(id(GF, 2, 1)).unwrap().best_of, 5);
    }

    #[test]
    fn double_byes_cascade_into_losers_bracket() {
        let mut b = EliminationBracket::new(double(true), seeds(3)).unwrap();
        assert_eq!(b.get(id(L, 1, 1)).unwrap().state, MatchState::Pending);
        assert_eq!(b.ready_matches().count(), 1);
        let m = *b.get(id(W, 1, 2)).unwrap();
        b.report(played(&m, p(2))).unwrap();
        // o perdedor de 2 x 3 avança sem jogar na L1 e espera o perdedor da final de cima.
        assert_eq!(
            b.get(id(L, 1, 1)).unwrap().state,
            MatchState::Bye {
                advanced: Some(p(3))
            }
        );
        assert_eq!(b.get(id(L, 2, 1)).unwrap().state, MatchState::Pending);
        let m = *b.get(id(W, 2, 1)).unwrap();
        b.report(played(&m, p(1))).unwrap();
        assert_eq!(
            b.get(id(L, 2, 1)).unwrap().state,
            MatchState::Ready { a: p(3), b: p(2) }
        );
    }

    #[test]
    fn grand_final_without_reset_ends_after_round_one() {
        let mut b = EliminationBracket::new(double(false), seeds(4)).unwrap();
        play_favorites(&mut b, true);
        assert!(!b.is_finished());
        let (up, low) = ready_pair(&b, id(GF, 1, 1));
        let gf = *b.get(id(GF, 1, 1)).unwrap();
        b.report(played(&gf, low)).unwrap();
        assert!(b.is_finished());
        assert!(b.get(id(GF, 2, 1)).is_none());
        let s = b.standings();
        assert_eq!(s[0].participant, low);
        assert_eq!(
            s[1],
            Placement {
                participant: up,
                rank: 2,
                tied: 1
            }
        );
    }

    #[test]
    fn grand_final_reset_not_needed_when_upper_champion_wins() {
        let mut b = EliminationBracket::new(double(true), seeds(4)).unwrap();
        play_favorites(&mut b, true);
        assert_eq!(b.get(id(GF, 2, 1)).unwrap().state, MatchState::Pending);
        let (up, _) = ready_pair(&b, id(GF, 1, 1));
        let gf = *b.get(id(GF, 1, 1)).unwrap();
        b.report(played(&gf, up)).unwrap();
        assert_eq!(b.get(id(GF, 2, 1)).unwrap().state, MatchState::NotNeeded);
        assert!(b.is_finished());
        assert_eq!(b.standings()[0].participant, up);
        assert_eq!(b.ready_matches().count(), 0);
    }

    #[test]
    fn grand_final_reset_needed_when_lower_champion_wins() {
        let mut b = EliminationBracket::new(double(true), seeds(4)).unwrap();
        play_favorites(&mut b, true);
        let (up, low) = ready_pair(&b, id(GF, 1, 1));
        let gf = *b.get(id(GF, 1, 1)).unwrap();
        b.report(played(&gf, low)).unwrap();
        assert!(!b.is_finished());
        assert_eq!(
            b.get(id(GF, 2, 1)).unwrap().state,
            MatchState::Ready { a: up, b: low }
        );
        let gf2 = *b.get(id(GF, 2, 1)).unwrap();
        b.report(played(&gf2, up)).unwrap();
        assert!(b.is_finished());
        let s = b.standings();
        assert_eq!(s[0].participant, up);
        assert_eq!(
            s[1],
            Placement {
                participant: low,
                rank: 2,
                tied: 1
            }
        );
    }

    #[test]
    fn double_standings_partial_and_final() {
        let mut b = EliminationBracket::new(double(false), seeds(8)).unwrap();
        // ninguém eliminado depois da chave de cima
        for round in 1..=3 {
            for m in b
                .matches()
                .iter()
                .filter(|m| m.id.side == W && m.id.round == round)
                .copied()
                .collect::<Vec<_>>()
            {
                let (x, y) = ready_pair(&b, m.id);
                b.report(played(&m, x.min(y))).unwrap();
            }
        }
        assert!(b.standings().is_empty());
        play_favorites(&mut b, false);
        assert!(b.is_finished());
        let s = b.standings();
        assert_eq!(s.len(), 8);
        assert_eq!(
            s[0],
            Placement {
                participant: p(1),
                rank: 1,
                tied: 1
            }
        );
        assert_eq!(
            s[1],
            Placement {
                participant: p(2),
                rank: 2,
                tied: 1
            }
        );
        // L6 (final da chave de baixo): 3º; L5: 4º; L4: 5º-6º; L3: 7º... conforme as rodadas.
        let ranks: Vec<(u16, u16)> = s.iter().map(|x| (x.rank, x.tied)).collect();
        assert_eq!(
            ranks,
            vec![
                (1, 1),
                (2, 1),
                (3, 1),
                (4, 1),
                (5, 2),
                (5, 2),
                (7, 2),
                (7, 2)
            ]
        );
    }

    #[test]
    fn double_eliminated_band_is_final_when_eliminated() {
        let mut b = EliminationBracket::new(double(true), seeds(8)).unwrap();
        // joga tudo o que está pronto, só uma vez por rodada de passagem
        let mut early = Vec::new();
        for _ in 0..3 {
            let ready: Vec<Match> = b.ready_matches().copied().collect();
            for m in ready {
                let (x, y) = ready_pair(&b, m.id);
                b.report(played(&m, x.min(y))).unwrap();
            }
            early.push(b.standings());
        }
        let last = early.last().unwrap().clone();
        let mut fin = EliminationBracket::new(double(true), seeds(8)).unwrap();
        play_favorites(&mut fin, true);
        let (up, _) = ready_pair(&fin, id(GF, 1, 1));
        let gf = *fin.get(id(GF, 1, 1)).unwrap();
        fin.report(played(&gf, up)).unwrap();
        let end = fin.standings();
        for pl in last {
            assert!(end.contains(&pl), "{pl:?} mudou de faixa");
        }
    }

    // ---- report ----

    #[test]
    fn report_errors() {
        let mut b = EliminationBracket::new(single(3, 5), seeds(4)).unwrap();
        let m1 = *b.get(id(W, 1, 1)).unwrap();
        let unknown = id(W, 9, 9);
        assert_eq!(
            b.report(MatchReport {
                match_id: unknown,
                outcome: Outcome::Walkover { winner: p(1) }
            }),
            Err(EngineError::UnknownMatch(unknown))
        );
        let final_ = *b.get(id(W, 2, 1)).unwrap();
        assert_eq!(
            b.report(played(&final_, p(1))),
            Err(EngineError::MatchNotReady(final_.id))
        );
        assert_eq!(
            b.report(played(&m1, p(2))),
            Err(EngineError::NotInMatch {
                match_id: m1.id,
                participant: p(2)
            })
        );
        let bad = SeriesScore {
            winner: 1,
            loser: 0,
        };
        assert_eq!(
            b.report(MatchReport {
                match_id: m1.id,
                outcome: Outcome::Played {
                    winner: p(1),
                    score: bad
                }
            }),
            Err(EngineError::InvalidScore {
                best_of: 3,
                score: bad
            })
        );
        // erros não alteram o estado
        assert!(matches!(
            b.get(m1.id).unwrap().state,
            MatchState::Ready { .. }
        ));
        b.report(played(&m1, p(1))).unwrap();
        assert_eq!(
            b.report(played(&m1, p(1))),
            Err(EngineError::MatchAlreadyReported(m1.id))
        );
    }

    #[test]
    fn report_on_bye_match_is_not_ready() {
        let mut b = EliminationBracket::new(single(3, 5), seeds(3)).unwrap();
        let bye = *b.get(id(W, 1, 1)).unwrap();
        assert_eq!(
            b.report(played(&bye, p(1))),
            Err(EngineError::MatchNotReady(bye.id))
        );
    }

    #[test]
    fn walkover_skips_score_validation() {
        let mut b = EliminationBracket::new(single(3, 5), seeds(4)).unwrap();
        b.report(MatchReport {
            match_id: id(W, 1, 1),
            outcome: Outcome::Walkover { winner: p(4) },
        })
        .unwrap();
        assert_eq!(b.get(id(W, 2, 1)).unwrap().a, Slot::WinnerOf(id(W, 1, 1)));
        assert_eq!(
            b.standings(),
            vec![Placement {
                participant: p(1),
                rank: 3,
                tied: 2
            }]
        );
        assert_eq!(
            b.report(MatchReport {
                match_id: id(W, 1, 2),
                outcome: Outcome::Walkover { winner: p(9) }
            }),
            Err(EngineError::NotInMatch {
                match_id: id(W, 1, 2),
                participant: p(9)
            })
        );
    }

    #[test]
    fn replay_matches_incremental_state() {
        for config in [single(3, 5), double(true), double(false)] {
            for n in [2u128, 3, 5, 8, 11] {
                let mut inc = EliminationBracket::new(config, seeds(n)).unwrap();
                let reports = play_favorites(&mut inc, false);
                let replayed = EliminationBracket::replay(config, seeds(n), reports).unwrap();
                assert_eq!(inc, replayed);
            }
        }
    }

    #[test]
    fn replay_returns_error_of_contradicted_match() {
        let config = single(3, 5);
        let mut b = EliminationBracket::new(config, seeds(4)).unwrap();
        let reports = play_favorites(&mut b, false);
        // corrige o primeiro resultado: p(4) vence p(1); a semifinal reportada depois contradiz
        let m = *b.get(id(W, 1, 1)).unwrap();
        let mut fixed = reports.clone();
        fixed[0] = played(&m, p(4));
        let err = EliminationBracket::replay(config, seeds(4), fixed).unwrap_err();
        assert_eq!(
            err,
            EngineError::NotInMatch {
                match_id: id(W, 2, 1),
                participant: p(1)
            }
        );
    }

    // ---- pior caso ----

    #[test]
    fn worst_case_double_sixteen() {
        let b = EliminationBracket::new(double(true), seeds(16)).unwrap();
        assert_eq!(b.worst_case_matches(), 9);
        assert_eq!(b.worst_case_games(), 7 * 3 + 2 * 5);
        let b = EliminationBracket::new(double(false), seeds(16)).unwrap();
        assert_eq!(b.worst_case_matches(), 8);
        assert_eq!(b.worst_case_games(), 7 * 3 + 5);
    }

    // ---- sem revanche cedo ----

    #[test]
    fn dropping_players_do_not_meet_old_opponents_first() {
        for n in [8u128, 16, 32] {
            let mut b = EliminationBracket::new(double(false), seeds(n)).unwrap();
            let mut met: HashSet<(ParticipantId, ParticipantId)> = HashSet::new();
            loop {
                let next = b.ready_matches().next().copied();
                let Some(m) = next else { break };
                let (x, y) = ready_pair(&b, m.id);
                // A última descida (perdedor da final de cima contra o campeão da chave de baixo)
                // é sempre revanche com resultados de favorito: o seed 3 perdeu para o 2.
                let last_drop = 2 * (n.trailing_zeros() as u16 - 1);
                if m.id.side == L && m.id.round.is_multiple_of(2) && m.id.round < last_drop {
                    assert!(!met.contains(&(x, y)), "n={n}: revanche em {:?}", m.id);
                }
                met.insert((x, y));
                met.insert((y, x));
                b.report(played(&m, x.min(y))).unwrap();
            }
            assert!(b.is_finished());
        }
    }

    // ---- propriedades ----

    #[derive(Default)]
    struct Tally {
        matches: u32,
        games: u32,
        losses: u32,
    }

    fn check_invariants(b: &EliminationBracket) {
        let n = b.participants().len();
        let double = b.is_double();
        assert!(b.is_finished());

        // standings: cada participante uma vez, faixas consistentes
        let s = b.standings();
        assert_eq!(s.len(), n);
        let distinct: HashSet<_> = s.iter().map(|x| x.participant).collect();
        assert_eq!(distinct.len(), n);
        assert_eq!(s.iter().filter(|x| x.rank == 1).count(), 1);
        let mut next = 1u16;
        let mut i = 0;
        while i < s.len() {
            let (rank, tied) = (s[i].rank, s[i].tied);
            assert_eq!(rank, next, "buraco ou sobreposição em {s:?}");
            for x in &s[i..i + tied as usize] {
                assert_eq!((x.rank, x.tied), (rank, tied));
            }
            i += tied as usize;
            next += tied;
        }
        assert_eq!(next as usize, n + 1);

        // jogos por participante
        let mut tally: HashMap<ParticipantId, Tally> = HashMap::new();
        let mut appearances: HashSet<(ParticipantId, BracketSide, u16)> = HashSet::new();
        for m in b.matches() {
            let (x, y, played_it) = match m.state {
                MatchState::Completed { a, b, outcome } => {
                    let loser = loser_of(a, b, outcome);
                    let t = tally.entry(loser).or_default();
                    t.losses += 1;
                    for q in [a, b] {
                        let t = tally.entry(q).or_default();
                        t.matches += 1;
                        t.games += u32::from(m.best_of);
                    }
                    (Some(a), Some(b), true)
                }
                MatchState::Bye { advanced } => (advanced, None, false),
                _ => (None, None, false),
            };
            let _ = played_it;
            for q in [x, y].into_iter().flatten() {
                assert!(
                    appearances.insert((q, m.id.side, m.id.round)),
                    "{q:?} aparece duas vezes em {:?}",
                    m.id
                );
            }
        }
        for (q, t) in &tally {
            assert!(t.matches <= b.worst_case_matches(), "{q:?}");
            assert!(t.games <= b.worst_case_games(), "{q:?}");
            assert!(t.losses <= if double { 2 } else { 1 }, "{q:?}");
        }
    }

    fn arb_config() -> impl Strategy<Value = EliminationConfig> {
        prop_oneof![Just(single(3, 5)), Just(double(true)), Just(double(false)),]
    }

    proptest! {
        #[test]
        fn random_play_always_finishes_with_consistent_standings(
            n in 2u128..=64,
            config in arb_config(),
            choices in proptest::collection::vec(any::<bool>(), 1..200),
            walkovers in proptest::collection::vec(any::<bool>(), 1..50),
        ) {
            let mut b = EliminationBracket::new(config, seeds(n)).unwrap();
            let mut step = 0usize;
            let mut guard = 0;
            loop {
                let next = b.ready_matches().next().copied();
                let Some(m) = next else { break };
                guard += 1;
                prop_assert!(guard < 1000);
                let MatchState::Ready { a, b: other } = m.state else { unreachable!() };
                let winner = if choices[step % choices.len()] { a } else { other };
                let report = if walkovers[step % walkovers.len()] && step.is_multiple_of(3) {
                    MatchReport { match_id: m.id, outcome: Outcome::Walkover { winner } }
                } else {
                    played(&m, winner)
                };
                step += 1;
                b.report(report).unwrap();
            }
            check_invariants(&b);
        }
    }
}
