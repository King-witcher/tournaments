//! Eliminação simples.

use super::*;

// ---- eliminação simples ----

#[test]
fn single_two_participants() {
    let mut b = EliminationBracket::new(single(3, 5), seeds(2)).unwrap();
    assert_eq!(b.matches().len(), 1);
    assert_eq!(b.get(id(W, 1, 1)).unwrap().best_of, 5);
    assert!(b.standings().is_empty());
    let m = *b.get(id(W, 1, 1)).unwrap();
    b.report(played(&m, p(2))).unwrap();
    assert!(b.is_finished());
    assert_eq!(
        b.standings(),
        vec![
            Placement {
                participant: p(2),
                rank: 1,
                tied: 1
            },
            Placement {
                participant: p(1),
                rank: 2,
                tied: 1
            },
        ]
    );
    assert_eq!(b.worst_case_matches(), 1);
    assert_eq!(b.worst_case_games(), 5);
}

#[test]
fn single_three_gives_seed_one_a_bye() {
    let b = EliminationBracket::new(single(3, 5), seeds(3)).unwrap();
    assert_eq!(b.matches().len(), 3);
    assert_eq!(
        b.get(id(W, 1, 1)).unwrap().state,
        MatchState::Bye {
            advanced: Some(p(1))
        }
    );
    assert_eq!(
        b.get(id(W, 1, 2)).unwrap().state,
        MatchState::Ready { a: p(2), b: p(3) }
    );
    assert_eq!(b.get(id(W, 2, 1)).unwrap().state, MatchState::Pending);
    assert_eq!(b.worst_case_matches(), 2);
    assert_eq!(b.worst_case_games(), 3 + 5);
}

#[test]
fn single_four_five_and_eight_shapes() {
    let b4 = EliminationBracket::new(single(3, 5), seeds(4)).unwrap();
    assert_eq!(b4.matches().len(), 3);
    assert_eq!(b4.ready_matches().count(), 2);
    assert_eq!(b4.get(id(W, 1, 1)).unwrap().a, Slot::Participant(p(1)));
    assert_eq!(b4.get(id(W, 1, 1)).unwrap().b, Slot::Participant(p(4)));
    assert_eq!(b4.get(id(W, 2, 1)).unwrap().a, Slot::WinnerOf(id(W, 1, 1)));
    assert_eq!(b4.get(id(W, 2, 1)).unwrap().b, Slot::WinnerOf(id(W, 1, 2)));

    let b5 = EliminationBracket::new(single(3, 5), seeds(5)).unwrap();
    assert_eq!(b5.matches().len(), 7);
    assert_eq!(b5.ready_matches().count(), 2);
    assert_eq!(b5.worst_case_matches(), 3);

    let b8 = EliminationBracket::new(single(3, 5), seeds(8)).unwrap();
    assert_eq!(b8.matches().len(), 7);
    assert_eq!(b8.ready_matches().count(), 4);
    assert_eq!(b8.worst_case_matches(), 3);
    assert_eq!(b8.worst_case_games(), 3 + 3 + 5);
    let ids: Vec<_> = b8.matches().iter().map(|m| m.id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
}

#[test]
fn six_participants_give_seeds_one_and_two_a_bye() {
    for config in [single(3, 5), double(true)] {
        let b = EliminationBracket::new(config, seeds(6)).unwrap();
        assert_eq!(
            b.get(id(W, 1, 1)).unwrap().state,
            MatchState::Bye {
                advanced: Some(p(1))
            }
        );
        assert_eq!(
            b.get(id(W, 1, 3)).unwrap().state,
            MatchState::Bye {
                advanced: Some(p(2))
            }
        );
        assert_eq!(
            b.get(id(W, 1, 2)).unwrap().state,
            MatchState::Ready { a: p(4), b: p(5) }
        );
        assert_eq!(
            b.get(id(W, 1, 4)).unwrap().state,
            MatchState::Ready { a: p(3), b: p(6) }
        );
    }
}

#[test]
fn single_standings_partial_and_final() {
    let mut b = EliminationBracket::new(single(3, 5), seeds(8)).unwrap();
    let first = *b.get(id(W, 1, 1)).unwrap();
    b.report(played(&first, p(1))).unwrap();
    assert_eq!(
        b.standings(),
        vec![Placement {
            participant: p(8),
            rank: 5,
            tied: 4
        }]
    );
    play_favorites(&mut b, false);
    assert!(b.is_finished());
    let s = b.standings();
    assert_eq!(s.len(), 8);
    assert_eq!(
        s[0],
        Placement {
            participant: p(1),
            rank: 1,
            tied: 1
        }
    );
    assert_eq!(
        s[1],
        Placement {
            participant: p(2),
            rank: 2,
            tied: 1
        }
    );
    assert!(s[2..4].iter().all(|x| x.rank == 3 && x.tied == 2));
    assert!(s[4..].iter().all(|x| x.rank == 5 && x.tied == 4));
}

#[test]
fn single_standings_ignore_byes() {
    let mut b = EliminationBracket::new(single(1, 1), seeds(5)).unwrap();
    play_favorites(&mut b, false);
    let s = b.standings();
    assert_eq!(s.len(), 5);
    // 5 vs 4 na primeira rodada: único eliminado dela, faixa de tamanho 1.
    assert_eq!(
        s[4],
        Placement {
            participant: p(5),
            rank: 5,
            tied: 1
        }
    );
    assert!(s[2..4].iter().all(|x| x.rank == 3 && x.tied == 2));
}
