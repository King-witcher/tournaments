//! Fases ligadas: o que a configuração de cada fase diz além do formato (o
//! corte e o check-in), quem entra na fase seguinte e a classificação final
//! de um torneio em fases. A API diz quem está disponível (não desistiu e,
//! com check-in, confirmou presença) e grava quem entrou, com o seed.
//!
//! - **Quem entra** (`entry`): na ordem da classificação final da fase
//!   anterior, os disponíveis até o corte; os disponíveis abaixo dele ficam
//!   cortados, e os indisponíveis, ausentes. Os seeds da fase são essa ordem.
//! - **Classificação final** (`final_standings`): os da última fase pela
//!   colocação dela; depois, de trás para a frente, os cortados de cada fase
//!   e, por último dela, os ausentes, cada um numa posição, na ordem da
//!   classificação daquela fase.

use crate::core::{ParticipantId, Placement};
use crate::formats::StageFormat;

/// Corte mínimo: a fase seguinte precisa de dois lados.
pub const MIN_ADVANCE: u16 = 2;

/// Uma fase como a configuração do torneio a define: o formato e como os
/// participantes entram nela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageSetup {
    pub format: StageFormat,
    /// Fases 2 em diante: quantos da anterior entram, de `MIN_ADVANCE` ao
    /// teto do formato; `None`, todos. Na fase 1, sempre `None`.
    pub advance: Option<u16>,
    /// Fases 2 em diante: só entra quem confirmar presença. Na fase 1,
    /// sempre falso (o check-in é o do torneio).
    pub check_in: bool,
}

impl StageSetup {
    /// A fase com `format`, sem corte e sem check-in.
    pub const fn of(format: StageFormat) -> Self {
        Self {
            format,
            advance: None,
            check_in: false,
        }
    }
}

/// Um candidato a entrar na fase seguinte: quem jogou a anterior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub participant: ParticipantId,
    /// Não desistiu e, se a fase pede check-in, confirmou presença.
    pub available: bool,
}

/// Quem entra numa fase e quem fica de fora, cada lista na ordem da
/// classificação final da fase anterior.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// Quem entra, na ordem dos seeds da fase (1..K).
    pub entrants: Vec<ParticipantId>,
    /// Disponíveis abaixo do corte.
    pub cut: Vec<ParticipantId>,
    /// Desistiram ou não confirmaram presença.
    pub absent: Vec<ParticipantId>,
}

/// Quem entra na fase com corte `advance` (`None`: todos), dos candidatos
/// `ranking`, na ordem da classificação final da fase anterior: os
/// disponíveis, até o corte. Com menos disponíveis que o corte, entram
/// todos; quem chama confere o mínimo de dois.
pub fn entry(ranking: &[Candidate], advance: Option<u16>) -> Entry {
    let limit = advance.map_or(usize::MAX, usize::from);
    let mut out = Entry::default();
    for c in ranking {
        if !c.available {
            out.absent.push(c.participant);
        } else if out.entrants.len() < limit {
            out.entrants.push(c.participant);
        } else {
            out.cut.push(c.participant);
        }
    }
    out
}

/// A classificação final do torneio: `last`, as colocações da última fase
/// (com as faixas de empate da eliminação), e, para as anteriores, `earlier`,
/// quem entrou em cada fase da segunda em diante, na ordem das fases. Depois
/// dos que entraram na última, de trás para a frente, os cortados e então os
/// ausentes de cada fase, uma posição cada, depois das de `last` (na ordem em
/// que vieram). Com uma fase só (`earlier` vazio), é `last`.
pub fn final_standings(last: &[Placement], earlier: &[Entry]) -> Vec<Placement> {
    let mut out = last.to_vec();
    let Some(current) = earlier.last() else {
        return out;
    };
    let mut rank = u16::try_from(current.entrants.len()).unwrap_or(u16::MAX);
    for e in earlier.iter().rev() {
        for &participant in e.cut.iter().chain(&e.absent) {
            rank = rank.saturating_add(1);
            out.push(Placement {
                participant,
                rank,
                tied: 1,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn p(n: i64) -> ParticipantId {
        ParticipantId(n)
    }

    fn ranking(available: &[bool]) -> Vec<Candidate> {
        (1..)
            .zip(available)
            .map(|(n, &available)| Candidate {
                participant: p(n),
                available,
            })
            .collect()
    }

    fn ids(list: &[ParticipantId]) -> Vec<i64> {
        list.iter().map(|p| p.0).collect()
    }

    #[test]
    fn everyone_enters_without_a_cut() {
        let e = entry(&ranking(&[true; 5]), None);
        assert_eq!(ids(&e.entrants), [1, 2, 3, 4, 5]);
        assert!(e.cut.is_empty() && e.absent.is_empty());
    }

    #[test]
    fn the_cut_keeps_the_best_and_the_absent_are_skipped() {
        // O 2º não confirmou: o 5º sobe.
        let e = entry(&ranking(&[true, false, true, true, true, true]), Some(4));
        assert_eq!(ids(&e.entrants), [1, 3, 4, 5]);
        assert_eq!(ids(&e.cut), [6]);
        assert_eq!(ids(&e.absent), [2]);
    }

    #[test]
    fn a_cut_above_the_candidates_takes_everyone() {
        let e = entry(&ranking(&[true, true, false]), Some(8));
        assert_eq!(ids(&e.entrants), [1, 2]);
        assert_eq!(ids(&e.absent), [3]);
    }

    #[test]
    fn one_stage_is_its_own_standings() {
        let last = [
            Placement {
                participant: p(2),
                rank: 1,
                tied: 1,
            },
            Placement {
                participant: p(1),
                rank: 2,
                tied: 1,
            },
        ];
        assert_eq!(final_standings(&last, &[]), last);
    }

    #[test]
    fn earlier_stages_come_back_to_front_cut_before_absent() {
        // Fase 1 com 8: entram 4 na fase 2 (o 3º desistiu, o 7º e o 8º
        // ficam cortados); da fase 2, entram 2 na 3 (o 4º da fase 2 não
        // confirmou, o 3º ficou cortado).
        let one = entry(
            &ranking(&[true, true, false, true, true, true, true, true]),
            Some(4),
        );
        assert_eq!(ids(&one.entrants), [1, 2, 4, 5]);
        let two = entry(
            &[
                Candidate {
                    participant: p(4),
                    available: true,
                },
                Candidate {
                    participant: p(1),
                    available: true,
                },
                Candidate {
                    participant: p(2),
                    available: true,
                },
                Candidate {
                    participant: p(5),
                    available: false,
                },
            ],
            Some(2),
        );
        let last = [
            Placement {
                participant: p(1),
                rank: 1,
                tied: 1,
            },
            Placement {
                participant: p(4),
                rank: 2,
                tied: 1,
            },
        ];
        let standings = final_standings(&last, &[one, two]);
        let order: Vec<(i64, u16)> = standings
            .iter()
            .map(|s| (s.participant.0, s.rank))
            .collect();
        assert_eq!(
            order,
            [
                (1, 1),
                (4, 2),
                (2, 3),
                (5, 4),
                (6, 5),
                (7, 6),
                (8, 7),
                (3, 8)
            ]
        );
    }

    #[test]
    fn the_last_stage_may_still_be_open() {
        // Na eliminação em andamento, só os eliminados têm colocação; os das
        // fases anteriores já ficam atrás de todos os que entraram nela.
        let one = entry(&ranking(&[true; 6]), Some(4));
        let last = [Placement {
            participant: p(4),
            rank: 3,
            tied: 2,
        }];
        let standings = final_standings(&last, &[one]);
        let order: Vec<(i64, u16)> = standings
            .iter()
            .map(|s| (s.participant.0, s.rank))
            .collect();
        assert_eq!(order, [(4, 3), (5, 5), (6, 6)]);
    }

    proptest! {
        /// Quem entra são os primeiros disponíveis, no máximo o corte; ninguém
        /// some nem se repete, e a ordem de cada lista é a da classificação.
        #[test]
        fn entry_splits_the_ranking(
            available in proptest::collection::vec(any::<bool>(), 0..40),
            advance in proptest::option::of(2u16..40),
        ) {
            let ranking = ranking(&available);
            let e = entry(&ranking, advance);
            let free = available.iter().filter(|a| **a).count();
            let expected = advance.map_or(free, |a| free.min(usize::from(a)));
            prop_assert_eq!(e.entrants.len(), expected);
            prop_assert_eq!(e.entrants.len() + e.cut.len(), free);
            prop_assert_eq!(e.absent.len(), available.len() - free);
            for list in [&e.entrants, &e.cut, &e.absent] {
                prop_assert!(list.windows(2).all(|w| w[0].0 < w[1].0));
            }
            // Nenhum cortado está à frente de quem entrou.
            if let (Some(last_in), Some(first_cut)) = (e.entrants.last(), e.cut.first()) {
                prop_assert!(last_in.0 < first_cut.0);
            }
        }

        /// A classificação final tem todos os candidatos da fase 1, cada um
        /// uma vez, e as posições depois da última fase seguem sem buraco.
        #[test]
        fn final_standings_place_everyone_once(
            available in proptest::collection::vec(any::<bool>(), 2..30),
            advance in proptest::option::of(2u16..30),
        ) {
            let one = entry(&ranking(&available), advance);
            let last: Vec<Placement> = (1..)
                .zip(&one.entrants)
                .map(|(rank, &participant)| Placement { participant, rank, tied: 1 })
                .collect();
            let standings = final_standings(&last, std::slice::from_ref(&one));
            prop_assert_eq!(standings.len(), available.len());
            let mut seen: Vec<i64> = standings.iter().map(|s| s.participant.0).collect();
            seen.sort_unstable();
            seen.dedup();
            prop_assert_eq!(seen.len(), available.len());
            let ranks: Vec<u16> = standings.iter().map(|s| s.rank).collect();
            let expected: Vec<u16> = (1..=available.len() as u16).collect();
            prop_assert_eq!(ranks, expected);
        }
    }
}
