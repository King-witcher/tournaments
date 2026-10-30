//! Propriedades com proptest: qualquer sequência de resultados.

use super::*;
use proptest::prelude::*;

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
        frags: (fw % 5 != 0).then_some(SeriesFrags {
            winner: fw,
            loser: fl,
        }),
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
}
