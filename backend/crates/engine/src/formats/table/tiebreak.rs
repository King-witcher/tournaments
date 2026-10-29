//! A ordem da tabela pelos critérios, um grupo de empatados por vez.

use std::cmp::Reverse;

use super::Criterion;

/// Ordena os participantes `0..n` (índices na ordem dos seeds) pelos
/// `criteria`, nesta ordem; o último precisa separar todos (o seed).
/// `value` dá o valor de cada critério (maior é melhor), menos o do confronto
/// direto, que sai de `results` (vencedor, perdedor) dentro de cada grupo e
/// só vale quando todos do grupo já se enfrentaram. `usable` diz se um
/// critério vale para um grupo (senão, o seguinte). Devolve a ordem e, para
/// cada um, o critério que o pôs à frente do seguinte (`None` no último).
pub(super) fn order(
    n: usize,
    criteria: &[Criterion],
    value: impl Fn(Criterion, usize) -> i64,
    usable: impl Fn(Criterion, &[usize]) -> bool,
    results: &[(usize, usize)],
) -> Vec<(usize, Option<Criterion>)> {
    let ranker = Ranker {
        criteria,
        value: &value,
        usable: &usable,
        results,
    };
    let all: Vec<usize> = (0..n).collect();
    ranker.rank(&all, 0)
}

struct Ranker<'a> {
    criteria: &'a [Criterion],
    value: &'a dyn Fn(Criterion, usize) -> i64,
    usable: &'a dyn Fn(Criterion, &[usize]) -> bool,
    results: &'a [(usize, usize)],
}

impl Ranker<'_> {
    /// Séries que `i` venceu contra os outros de `group`: a mini-tabela.
    fn head_to_head(&self, group: &[usize], i: usize) -> i64 {
        self.results
            .iter()
            .filter(|(w, l)| *w == i && group.contains(l))
            .count() as i64
    }

    /// Todo par de `group` tem série decidida entre si.
    fn all_met(&self, group: &[usize]) -> bool {
        group.iter().enumerate().all(|(k, &x)| {
            group[k + 1..].iter().all(|&y| {
                self.results
                    .iter()
                    .any(|&(w, l)| (w, l) == (x, y) || (w, l) == (y, x))
            })
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
        // A mini-tabela é refeita entre os que continuam empatados nela; os
        // outros critérios seguem para o próximo.
        let next = if criterion == Criterion::HeadToHead {
            from
        } else {
            from + 1
        };
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
