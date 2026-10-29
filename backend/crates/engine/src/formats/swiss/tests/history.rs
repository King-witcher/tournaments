//! Histórico: replay, correções, desistências, fim e erros.

use super::*;

#[test]
fn replay_with_the_same_history_is_the_same() {
    let mut run = Run::new(9, None);
    run.play(favorite);
    let b = run.bracket();
    assert!(b.is_finished());
    assert_eq!(run.bracket(), b);
    assert_eq!(b.rounds().len(), 4);
    assert_eq!(b.placements().len(), 9);
    assert_eq!(b.placements()[0].participant, b.table()[0].participant);
    assert_eq!(b.worst_case_games(), 12);
}

#[test]
fn placements_only_at_the_end() {
    let mut run = Run::new(4, None);
    run.pair();
    run.decide(favorite);
    let b = run.bracket();
    assert!(!b.is_finished());
    assert!(b.placements().is_empty());
    assert_eq!(b.table().len(), 4);
}

#[test]
fn correction_changes_the_table_not_the_pairings() {
    let mut run = Run::new(8, None);
    run.pair();
    run.decide(favorite);
    run.pair();
    run.decide(favorite);
    let before = run.bracket();
    // Corrige 1 x 5, da rodada 1, na mesma posição do histórico: 5 venceu.
    let i = run
        .reports
        .iter()
        .position(|r| r.report.match_id == id(1, 1))
        .unwrap();
    run.reports[i] = played(before.get(id(1, 1)).unwrap(), p(5), 1);
    let after = run.bracket();
    let sides = |b: &SwissBracket| -> Vec<(MatchId, Slot, Slot)> {
        b.matches().iter().map(|m| (m.id, m.a, m.b)).collect()
    };
    assert_eq!(sides(&after), sides(&before));
    assert_eq!(after.rounds(), before.rounds());
    assert_ne!(after.table(), before.table());
    let points = |b: &SwissBracket, n: i64| {
        b.table()
            .iter()
            .find(|r| r.participant == p(n))
            .unwrap()
            .points
    };
    assert_eq!((points(&before, 5), points(&after, 5)), (1, 2));
    assert_eq!((points(&before, 1), points(&after, 1)), (2, 1));
}

#[test]
fn reopened_series_holds_the_next_round() {
    let mut run = Run::new(4, None);
    run.pair();
    run.decide(favorite);
    assert!(run.bracket().next_round().is_some());
    // A correção que reabre a série (sai do histórico) segura a rodada.
    run.reports.remove(0);
    assert_eq!(run.bracket().next_round(), None);
}

#[test]
fn withdrawn_players_are_not_paired() {
    let mut run = Run::new(6, None);
    run.pair();
    run.decide(favorite);
    // O líder sai entre as rodadas: 5 ativos, bye para o pior ativo.
    run.out.push(p(1));
    let r = run.pair().unwrap();
    assert_eq!(plain(&r), (vec![[2, 3], [4, 5]], Some(6)));
    assert_eq!(run.bracket().out(), [p(1)]);
}

#[test]
fn withdrawal_mid_round_waits_for_the_walkover() {
    let mut run = Run::new(6, None);
    run.pair();
    run.decide(favorite);
    run.pair();
    // 4 sai com a rodada 2 em andamento: a partida dele segue pronta até a
    // API dar o W.O.
    run.out.push(p(4));
    let b = run.bracket();
    let m = b
        .matches()
        .iter()
        .find(|m| {
            m.id.round == 2
                && matches!(m.state, MatchState::Ready { a, b } if a == p(4) || b == p(4))
        })
        .copied()
        .unwrap();
    assert_eq!(b.next_round(), None);
    run.decide(favorite);
    let b = run.bracket();
    assert!(matches!(
        b.get(m.id).unwrap().state,
        MatchState::Completed {
            outcome: Outcome::Walkover { .. },
            ..
        }
    ));
    let r = b.next_round().unwrap();
    let everyone: Vec<i64> = r
        .pairs
        .iter()
        .flatten()
        .chain(&r.bye)
        .map(|x| x.0)
        .collect();
    assert!(!everyone.contains(&4));
    assert_eq!(everyone.len(), 5);
}

#[test]
fn ends_early_without_two_players_left() {
    let mut run = Run::new(4, Some(3));
    run.pair();
    run.out = vec![p(2), p(3), p(4)];
    assert!(!run.bracket().is_finished());
    run.decide(favorite);
    let b = run.bracket();
    assert!(b.is_finished());
    assert_eq!(b.next_round(), None);
    assert_eq!(b.rounds().len(), 1);
    assert_eq!(b.placements().len(), 4);
    assert_eq!(b.placements()[0].participant, p(1));
}

#[test]
fn unknown_out_ids_are_ignored() {
    let b = SwissBracket::replay(config(None), seeds(4), Vec::new(), [], &[p(9), p(2)]).unwrap();
    assert_eq!(b.out(), [p(2)]);
}

#[test]
fn bad_pairings_are_rejected() {
    let bad = |rounds: Vec<SwissRound>, round: u16| {
        assert_eq!(
            SwissBracket::replay(config(None), seeds(4), rounds, [], &[]),
            Err(EngineError::InvalidPairing { round })
        );
    };
    bad(vec![round(&[[1, 9]], None)], 1);
    bad(vec![round(&[[1, 2], [2, 3]], None)], 1);
    bad(vec![round(&[[1, 1]], None)], 1);
    bad(vec![round(&[[1, 2]], Some(1))], 1);
    bad(vec![round(&[], Some(1))], 1);
    let ok = round(&[[1, 3], [2, 4]], None);
    bad(vec![ok.clone(), round(&[[1, 5]], None)], 2);
    // Com 4, são 2 rodadas.
    bad(vec![ok.clone(), ok.clone(), ok], 3);
}

#[test]
fn report_errors() {
    let mut run = Run::new(3, None);
    run.pair();
    let mut b = run.bracket();
    let bye = *b.get(id(1, 2)).unwrap();
    assert_eq!(
        b.report(played(&bye, p(3), 0)),
        Err(EngineError::MatchNotReady(bye.id))
    );
    let m = *b.get(id(1, 1)).unwrap();
    let mut later = m;
    later.id = id(2, 1);
    assert_eq!(
        b.report(played(&later, p(1), 0)),
        Err(EngineError::UnknownMatch(later.id))
    );
    b.report(played(&m, p(2), 1)).unwrap();
    assert_eq!(
        b.report(walkover(&m, p(1))),
        Err(EngineError::MatchAlreadyReported(m.id))
    );
}
