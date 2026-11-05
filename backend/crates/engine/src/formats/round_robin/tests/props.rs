//! Propriedades com proptest: qualquer sequência de resultados.

use super::*;
use crate::formats::table::{Criterion, Rules};
use proptest::prelude::*;

/// A ordem de antes do saldo de séries: vitórias e, no confronto direto, só as
/// vitórias dentro do grupo.
const BY_WINS: Rules = Rules {
    bye_points: false,
    criteria: &[
        Criterion::Points,
        Criterion::HeadToHead,
        Criterion::MapDiff,
        Criterion::FragDiff,
        Criterion::Seed,
    ],
    mini_table_balance: false,
};

/// O resultado do passo `i`: vencedor, W.O. ou placar, frags.
fn report(m: &Match, i: usize, picks: &[(bool, bool, u8, i32, i32)]) -> StageReport {
    let MatchState::Ready { a, b } = m.state else {
        unreachable!()
    };
    let (first, walkover, loser_maps, fw, fl) = picks[i % picks.len()];
    let winner = if first { a } else { b };
    let outcome = if walkover && i.is_multiple_of(4) {
        Outcome::Walkover { winner }
    } else {
        Outcome::Played {
            winner,
            score: SeriesScore {
                winner: wins_needed(m.best_of),
                loser: loser_maps % wins_needed(m.best_of),
            },
        }
    };
    StageReport {
        report: MatchReport {
            match_id: m.id,
            outcome,
        },
        frags: (fw % 5 != 0).then_some(SeriesFrags { a: fw, b: fl }),
    }
}

fn check_table(b: &RoundRobinBracket) {
    let n = b.participants().len();
    let t = b.table();
    assert_eq!(t.len(), n);
    let positions: Vec<u16> = t.iter().map(|r| r.position).collect();
    assert_eq!(positions, (1..=n as u16).collect::<Vec<_>>());
    let distinct: HashSet<_> = t.iter().map(|r| r.participant).collect();
    assert_eq!(distinct.len(), n);
    assert!(t.windows(2).all(|w| w[0].points >= w[1].points));
    assert!(t.windows(2).all(|w| w[0].series_diff >= w[1].series_diff));
    let wins: u32 = t.iter().map(|r| u32::from(r.wins)).sum();
    let losses: u32 = t.iter().map(|r| u32::from(r.losses)).sum();
    assert_eq!(wins, losses);
    assert_eq!(wins as usize, n * (n - 1) / 2);
    let map_diff: i32 = t.iter().map(|r| r.map_diff).sum();
    assert_eq!(map_diff, 0);
    // Os frags de uma série entram pró de um e contra do outro, com sinal.
    let frags_for: i64 = t.iter().map(|r| r.frags_for).sum();
    let frags_against: i64 = t.iter().map(|r| r.frags_against).sum();
    assert_eq!(frags_for, frags_against);
    assert_eq!(t.iter().map(|r| r.frag_diff).sum::<i64>(), 0);
    for r in t {
        assert_eq!(r.points, r.wins);
        assert_eq!(r.series_diff, i32::from(r.wins) - i32::from(r.losses));
        assert_eq!(r.draws, 0);
        assert_eq!(r.played as usize, n - 1);
        assert_eq!((r.byes, r.buchholz), (0, None));
    }
    assert_eq!(t.last().unwrap().ahead_by, None);
    assert!(t[..n - 1].iter().all(|r| r.ahead_by.is_some()));
}

proptest! {
    #[test]
    fn any_results_finish_with_a_consistent_table(
        n in 2i64..=16,
        picks in proptest::collection::vec(
            (any::<bool>(), any::<bool>(), 0u8..3, -50i32..200, -50i32..200), 1..40),
        order_seed in any::<u64>(),
    ) {
        let mut b = RoundRobinBracket::new(config(), seeds(n)).unwrap();
        check_schedule(&b);
        let ready: Vec<Match> = b.matches().iter()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
            .copied()
            .collect();
        let mut reports = Vec::new();
        for (i, m) in ready.iter().enumerate() {
            let r = report(m, i, &picks);
            b.report(r).unwrap();
            reports.push(r);
        }
        prop_assert!(b.is_finished());
        check_table(&b);
        prop_assert_eq!(b.placements().len(), n as usize);

        // Replay determinístico, e a ordem dos resultados não importa.
        let again = RoundRobinBracket::replay(config(), seeds(n), reports.clone()).unwrap();
        prop_assert_eq!(&again, &b);
        let k = reports.len();
        let shift = (order_seed as usize) % k;
        reports.rotate_left(shift);
        reports.reverse();
        let shuffled = RoundRobinBracket::replay(config(), seeds(n), reports).unwrap();
        prop_assert_eq!(shuffled.table(), b.table());
    }

    /// A classificação final em md ímpar é a mesma de antes do saldo: em torneio
    /// completo, sem empates, a ordem por saldo de séries é a ordem por vitórias
    /// (todos jogam N − 1 séries). W.O. e série sem frags inclusive.
    #[test]
    fn a_finished_odd_series_ranks_the_same_by_balance_and_by_wins(
        n in 2i64..=16,
        best_of in prop_oneof![Just(1u8), Just(3u8), Just(5u8), Just(7u8)],
        picks in proptest::collection::vec(
            (any::<bool>(), any::<bool>(), 0u8..4, -50i32..200, -50i32..200), 1..40),
    ) {
        let config = RoundRobinConfig { best_of };
        let mut b = RoundRobinBracket::new(config, seeds(n)).unwrap();
        let mut by_wins = table::League::new(
            BY_WINS,
            seeds(n),
            schedule::build(&seeds(n), best_of),
        );
        let ready: Vec<Match> = b.matches().iter()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
            .copied()
            .collect();
        for (i, m) in ready.iter().enumerate() {
            let r = report(m, i, &picks);
            b.report(r).unwrap();
            by_wins.report(r).unwrap();
        }
        prop_assert!(b.is_finished());
        let order = |t: &[TableRow]| t.iter().map(|r| r.participant).collect::<Vec<_>>();
        prop_assert_eq!(order(b.table()), order(by_wins.table()));
    }

    /// O mesmo nas séries pares sem nenhum empate: é uma série como as outras.
    #[test]
    fn an_even_series_without_draws_ranks_like_any_other(
        n in 2i64..=12,
        best_of in prop_oneof![Just(2u8), Just(4u8), Just(6u8)],
        firsts in proptest::collection::vec(any::<bool>(), 1..40),
    ) {
        let config = RoundRobinConfig { best_of };
        let mut b = RoundRobinBracket::new(config, seeds(n)).unwrap();
        let mut by_wins = table::League::new(
            BY_WINS,
            seeds(n),
            schedule::build(&seeds(n), best_of),
        );
        let ready: Vec<Match> = b.matches().iter()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
            .copied()
            .collect();
        for (i, m) in ready.iter().enumerate() {
            let MatchState::Ready { a, b: other } = m.state else { unreachable!() };
            let winner = if firsts[i % firsts.len()] { a } else { other };
            let r = StageReport::from(MatchReport {
                match_id: m.id,
                outcome: Outcome::Played {
                    winner,
                    score: SeriesScore { winner: wins_needed(best_of), loser: 0 },
                },
            });
            b.report(r).unwrap();
            by_wins.report(r).unwrap();
        }
        let order = |t: &[TableRow]| t.iter().map(|r| r.participant).collect::<Vec<_>>();
        prop_assert_eq!(order(b.table()), order(by_wins.table()));
    }

    /// Com empates, as contas fecham: cada empate soma um para os dois lados e
    /// metade dos mapas para cada um, o saldo soma zero e, só na md2, o saldo de
    /// mapas é o dobro do de séries.
    #[test]
    fn an_even_series_with_draws_adds_up(
        n in 2i64..=12,
        best_of in prop_oneof![Just(2u8), Just(4u8), Just(6u8)],
        // 0 e 1: venceu um dos lados; 2: empate; 3: W.O.
        kinds in proptest::collection::vec(0u8..4, 1..40),
    ) {
        let config = RoundRobinConfig { best_of };
        let mut b = RoundRobinBracket::new(config, seeds(n)).unwrap();
        let ready: Vec<Match> = b.matches().iter()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
            .copied()
            .collect();
        let mut draws = 0usize;
        let swept = SeriesScore { winner: wins_needed(best_of), loser: 0 };
        for (i, m) in ready.iter().enumerate() {
            let MatchState::Ready { a, b: other } = m.state else { unreachable!() };
            let outcome = match kinds[i % kinds.len()] {
                0 => Outcome::Played { winner: a, score: swept },
                1 => Outcome::Played { winner: other, score: swept },
                2 => {
                    draws += 1;
                    Outcome::Drawn { maps: best_of / 2 }
                }
                _ => Outcome::Walkover { winner: a },
            };
            b.report(StageReport::from(MatchReport { match_id: m.id, outcome })).unwrap();
        }
        prop_assert!(b.is_finished());
        prop_assert_eq!(b.placements().len(), n as usize);
        let t = b.table();
        prop_assert!(t.windows(2).all(|w| w[0].series_diff >= w[1].series_diff));
        prop_assert_eq!(t.iter().map(|r| r.series_diff).sum::<i32>(), 0);
        prop_assert_eq!(t.iter().map(|r| usize::from(r.draws)).sum::<usize>(), 2 * draws);
        prop_assert_eq!(t.iter().map(|r| r.map_diff).sum::<i32>(), 0);
        for r in t {
            prop_assert_eq!(r.played as usize, n as usize - 1);
            if best_of == 2 {
                prop_assert_eq!(r.map_diff, 2 * r.series_diff);
            }
            prop_assert_eq!(usize::from(r.wins + r.draws + r.losses), n as usize - 1);
        }
    }
}
