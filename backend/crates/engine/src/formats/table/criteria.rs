//! Os critérios da tabela e a ordem deles numa fase: o primeiro, fixo pelo
//! formato; os desempates, que a fase escolhe (`Tiebreakers`); e o seed,
//! sempre o último, para a posição nunca empatar.

use serde::{Deserialize, Serialize};

use crate::core::EngineError;

/// Critério da ordem da tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criterion {
    SeriesDiff,
    Points,
    Buchholz,
    HeadToHead,
    MapDiff,
    FragDiff,
    /// O rating menor fica à frente: o do início da fase, que a API congela
    /// por participante e passa no replay.
    LowerRating,
    Seed,
}

impl Criterion {
    /// Pode ser escolhido como desempate: o confronto direto, os saldos de
    /// mapas e de frags, o menor rating e o Buchholz (só no suíço). O
    /// primeiro critério de cada formato e o seed são fixos.
    pub const fn is_tiebreaker(self) -> bool {
        matches!(
            self,
            Self::Buchholz | Self::HeadToHead | Self::MapDiff | Self::FragDiff | Self::LowerRating
        )
    }
}

/// Os desempates de uma fase de tabela, na ordem em que valem: entre o
/// primeiro critério do formato (saldo de séries no round robin, pontos no
/// suíço) e o seed. Cada um no máximo uma vez; pode ser vazio, e então
/// decide o seed. Tamanho fixo, para o formato continuar `Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tiebreakers {
    len: u8,
    /// Valem os `len` primeiros; o resto é sempre `Seed`, só para preencher
    /// (a igualdade compara o array inteiro).
    list: [Criterion; Tiebreakers::MAX],
}

impl Tiebreakers {
    /// Os cinco desempates que existem, cada um uma vez.
    pub const MAX: usize = 5;

    /// Nenhum desempate: depois do primeiro critério, o seed.
    pub const NONE: Self = Self {
        len: 0,
        list: [Criterion::Seed; Self::MAX],
    };

    /// O padrão do round robin: confronto direto, saldo de mapas, saldo de
    /// frags.
    pub const ROUND_ROBIN: Self = Self {
        len: 3,
        list: [
            Criterion::HeadToHead,
            Criterion::MapDiff,
            Criterion::FragDiff,
            Criterion::Seed,
            Criterion::Seed,
        ],
    };

    /// O padrão do suíço: Buchholz, confronto direto, saldo de mapas, saldo de
    /// frags.
    pub const SWISS: Self = Self {
        len: 4,
        list: [
            Criterion::Buchholz,
            Criterion::HeadToHead,
            Criterion::MapDiff,
            Criterion::FragDiff,
            Criterion::Seed,
        ],
    };

    /// `list` na ordem em que vale. Erros: `InvalidTiebreakers` com critério
    /// repetido ou que não desempata (o saldo de séries, os pontos e o seed
    /// são fixos). O Buchholz fora do suíço é recusado na configuração do
    /// formato (`RoundRobinConfig::validate`).
    pub fn new(list: &[Criterion]) -> Result<Self, EngineError> {
        let mut out = Self::NONE;
        for &c in list {
            // São cinco desempates, então o sexto sempre cai aqui: o índice
            // abaixo nunca passa de `MAX`.
            if !c.is_tiebreaker() || out.contains(c) {
                return Err(EngineError::InvalidTiebreakers);
            }
            out.list[usize::from(out.len)] = c;
            out.len += 1;
        }
        Ok(out)
    }

    /// Os desempates, em ordem.
    pub fn as_slice(&self) -> &[Criterion] {
        &self.list[..usize::from(self.len)]
    }

    pub fn contains(&self, criterion: Criterion) -> bool {
        self.as_slice().contains(&criterion)
    }

    /// Só o suíço tem Buchholz (`swiss`: a fase é suíço). Erro:
    /// `InvalidTiebreakers`.
    pub(crate) fn check(&self, swiss: bool) -> Result<(), EngineError> {
        if !swiss && self.contains(Criterion::Buchholz) {
            return Err(EngineError::InvalidTiebreakers);
        }
        Ok(())
    }
}

/// Como um formato de tabela pontua e desempata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rules {
    /// O bye vale vitória (suíço) ou é só folga (round robin).
    pub bye_points: bool,
    /// O primeiro critério, fixo: saldo de séries (round robin) ou pontos
    /// (suíço).
    pub first: Criterion,
    /// Os desempates da fase; depois deles, sempre o seed.
    pub tiebreakers: Tiebreakers,
    /// A mini-tabela do confronto direto conta o saldo (round robin) ou só as
    /// vitórias (suíço). Sem revanche e sem empate, dão a mesma ordem; com
    /// revanche, não, e o suíço fica como sempre foi.
    pub mini_table_balance: bool,
}

impl Rules {
    /// Round robin: saldo de séries, os desempates, seed.
    pub const fn round_robin(tiebreakers: Tiebreakers) -> Self {
        Self {
            bye_points: false,
            first: Criterion::SeriesDiff,
            tiebreakers,
            mini_table_balance: true,
        }
    }

    /// Suíço: pontos, os desempates, seed.
    pub const fn swiss(tiebreakers: Tiebreakers) -> Self {
        Self {
            bye_points: true,
            first: Criterion::Points,
            tiebreakers,
            mini_table_balance: false,
        }
    }

    /// Todos os critérios, em ordem: o primeiro, os desempates e o seed.
    pub fn criteria(&self) -> Vec<Criterion> {
        std::iter::once(self.first)
            .chain(self.tiebreakers.as_slice().iter().copied())
            .chain(std::iter::once(Criterion::Seed))
            .collect()
    }
}
