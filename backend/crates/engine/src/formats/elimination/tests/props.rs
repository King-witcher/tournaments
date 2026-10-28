//! Propriedades com proptest.

use super::*;

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
        n in 2i64..=64,
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
