//! O empate que sobra no fim de uma fase de tabela: o grupo que, depois de
//! todos os critérios da fase, só a ordem da chave (seed) separa.
//!
//! - **Ordem da chave** (`FinalTie::Seed`, o padrão): o seed decide, como
//!   sempre.
//! - **A organização decide** (`FinalTie::Organizer`): só no fim (todas as
//!   séries decididas), cada grupo assim é um empate pendente (`PendingTie`),
//!   a não ser que haja uma decisão (`TieOrder`) para exatamente aquele
//!   conjunto de participantes, que então dá a ordem do grupo
//!   (`Criterion::Organizer` na explicação). A decisão vale pelo conjunto: se
//!   uma correção muda quem está empatado, ela deixa de valer e o grupo volta
//!   a ser pendente; com o mesmo conjunto, continua valendo. Sem decisão, o
//!   grupo fica provisoriamente na ordem do seed, e a fase não termina.
//!
//! Durante a fase nada muda: a tabela ao vivo e o pareamento do suíço
//! desempatam pelo seed.

use serde::Serialize;

use super::{Criterion, TableRow};
use crate::core::ParticipantId;

/// Quem decide o empate que sobra depois de todos os critérios.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FinalTie {
    /// A ordem da chave (seed).
    #[default]
    Seed,
    /// A organização, no fim da fase.
    Organizer,
}

/// A ordem que a organização decidiu para um grupo empatado: os
/// participantes do grupo, do melhor ao pior.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TieOrder {
    pub participants: Vec<ParticipantId>,
}

/// Um grupo empatado que espera a decisão da organização.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PendingTie {
    /// A melhor posição do grupo, que os empatados dividem.
    pub position: u16,
    /// Os empatados, na ordem do seed (a provisória da tabela).
    pub participants: Vec<ParticipantId>,
}

impl TieOrder {
    /// A decisão é para o grupo `group`: os mesmos participantes, cada um uma
    /// vez (é o que faz ela valer no fim da fase).
    pub fn is_for(&self, group: &[ParticipantId]) -> bool {
        same_set(group, &self.participants)
    }
}

impl PendingTie {
    /// `order` tem exatamente os participantes do grupo, cada um uma vez.
    pub fn is_order(&self, order: &[ParticipantId]) -> bool {
        same_set(&self.participants, order)
    }
}

fn same_set(group: &[ParticipantId], order: &[ParticipantId]) -> bool {
    order.len() == group.len()
        && group.iter().all(|p| order.contains(p))
        && order.iter().all(|p| group.contains(p))
}

/// Os grupos (índices em `table`, em ordem) que só o seed separa: cada linha,
/// menos a última do grupo, está à frente da seguinte pelo seed. O seed só
/// separa singletons dentro do grupo que chegou a ele, então a última linha
/// do grupo tem o critério que o separou do seguinte.
fn seed_groups(table: &[TableRow]) -> Vec<std::ops::Range<usize>> {
    let mut groups = Vec::new();
    let mut start = 0;
    for (i, row) in table.iter().enumerate() {
        if row.ahead_by == Some(Criterion::Seed) {
            continue;
        }
        if i > start {
            groups.push(start..i + 1);
        }
        start = i + 1;
    }
    groups
}

/// No fim da fase: aplica a `table` as decisões de `orders` que batem com um
/// grupo empatado e devolve os grupos que esperam decisão. Com
/// `FinalTie::Seed`, nada muda e nada fica pendente. Se mais de uma decisão
/// bate com o grupo, vale a última.
pub(super) fn settle(
    table: &mut [TableRow],
    final_tie: FinalTie,
    orders: &[TieOrder],
) -> Vec<PendingTie> {
    if final_tie == FinalTie::Seed {
        return Vec::new();
    }
    let mut pending = Vec::new();
    for group in seed_groups(table) {
        let rows = &mut table[group];
        let participants: Vec<ParticipantId> = rows.iter().map(|r| r.participant).collect();
        let decided = orders.iter().rev().find(|o| o.is_for(&participants));
        let Some(order) = decided else {
            pending.push(PendingTie {
                position: rows[0].position,
                participants,
            });
            continue;
        };
        let first = rows[0].position;
        let last_ahead_by = rows[rows.len() - 1].ahead_by;
        rows.sort_by_key(|r| order.participants.iter().position(|p| *p == r.participant));
        let n = rows.len();
        for (k, row) in rows.iter_mut().enumerate() {
            row.position = first + k as u16;
            row.ahead_by = if k + 1 < n {
                Some(Criterion::Organizer)
            } else {
                last_ahead_by
            };
        }
    }
    pending
}
