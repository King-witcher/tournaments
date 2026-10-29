//! Propriedades com proptest: resultados e desistências quaisquer.

use super::*;
use proptest::prelude::*;

fn key(a: ParticipantId, b: ParticipantId) -> (ParticipantId, ParticipantId) {
    (a.min(b), a.max(b))
}

/// Há pareamento de `players` (número par) sem repetir um par de `met`?
/// Força bruta, independente do emparelhamento do pareamento.
fn perfect(players: &[ParticipantId], met: &HashSet<(ParticipantId, ParticipantId)>) -> bool {
    let Some((first, rest)) = players.split_first() else {
        return true;
    };
    rest.iter().enumerate().any(|(i, q)| {
        !met.contains(&key(*first, *q)) && {
            let mut others = rest.to_vec();
            others.remove(i);
            perfect(&others, met)
        }
    })
}

/// Confere o pareamento `round` gerado a partir de `b`.
fn check_round(b: &SwissBracket, round: &SwissRound) {
    let active: Vec<ParticipantId> = b
        .participants()
        .iter()
        .filter(|x| !b.out().contains(x))
        .copied()
        .collect();
    // Todo ativo uma vez; ninguém de fora.
    let everyone: Vec<ParticipantId> = round
        .pairs
        .iter()
        .flatten()
        .chain(&round.bye)
        .copied()
        .collect();
    let distinct: HashSet<ParticipantId> = everyone.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        everyone.len(),
        "alguém duas vezes: {round:?}"
    );
    assert_eq!(distinct, active.iter().copied().collect::<HashSet<_>>());

    // Bye: só com número ímpar, para quem tem menos byes.
    let byes = |x: ParticipantId| b.table().iter().find(|r| r.participant == x).unwrap().byes;
    let fewest = active.iter().map(|x| byes(*x)).min().unwrap();
    assert_eq!(round.bye.is_some(), active.len() % 2 == 1);
    if let Some(bye) = round.bye {
        assert_eq!(
            byes(bye),
            fewest,
            "bye repetido com quem não teve: {round:?}"
        );
    }

    // Revanche só sem alternativa (respeitando a regra do bye).
    let met: HashSet<_> = b
        .rounds()
        .iter()
        .flat_map(|r| &r.pairs)
        .map(|[x, y]| key(*x, *y))
        .collect();
    let rematch = round.pairs.iter().any(|[x, y]| met.contains(&key(*x, *y)));
    if rematch {
        let alternative = if active.len().is_multiple_of(2) {
            perfect(&active, &met)
        } else {
            active.iter().filter(|c| byes(**c) == fewest).any(|c| {
                let others: Vec<_> = active.iter().filter(|x| *x != c).copied().collect();
                perfect(&others, &met)
            })
        };
        assert!(
            !alternative,
            "revanche evitável: {round:?} depois de {:?}",
            b.rounds()
        );
    }
    // O lado `a` é o melhor colocado.
    let position = |x: ParticipantId| b.table().iter().position(|r| r.participant == x).unwrap();
    assert!(round.pairs.iter().all(|[x, y]| position(*x) < position(*y)));
}

proptest! {
    #[test]
    fn any_results_and_withdrawals_keep_the_rules(
        n in 2i64..=14,
        rounds in proptest::option::of(1u16..=13),
        picks in proptest::collection::vec((any::<bool>(), 0u8..2, 0u32..100, 0u32..100), 1..40),
        withdrawals in proptest::collection::vec((0usize..60, 0i64..14), 0..4),
    ) {
        let mut run = Run::new(n, rounds);
        let total = run.bracket().total_rounds();
        let mut step = 0usize;
        loop {
            prop_assert!(step < 1000);
            for &(when, who) in &withdrawals {
                let x = p(who % n + 1);
                if when == step && !run.out.contains(&x) {
                    run.out.push(x);
                }
            }
            let b = run.bracket();
            if let Some(round) = b.next_round() {
                check_round(&b, &round);
                prop_assert_eq!(b.next_round(), Some(round.clone()));
                run.rounds.push(round);
            } else if let Some(m) = b.matches().iter().find(|m| matches!(m.state, MatchState::Ready { .. })) {
                let MatchState::Ready { a, b: other } = m.state else { unreachable!() };
                let (first, loser_maps, fw, fl) = picks[step % picks.len()];
                let report = match (run.out.contains(&a), run.out.contains(&other)) {
                    (true, false) => walkover(m, other),
                    (false, true) | (true, true) => walkover(m, a),
                    (false, false) => {
                        let winner = if first { a } else { other };
                        let mut r = played(m, winner, loser_maps);
                        r.frags = (fw % 7 != 0).then_some(SeriesFrags { winner: fw, loser: fl });
                        r
                    }
                };
                run.reports.push(report);
            } else {
                break;
            }
            step += 1;
        }
        let b = run.bracket();
        prop_assert!(b.is_finished());
        prop_assert!(b.rounds().len() <= usize::from(total));
        let active = n as usize - b.out().len();
        if active >= 2 {
            prop_assert_eq!(b.rounds().len(), usize::from(total));
        }
        let placements = b.placements();
        prop_assert_eq!(placements.len(), n as usize);
        prop_assert!(placements.iter().enumerate().all(|(i, x)| x.rank as usize == i + 1 && x.tied == 1));
        // Uma partida (ou bye) por rodada, no máximo, e o pior caso vale.
        for x in b.participants() {
            let seen = b.matches().iter().filter(|m| {
                matches!(m.a, Slot::Participant(y) if y == *x) || matches!(m.b, Slot::Participant(y) if y == *x)
            }).count();
            prop_assert!(seen <= b.rounds().len());
            let r = b.table().iter().find(|r| r.participant == *x).unwrap();
            prop_assert!(u32::from(r.maps_won + r.maps_lost) <= b.worst_case_games());
        }
        // Replay determinístico.
        prop_assert_eq!(run.bracket(), b);
    }
}
