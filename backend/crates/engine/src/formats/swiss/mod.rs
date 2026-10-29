//! Suíço clássico, numa fase única: número fixo de rodadas, todos jogam
//! todas, pareamento por pontos sem revanche; a classificação final sai da
//! tabela.
//!
//! Regras:
//! - Rodadas: `config.rounds` ou, sem número, ⌈log2 N⌉; nunca mais que
//!   N − 1 (`SwissConfig::total_rounds`, com os N do início).
//! - A rodada seguinte nasce quando todas as séries das rodadas já pareadas
//!   estão decididas: `next_round` devolve o pareamento (regras em
//!   `pairing`), e a API grava. O `replay` usa os pareamentos gravados:
//!   corrigir um resultado muda a tabela, nunca as rodadas já pareadas.
//! - Bye (número ímpar de ativos): vitória sem mapas, para o pior colocado
//!   entre os que têm menos byes. É uma partida `Bye`, a última da rodada.
//! - Quem está fora (desistiu ou saiu, `out`) não é pareado nem recebe bye;
//!   a série dele que estiver pronta a API resolve por W.O., como na
//!   eliminação. Com menos de 2 ativos não há próxima rodada, e a fase
//!   termina com as rodadas que teve.
//! - Em cada partida, o lado `a` é o melhor colocado na tabela do pareamento.
//! - Uma série (melhor de N, ímpar) para todas as partidas.
//! - Classificação em `formats::table`: pontos, Buchholz, confronto direto,
//!   saldo de mapas, saldo de frags, seed. O pareamento segue a mesma ordem.
//! - Termina com as rodadas todas pareadas e decididas; só então há
//!   colocação (`placements`).

mod matching;
mod pairing;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::table::{self, League, TableRow, pairing as ready, rest};
use super::{check_count, check_participants};
use crate::core::{
    BracketSide, EngineError, Match, MatchId, Occupant, ParticipantId, Placement, StageReport,
};
use crate::series::validate_best_of;

/// Teto de participantes: o pareamento confere, a cada par, se o resto
/// ainda pareia (emparelhamento máximo, O(n²) por par na prática).
pub const MAX_PARTICIPANTS: usize = 128;

/// Teto de rodadas configuradas; na fase, vale no máximo N − 1.
pub const MAX_ROUNDS: u16 = MAX_PARTICIPANTS as u16 - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwissConfig {
    /// Melhor de N de todas as partidas.
    pub best_of: u8,
    /// Número de rodadas; `None` é ⌈log2 N⌉ com os N do início.
    pub rounds: Option<u16>,
}

impl SwissConfig {
    /// Erros: melhor de N inválido, rodadas fora de 1 a `MAX_ROUNDS`.
    pub fn validate(&self) -> Result<(), EngineError> {
        validate_best_of(self.best_of)?;
        match self.rounds {
            Some(got) if !(1..=MAX_ROUNDS).contains(&got) => Err(EngineError::InvalidRounds {
                max: MAX_ROUNDS,
                got,
            }),
            _ => Ok(()),
        }
    }

    /// Rodadas da fase com `participants` no início: as configuradas (ou
    /// ⌈log2 N⌉), no máximo N − 1. Não diminui com mais participantes.
    pub fn total_rounds(&self, participants: usize) -> u16 {
        if participants < 2 {
            return 0;
        }
        let auto = participants.next_power_of_two().trailing_zeros() as usize;
        let wanted = self.rounds.map_or(auto, usize::from);
        wanted.min(participants - 1).min(usize::from(u16::MAX)) as u16
    }

    /// Máximo de mapas de um participante com `participants`: uma série por
    /// rodada. Erros: poucos ou participantes demais, os de `validate`.
    pub fn worst_case_games(&self, participants: usize) -> Result<u32, EngineError> {
        check_count(participants, MAX_PARTICIPANTS)?;
        self.validate()?;
        Ok(u32::from(self.total_rounds(participants)) * u32::from(self.best_of))
    }
}

/// O pareamento de uma rodada, como a API grava: os pares na ordem das
/// partidas (`sw-<rodada>-1` é o primeiro; em cada par, o primeiro é o lado
/// `a`) e quem tem o bye (a partida seguinte ao último par).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwissRound {
    pub pairs: Vec<[ParticipantId; 2]>,
    pub bye: Option<ParticipantId>,
}

/// Só leitura de fora: para reidratar, `replay` com configuração,
/// participantes e histórico (pareamentos e resultados), que revalida tudo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwissBracket {
    config: SwissConfig,
    total_rounds: u16,
    rounds: Vec<SwissRound>,
    /// Fora do torneio, na ordem dos seeds.
    out: Vec<ParticipantId>,
    league: League,
}

impl SwissBracket {
    /// A fase sem nenhuma rodada pareada: a primeira sai de `next_round`.
    /// `seeded` está em ordem de seed: o primeiro é o seed 1.
    ///
    /// Erros: menos de 2 ou mais de `MAX_PARTICIPANTS` participantes,
    /// participante repetido, os de `SwissConfig::validate`.
    pub fn new(config: SwissConfig, seeded: Vec<ParticipantId>) -> Result<Self, EngineError> {
        Self::replay(config, seeded, Vec::new(), [], &[])
    }

    /// Reconstrói o estado com os pareamentos gravados (`rounds`, da rodada
    /// 1 em diante), os resultados em ordem e quem está fora (ids que não
    /// são participantes são ignorados). Corrigir um resultado é chamar
    /// `replay` com o histórico corrigido: a tabela muda, os pareamentos não.
    ///
    /// Erros: os de `new`; `InvalidPairing` para rodada além do total, sem
    /// par, ou com participante desconhecido ou repetido; os de cada
    /// resultado (ver `RoundRobinBracket::report`).
    pub fn replay(
        config: SwissConfig,
        seeded: Vec<ParticipantId>,
        rounds: Vec<SwissRound>,
        reports: impl IntoIterator<Item = StageReport>,
        out: &[ParticipantId],
    ) -> Result<Self, EngineError> {
        check_participants(&seeded, MAX_PARTICIPANTS)?;
        config.validate()?;
        let total_rounds = config.total_rounds(seeded.len());
        let matches = build(&seeded, &rounds, total_rounds, config.best_of)?;
        let out = seeded.iter().filter(|p| out.contains(p)).copied().collect();
        let mut league = League::new(table::SWISS, seeded, matches);
        for report in reports {
            league.apply(report)?;
        }
        league.refresh();
        Ok(Self {
            config,
            total_rounds,
            rounds,
            out,
            league,
        })
    }

    pub fn config(&self) -> SwissConfig {
        self.config
    }

    pub fn participants(&self) -> &[ParticipantId] {
        self.league.participants()
    }

    /// As partidas das rodadas já pareadas, por rodada e índice.
    pub fn matches(&self) -> &[Match] {
        self.league.matches()
    }

    pub fn get(&self, id: MatchId) -> Option<&Match> {
        self.league.get(id)
    }

    pub fn occupants(&self, id: MatchId) -> Option<(Occupant, Occupant)> {
        self.league.occupants(id)
    }

    /// Rodadas da fase (`SwissConfig::total_rounds` com os N do início).
    pub fn total_rounds(&self) -> u16 {
        self.total_rounds
    }

    /// Os pareamentos gravados.
    pub fn rounds(&self) -> &[SwissRound] {
        &self.rounds
    }

    /// Quem está fora, na ordem dos seeds.
    pub fn out(&self) -> &[ParticipantId] {
        &self.out
    }

    /// Registra um resultado (ver `RoundRobinBracket::report`).
    pub fn report(&mut self, report: StageReport) -> Result<(), EngineError> {
        self.league.report(report)
    }

    fn active_count(&self) -> usize {
        self.participants().len() - self.out.len()
    }

    /// O pareamento da próxima rodada, quando é hora: todas as séries das
    /// rodadas pareadas decididas, rodada por parear e pelo menos 2 ativos.
    /// Determinístico: os mesmos dados dão sempre o mesmo pareamento.
    pub fn next_round(&self) -> Option<SwissRound> {
        if !self.league.all_decided()
            || self.rounds.len() >= usize::from(self.total_rounds)
            || self.active_count() < 2
        {
            return None;
        }
        let active: Vec<&TableRow> = self
            .league
            .table()
            .iter()
            .filter(|r| !self.out.contains(&r.participant))
            .collect();
        let index: HashMap<ParticipantId, usize> = active
            .iter()
            .enumerate()
            .map(|(i, r)| (r.participant, i))
            .collect();
        let k = active.len();
        let mut fresh: Vec<Vec<bool>> = (0..k).map(|i| (0..k).map(|j| i != j).collect()).collect();
        for [a, b] in self.rounds.iter().flat_map(|r| &r.pairs) {
            if let (Some(&i), Some(&j)) = (index.get(a), index.get(b)) {
                fresh[i][j] = false;
                fresh[j][i] = false;
            }
        }
        let points: Vec<u16> = active.iter().map(|r| r.points).collect();
        let byes: Vec<u16> = active.iter().map(|r| r.byes).collect();
        let pairing = pairing::pair(&points, &byes, &fresh);
        Some(SwissRound {
            pairs: pairing
                .pairs
                .iter()
                .map(|&(a, b)| [active[a].participant, active[b].participant])
                .collect(),
            bye: pairing.bye.map(|i| active[i].participant),
        })
    }

    pub fn is_finished(&self) -> bool {
        self.league.all_decided()
            && (self.rounds.len() >= usize::from(self.total_rounds) || self.active_count() < 2)
    }

    /// A tabela atual, em ordem de posição (é a ordem do pareamento).
    pub fn table(&self) -> &[TableRow] {
        self.league.table()
    }

    /// A colocação final, quando termina; antes, vazia.
    pub fn placements(&self) -> Vec<Placement> {
        if self.is_finished() {
            self.league.placements()
        } else {
            Vec::new()
        }
    }

    /// Máximo de mapas de um participante: uma série por rodada.
    pub fn worst_case_games(&self) -> u32 {
        u32::from(self.total_rounds) * u32::from(self.config.best_of)
    }
}

/// As partidas dos pareamentos gravados, conferidos.
fn build(
    seeded: &[ParticipantId],
    rounds: &[SwissRound],
    total: u16,
    best_of: u8,
) -> Result<Vec<Match>, EngineError> {
    if rounds.len() > usize::from(total) {
        return Err(EngineError::InvalidPairing { round: total + 1 });
    }
    let known: HashSet<ParticipantId> = seeded.iter().copied().collect();
    let mut out = Vec::new();
    for (r, round) in rounds.iter().enumerate() {
        let number = r as u16 + 1;
        let invalid = EngineError::InvalidPairing { round: number };
        if round.pairs.is_empty() {
            return Err(invalid);
        }
        let mut seen = HashSet::new();
        for p in round.pairs.iter().flatten().chain(&round.bye) {
            if !known.contains(p) || !seen.insert(*p) {
                return Err(invalid);
            }
        }
        let id = |index: usize| MatchId {
            side: BracketSide::Swiss,
            round: number,
            index: index as u16,
        };
        for (i, [a, b]) in round.pairs.iter().enumerate() {
            out.push(ready(id(i + 1), *a, *b, best_of));
        }
        if let Some(p) = round.bye {
            out.push(rest(id(round.pairs.len() + 1), p, best_of));
        }
    }
    Ok(out)
}
