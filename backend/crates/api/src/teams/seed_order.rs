//! Ordem do seeding: quais times são elegíveis e em que ordem entram na
//! chave. O sorteio em si é do `engine` (`seeding::random_order`,
//! reproduzível pela seed); aqui ficam as regras do site em volta dele.

use std::collections::HashSet;

use engine::{core::ParticipantId, seeding};

use crate::shared::ApiError;

/// Um time, como o seeding o vê.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrant {
    /// `teams.id`: o `ParticipantId` do `engine`.
    pub id: i64,
    pub seed: Option<i32>,
    /// Todos os jogadores do time fizeram check-in (no duelo, o único).
    pub checked_in: bool,
}

/// Quem entra no seeding e na chave: os times com o check-in de todos os
/// jogadores feito ou, se o torneio não tem check-in, todos. Mantém a ordem
/// de `entrants`.
pub fn eligible(entrants: Vec<Entrant>, has_check_in: bool) -> Vec<Entrant> {
    entrants
        .into_iter()
        .filter(|e| e.checked_in || !has_check_in)
        .collect()
}

/// A ordem que vale ao iniciar: primeiro os que têm seed, pelo seed; depois
/// os sem, na ordem em que vieram (a de criação; no duelo, a de inscrição).
pub fn current_order(mut eligible: Vec<Entrant>) -> Vec<Entrant> {
    eligible.sort_by_key(|e| e.seed.map_or((1, 0), |seed| (0, seed)));
    eligible
}

/// A ordem manual: `team_ids` precisa ter cada elegível exatamente uma vez.
/// 422 `seeding.order_invalid` se não tem (a lista mudou, por exemplo com um
/// check-in no meio do caminho).
pub fn manual_order(eligible: Vec<Entrant>, team_ids: &[i64]) -> Result<Vec<Entrant>, ApiError> {
    let distinct: HashSet<i64> = team_ids.iter().copied().collect();
    let invalid = || ApiError::invalid("seeding.order_invalid");
    if distinct.len() != team_ids.len() || team_ids.len() != eligible.len() {
        return Err(invalid());
    }
    let mut by_team: Vec<Option<Entrant>> = eligible.into_iter().map(Some).collect();
    team_ids
        .iter()
        .map(|&team| {
            by_team
                .iter_mut()
                .find(|e| e.as_ref().is_some_and(|e| e.id == team))
                .and_then(Option::take)
                .ok_or_else(invalid)
        })
        .collect()
}

/// A ordem sorteada com `seed`: a mesma seed e os mesmos elegíveis dão a
/// mesma ordem.
pub fn drawn_order(eligible: Vec<Entrant>, seed: u64) -> Vec<Entrant> {
    let ids: Vec<ParticipantId> = eligible.iter().map(|e| ParticipantId(e.id)).collect();
    let mut rest: Vec<Option<Entrant>> = eligible.into_iter().map(Some).collect();
    seeding::random_order(&ids, seed)
        .into_iter()
        .filter_map(|ParticipantId(id)| {
            rest.iter_mut()
                .find(|e| e.as_ref().is_some_and(|e| e.id == id))
                .and_then(Option::take)
        })
        .collect()
}

/// Os ids dos times, na ordem, para gravar como seeds 1..N.
pub fn ids(order: &[Entrant]) -> Vec<i64> {
    order.iter().map(|e| e.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Time `n` com o seed dado.
    fn entrant(n: i64, seed: Option<i32>, checked_in: bool) -> Entrant {
        Entrant {
            id: n,
            seed,
            checked_in,
        }
    }

    #[test]
    fn eligibility_depends_on_check_in() {
        let all = vec![
            entrant(1, None, true),
            entrant(2, None, false),
            entrant(3, None, true),
        ];
        assert_eq!(ids(&eligible(all.clone(), true)), [1, 3]);
        assert_eq!(ids(&eligible(all, false)), [1, 2, 3]);
    }

    #[test]
    fn seeded_first_then_registration_order() {
        let order = current_order(vec![
            entrant(1, None, true),
            entrant(2, Some(2), true),
            entrant(3, None, true),
            entrant(4, Some(1), true),
        ]);
        assert_eq!(ids(&order), [4, 2, 1, 3]);
    }

    #[test]
    fn manual_order_needs_every_eligible_once() {
        let all = || vec![entrant(1, None, true), entrant(2, None, true)];
        assert_eq!(ids(&manual_order(all(), &[2, 1]).unwrap()), [2, 1]);
        for bad in [&[1][..], &[1, 1], &[1, 3], &[2, 1, 3]] {
            let err = manual_order(all(), bad).unwrap_err();
            assert_eq!(err.code, "seeding.order_invalid", "{bad:?}");
        }
    }

    #[test]
    fn draw_is_reproducible() {
        let all = || (1..=8).map(|n| entrant(n, None, true)).collect::<Vec<_>>();
        let drawn = drawn_order(all(), 99);
        assert_eq!(drawn, drawn_order(all(), 99));
        let mut sorted = ids(&drawn);
        sorted.sort();
        assert_eq!(sorted, (1..=8).collect::<Vec<_>>());
    }
}
