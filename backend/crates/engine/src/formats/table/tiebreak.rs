//! A ordem da tabela pelos critérios, um grupo de empatados por vez.

use std::cmp::Reverse;

use super::{Criterion, Rules};

/// Uma série decidida entre dois participantes (índices na ordem dos seeds),
/// para o confronto direto; `winner` nulo é a série empatada, que conta como
/// confronto realizado e vale 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Meeting {
    pub a: usize,
    pub b: usize,
    pub winner: Option<usize>,
}

impl Meeting {
    /// O adversário de `i` nesta série, se `i` a jogou.
    fn against(&self, i: usize) -> Option<usize> {
        if self.a == i {
            Some(self.b)
        } else if self.b == i {
            Some(self.a)
        } else {
            None
        }
    }
}

/// Ordena os participantes `0..n` (índices na ordem dos seeds) pelos critérios
/// de `rules` (`Rules::criteria`: o primeiro, os desempates da fase em
/// qualquer ordem e o seed, que separa todos). `value` dá o valor de cada
/// critério (maior é melhor), menos o do confronto direto, que sai de
/// `meetings` dentro de cada grupo e só vale quando todos do grupo já se
/// enfrentaram. `usable` diz se um critério vale para um grupo (senão, o
/// seguinte). Devolve a ordem e, para cada um, o critério que o pôs à frente
/// do seguinte (`None` no último). Quando um critério depois do confronto
/// direto separa um grupo, quem segue empatado volta ao confronto direto, só
/// entre si; sem ele entre os desempates, não há volta.
pub(super) fn order(
    n: usize,
    rules: Rules,
    value: impl Fn(Criterion, usize) -> i64,
    usable: impl Fn(Criterion, &[usize]) -> bool,
    meetings: &[Meeting],
) -> Vec<(usize, Option<Criterion>)> {
    let criteria = rules.criteria();
    let ranker = Ranker {
        criteria: &criteria,
        head_to_head: criteria.iter().position(|c| *c == Criterion::HeadToHead),
        balance: rules.mini_table_balance,
        value: &value,
        usable: &usable,
        meetings,
    };
    let all: Vec<usize> = (0..n).collect();
    ranker.rank(&all, 0)
}

struct Ranker<'a> {
    /// Termina sempre no seed.
    criteria: &'a [Criterion],
    /// Onde o confronto direto está em `criteria`, se a fase o usa.
    head_to_head: Option<usize>,
    /// `Rules::mini_table_balance`.
    balance: bool,
    value: &'a dyn Fn(Criterion, usize) -> i64,
    usable: &'a dyn Fn(Criterion, &[usize]) -> bool,
    meetings: &'a [Meeting],
}

impl Ranker<'_> {
    /// A mini-tabela: o que `i` fez contra os outros de `group`. No round
    /// robin, o saldo (vitória +1, empate 0, derrota −1); no suíço, as vitórias
    /// (`Rules::mini_table_balance`).
    fn head_to_head(&self, group: &[usize], i: usize) -> i64 {
        self.meetings
            .iter()
            .filter(|m| m.against(i).is_some_and(|o| group.contains(&o)))
            .map(|m| match m.winner {
                Some(w) if w == i => 1,
                Some(_) if self.balance => -1,
                _ => 0,
            })
            .sum()
    }

    /// Todo par de `group` tem série decidida entre si, empatada inclusive.
    fn all_met(&self, group: &[usize]) -> bool {
        group.iter().enumerate().all(|(k, &x)| {
            group[k + 1..]
                .iter()
                .all(|&y| self.meetings.iter().any(|m| m.against(x) == Some(y)))
        })
    }

    fn applies(&self, criterion: Criterion, group: &[usize]) -> bool {
        match criterion {
            Criterion::HeadToHead => self.all_met(group),
            c => (self.usable)(c, group),
        }
    }

    /// Ordena `group` a partir do critério `from`.
    fn rank(&self, group: &[usize], from: usize) -> Vec<(usize, Option<Criterion>)> {
        if group.len() <= 1 {
            return group.iter().map(|&i| (i, None)).collect();
        }
        let criterion = self.criteria[from];
        if criterion != Criterion::Seed && !self.applies(criterion, group) {
            return self.rank(group, from + 1);
        }
        let score = |i: usize| match criterion {
            Criterion::HeadToHead => self.head_to_head(group, i),
            c => (self.value)(c, i),
        };
        let mut scored: Vec<(i64, usize)> = group.iter().map(|&i| (score(i), i)).collect();
        scored.sort_by_key(|&(s, i)| (Reverse(s), i));
        let runs: Vec<Vec<usize>> = scored
            .chunk_by(|x, y| x.0 == y.0)
            .map(|run| run.iter().map(|&(_, i)| i).collect())
            .collect();
        if runs.len() == 1 {
            debug_assert!(criterion != Criterion::Seed, "o seed separa todos");
            return self.rank(group, from + 1);
        }
        // Quem continua empatado volta ao confronto direto, agora só entre si,
        // quando ele já passou na ordem (`h2h <= from`): vale para a
        // mini-tabela refeita e para o grupo que um critério depois dele
        // separou (saldo de mapas, de frags, ou o Buchholz posto depois),
        // antes de seguir. Os critérios antes dele são globais e já iguais
        // dentro do grupo, então não precisam voltar. Sem confronto direto
        // atrás de nós (depois na ordem, ou desligado), segue o próximo.
        let next = self
            .head_to_head
            .filter(|&h2h| h2h <= from)
            .unwrap_or(from + 1);
        let last = runs.len() - 1;
        let mut out = Vec::with_capacity(group.len());
        for (k, run) in runs.iter().enumerate() {
            let mut sub = self.rank(run, next);
            if k < last
                && let Some(tail) = sub.last_mut()
            {
                tail.1 = Some(criterion);
            }
            out.extend(sub);
        }
        out
    }
}
