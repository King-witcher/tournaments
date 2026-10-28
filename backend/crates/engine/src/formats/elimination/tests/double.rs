//! Eliminação dupla, pior caso e ausência de revanche cedo.

use super::*;

// ---- eliminação dupla ----

#[test]
fn double_two_participants() {
    let mut b = EliminationBracket::new(double(true), seeds(2)).unwrap();
    assert_eq!(count(&b, L), 0);
    assert_eq!(count(&b, GF), 2);
    assert_eq!(b.get(id(GF, 1, 1)).unwrap().a, Slot::WinnerOf(id(W, 1, 1)));
    assert_eq!(b.get(id(GF, 1, 1)).unwrap().b, Slot::LoserOf(id(W, 1, 1)));
    let m = *b.get(id(W, 1, 1)).unwrap();
    b.report(played(&m, p(1))).unwrap();
    assert_eq!(
        b.get(id(GF, 1, 1)).unwrap().state,
        MatchState::Ready { a: p(1), b: p(2) }
    );
    let gf = *b.get(id(GF, 1, 1)).unwrap();
    b.report(played(&gf, p(2))).unwrap();
    assert!(!b.is_finished());
    assert_eq!(
        b.get(id(GF, 2, 1)).unwrap().state,
        MatchState::Ready { a: p(1), b: p(2) }
    );
    let gf2 = *b.get(id(GF, 2, 1)).unwrap();
    b.report(played(&gf2, p(2))).unwrap();
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
    assert_eq!(b.worst_case_matches(), 3);
    assert_eq!(b.worst_case_games(), 3 + 5 + 5);
}

#[test]
fn double_shapes() {
    // (participantes, partidas das chaves de cima e de baixo)
    for (n, upper, lower) in [(3, 3, 2), (4, 3, 2), (5, 7, 6), (8, 7, 6)] {
        let b = EliminationBracket::new(double(true), seeds(n)).unwrap();
        assert_eq!(count(&b, W), upper, "n={n}");
        assert_eq!(count(&b, L), lower, "n={n}");
        assert_eq!(count(&b, GF), 2, "n={n}");
        let b = EliminationBracket::new(double(false), seeds(n)).unwrap();
        assert_eq!(count(&b, GF), 1, "n={n}");
        assert_eq!(b.matches().last().unwrap().id, id(GF, 1, 1));
    }
    let b = EliminationBracket::new(double(true), seeds(3)).unwrap();
    // L1 junta o perdedor do bye com o de 2 x 3: fica pendente até 2 x 3 acabar.
    assert_eq!(b.get(id(L, 1, 1)).unwrap().state, MatchState::Pending);
    let b = EliminationBracket::new(double(true), seeds(4)).unwrap();
    assert_eq!(b.get(id(L, 1, 1)).unwrap().a, Slot::LoserOf(id(W, 1, 1)));
    assert_eq!(b.get(id(L, 1, 1)).unwrap().b, Slot::LoserOf(id(W, 1, 2)));
    assert_eq!(b.get(id(L, 2, 1)).unwrap().a, Slot::WinnerOf(id(L, 1, 1)));
    assert_eq!(b.get(id(L, 2, 1)).unwrap().b, Slot::LoserOf(id(W, 2, 1)));
    // melhor de N: só a grande final usa `finals`
    assert_eq!(b.get(id(W, 2, 1)).unwrap().best_of, 3);
    assert_eq!(b.get(id(L, 2, 1)).unwrap().best_of, 3);
    assert_eq!(b.get(id(GF, 1, 1)).unwrap().best_of, 5);
    assert_eq!(b.get(id(GF, 2, 1)).unwrap().best_of, 5);
}

#[test]
fn double_byes_cascade_into_losers_bracket() {
    let mut b = EliminationBracket::new(double(true), seeds(3)).unwrap();
    assert_eq!(b.get(id(L, 1, 1)).unwrap().state, MatchState::Pending);
    assert_eq!(b.ready_matches().count(), 1);
    let m = *b.get(id(W, 1, 2)).unwrap();
    b.report(played(&m, p(2))).unwrap();
    // o perdedor de 2 x 3 avança sem jogar na L1 e espera o perdedor da final de cima.
    assert_eq!(
        b.get(id(L, 1, 1)).unwrap().state,
        MatchState::Bye {
            advanced: Some(p(3))
        }
    );
    assert_eq!(b.get(id(L, 2, 1)).unwrap().state, MatchState::Pending);
    let m = *b.get(id(W, 2, 1)).unwrap();
    b.report(played(&m, p(1))).unwrap();
    assert_eq!(
        b.get(id(L, 2, 1)).unwrap().state,
        MatchState::Ready { a: p(3), b: p(2) }
    );
}

#[test]
fn grand_final_without_reset_ends_after_round_one() {
    let mut b = EliminationBracket::new(double(false), seeds(4)).unwrap();
    play_favorites(&mut b, true);
    assert!(!b.is_finished());
    let (up, low) = ready_pair(&b, id(GF, 1, 1));
    let gf = *b.get(id(GF, 1, 1)).unwrap();
    b.report(played(&gf, low)).unwrap();
    assert!(b.is_finished());
    assert!(b.get(id(GF, 2, 1)).is_none());
    let s = b.standings();
    assert_eq!(s[0].participant, low);
    assert_eq!(
        s[1],
        Placement {
            participant: up,
            rank: 2,
            tied: 1
        }
    );
}

#[test]
fn grand_final_reset_not_needed_when_upper_champion_wins() {
    let mut b = EliminationBracket::new(double(true), seeds(4)).unwrap();
    play_favorites(&mut b, true);
    assert_eq!(b.get(id(GF, 2, 1)).unwrap().state, MatchState::Pending);
    let (up, _) = ready_pair(&b, id(GF, 1, 1));
    let gf = *b.get(id(GF, 1, 1)).unwrap();
    b.report(played(&gf, up)).unwrap();
    assert_eq!(b.get(id(GF, 2, 1)).unwrap().state, MatchState::NotNeeded);
    assert!(b.is_finished());
    assert_eq!(b.standings()[0].participant, up);
    assert_eq!(b.ready_matches().count(), 0);
}

#[test]
fn grand_final_reset_needed_when_lower_champion_wins() {
    let mut b = EliminationBracket::new(double(true), seeds(4)).unwrap();
    play_favorites(&mut b, true);
    let (up, low) = ready_pair(&b, id(GF, 1, 1));
    let gf = *b.get(id(GF, 1, 1)).unwrap();
    b.report(played(&gf, low)).unwrap();
    assert!(!b.is_finished());
    assert_eq!(
        b.get(id(GF, 2, 1)).unwrap().state,
        MatchState::Ready { a: up, b: low }
    );
    let gf2 = *b.get(id(GF, 2, 1)).unwrap();
    b.report(played(&gf2, up)).unwrap();
    assert!(b.is_finished());
    let s = b.standings();
    assert_eq!(s[0].participant, up);
    assert_eq!(
        s[1],
        Placement {
            participant: low,
            rank: 2,
            tied: 1
        }
    );
}

#[test]
fn double_standings_partial_and_final() {
    let mut b = EliminationBracket::new(double(false), seeds(8)).unwrap();
    // ninguém eliminado depois da chave de cima
    for round in 1..=3 {
        for m in b
            .matches()
            .iter()
            .filter(|m| m.id.side == W && m.id.round == round)
            .copied()
            .collect::<Vec<_>>()
        {
            let (x, y) = ready_pair(&b, m.id);
            b.report(played(&m, x.min(y))).unwrap();
        }
    }
    assert!(b.standings().is_empty());
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
    // L6 (final da chave de baixo): 3º; L5: 4º; L4: 5º-6º; L3: 7º... conforme as rodadas.
    let ranks: Vec<(u16, u16)> = s.iter().map(|x| (x.rank, x.tied)).collect();
    assert_eq!(
        ranks,
        vec![
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 1),
            (5, 2),
            (5, 2),
            (7, 2),
            (7, 2)
        ]
    );
}

#[test]
fn double_eliminated_band_is_final_when_eliminated() {
    let mut b = EliminationBracket::new(double(true), seeds(8)).unwrap();
    // joga tudo o que está pronto, só uma vez por rodada de passagem
    let mut early = Vec::new();
    for _ in 0..3 {
        let ready: Vec<Match> = b.ready_matches().copied().collect();
        for m in ready {
            let (x, y) = ready_pair(&b, m.id);
            b.report(played(&m, x.min(y))).unwrap();
        }
        early.push(b.standings());
    }
    let last = early.last().unwrap().clone();
    let mut fin = EliminationBracket::new(double(true), seeds(8)).unwrap();
    play_favorites(&mut fin, true);
    let (up, _) = ready_pair(&fin, id(GF, 1, 1));
    let gf = *fin.get(id(GF, 1, 1)).unwrap();
    fin.report(played(&gf, up)).unwrap();
    let end = fin.standings();
    for pl in last {
        assert!(end.contains(&pl), "{pl:?} mudou de faixa");
    }
}

// ---- pior caso ----

#[test]
fn worst_case_double_sixteen() {
    let b = EliminationBracket::new(double(true), seeds(16)).unwrap();
    assert_eq!(b.worst_case_matches(), 9);
    assert_eq!(b.worst_case_games(), 7 * 3 + 2 * 5);
    let b = EliminationBracket::new(double(false), seeds(16)).unwrap();
    assert_eq!(b.worst_case_matches(), 8);
    assert_eq!(b.worst_case_games(), 7 * 3 + 5);
}

// ---- sem revanche cedo ----

#[test]
fn dropping_players_do_not_meet_old_opponents_first() {
    for n in [8i64, 16, 32] {
        let mut b = EliminationBracket::new(double(false), seeds(n)).unwrap();
        let mut met: HashSet<(ParticipantId, ParticipantId)> = HashSet::new();
        loop {
            let next = b.ready_matches().next().copied();
            let Some(m) = next else { break };
            let (x, y) = ready_pair(&b, m.id);
            // A última descida (perdedor da final de cima contra o campeão da chave de baixo)
            // é sempre revanche com resultados de favorito: o seed 3 perdeu para o 2.
            let last_drop = 2 * (n.trailing_zeros() as u16 - 1);
            if m.id.side == L && m.id.round.is_multiple_of(2) && m.id.round < last_drop {
                assert!(!met.contains(&(x, y)), "n={n}: revanche em {:?}", m.id);
            }
            met.insert((x, y));
            met.insert((y, x));
            b.report(played(&m, x.min(y))).unwrap();
        }
        assert!(b.is_finished());
    }
}
