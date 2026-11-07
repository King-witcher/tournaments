//! O empate que sobra no fim da fase (`ties`): pendente só no fim e só com a
//! organização decidindo, a decisão aplicada ao grupo de mesmo conjunto, e o
//! seed como sempre no padrão.

use super::*;
use crate::formats::round_robin::{RoundRobinBracket, RoundRobinConfig};
use crate::formats::swiss::{SwissConfig, SwissRound};
use crate::formats::{StageBracket, StageFormat, StageHistory};

fn seeds(n: i64) -> Vec<ParticipantId> {
    (1..=n).map(p).collect()
}

fn rr(final_tie: FinalTie) -> RoundRobinConfig {
    RoundRobinConfig {
        best_of: 1,
        tiebreakers: Tiebreakers::NONE,
        final_tie,
    }
}

fn order_of(list: &[i64]) -> TieOrder {
    TieOrder {
        participants: list.iter().map(|n| p(*n)).collect(),
    }
}

/// `winner` vence `loser` por 1 x 0 na partida pronta entre os dois.
fn beat(matches: &[Match], winner: i64, loser: i64) -> StageReport {
    let m = matches
        .iter()
        .find(|m| {
            matches!(m.state, MatchState::Ready { a, b }
                if (a, b) == (p(winner), p(loser)) || (a, b) == (p(loser), p(winner)))
        })
        .unwrap();
    StageReport::from(MatchReport {
        match_id: m.id,
        outcome: Outcome::Played {
            winner: p(winner),
            score: SeriesScore {
                winner: 1,
                loser: 0,
            },
        },
    })
}

/// Round robin de 4 sem desempates: 1 vence todos; 2, 3 e 4 ficam com uma
/// vitória cada (2 > 3, 3 > 4, 4 > 2), e só o seed os separa.
fn cycle(final_tie: FinalTie) -> (RoundRobinBracket, Vec<StageReport>) {
    let mut b = RoundRobinBracket::new(rr(final_tie), seeds(4)).unwrap();
    let mut reports = Vec::new();
    for (w, l) in [(1, 2), (1, 3), (1, 4), (2, 3), (3, 4), (4, 2)] {
        let r = beat(b.matches(), w, l);
        b.report(r).unwrap();
        reports.push(r);
    }
    (b, reports)
}

fn table_order(table: &[TableRow]) -> Vec<i64> {
    table.iter().map(|r| r.participant.0).collect()
}

#[test]
fn with_the_seed_the_tie_is_decided_as_before() {
    let (b, _) = cycle(FinalTie::Seed);
    assert!(b.is_finished());
    assert!(b.pending_ties().is_empty());
    assert_eq!(table_order(b.table()), [1, 2, 3, 4]);
    let ahead: Vec<_> = b.table().iter().map(|r| r.ahead_by).collect();
    assert_eq!(ahead, [Some(SeriesDiff), Some(Seed), Some(Seed), None]);
    assert_eq!(b.placements().len(), 4);
}

#[test]
fn the_organizer_tie_waits_only_at_the_end() {
    let mut b = RoundRobinBracket::new(rr(FinalTie::Organizer), seeds(4)).unwrap();
    for (w, l) in [(1, 2), (1, 3), (1, 4), (2, 3), (3, 4)] {
        let r = beat(b.matches(), w, l);
        b.report(r).unwrap();
        // Durante a fase, nada pendente: o seed desempata a tabela ao vivo.
        assert!(b.pending_ties().is_empty());
        assert!(!b.is_finished());
    }
    let r = beat(b.matches(), 4, 2);
    b.report(r).unwrap();
    assert_eq!(
        b.pending_ties(),
        [PendingTie {
            position: 2,
            participants: vec![p(2), p(3), p(4)],
        }]
    );
    assert!(!b.is_finished());
    // Provisória, na ordem do seed; só o 1º já tem posição.
    assert_eq!(table_order(b.table()), [1, 2, 3, 4]);
    let placements = b.placements();
    assert_eq!(placements.len(), 1);
    assert_eq!((placements[0].participant, placements[0].rank), (p(1), 1));
}

#[test]
fn the_decision_orders_the_group() {
    let (_, reports) = cycle(FinalTie::Organizer);
    let config = rr(FinalTie::Organizer);
    let b = RoundRobinBracket::replay_with_ties(
        config,
        seeds(4),
        reports,
        &[],
        vec![order_of(&[4, 2, 3])],
    )
    .unwrap();
    assert!(b.is_finished());
    assert!(b.pending_ties().is_empty());
    assert_eq!(table_order(b.table()), [1, 4, 2, 3]);
    let positions: Vec<u16> = b.table().iter().map(|r| r.position).collect();
    assert_eq!(positions, [1, 2, 3, 4]);
    let ahead: Vec<_> = b.table().iter().map(|r| r.ahead_by).collect();
    assert_eq!(
        ahead,
        [Some(SeriesDiff), Some(Organizer), Some(Organizer), None]
    );
    let ranks: Vec<(i64, u16)> = b
        .placements()
        .iter()
        .map(|pl| (pl.participant.0, pl.rank))
        .collect();
    assert_eq!(ranks, [(1, 1), (4, 2), (2, 3), (3, 4)]);
}

#[test]
fn a_decision_for_another_set_does_not_count() {
    let (_, reports) = cycle(FinalTie::Organizer);
    // Um conjunto a menos (como antes de uma correção) e um a mais.
    for stale in [order_of(&[3, 2]), order_of(&[4, 3, 2, 1])] {
        let b = RoundRobinBracket::replay_with_ties(
            rr(FinalTie::Organizer),
            seeds(4),
            reports.clone(),
            &[],
            vec![stale],
        )
        .unwrap();
        assert_eq!(b.pending_ties().len(), 1);
        assert!(!b.is_finished());
        assert_eq!(table_order(b.table()), [1, 2, 3, 4]);
    }
}

#[test]
fn the_last_decision_for_the_same_set_counts() {
    let (_, reports) = cycle(FinalTie::Organizer);
    let b = RoundRobinBracket::replay_with_ties(
        rr(FinalTie::Organizer),
        seeds(4),
        reports,
        &[],
        vec![order_of(&[4, 2, 3]), order_of(&[3, 4, 2])],
    )
    .unwrap();
    assert_eq!(table_order(b.table()), [1, 3, 4, 2]);
}

#[test]
fn decisions_do_nothing_with_the_seed() {
    let (_, reports) = cycle(FinalTie::Seed);
    let b = RoundRobinBracket::replay_with_ties(
        rr(FinalTie::Seed),
        seeds(4),
        reports,
        &[],
        vec![order_of(&[4, 2, 3])],
    )
    .unwrap();
    assert_eq!(table_order(b.table()), [1, 2, 3, 4]);
    assert!(b.is_finished());
}

#[test]
fn pending_tie_knows_its_orders() {
    let tie = PendingTie {
        position: 2,
        participants: vec![p(2), p(3)],
    };
    assert!(tie.is_order(&[p(3), p(2)]));
    assert!(!tie.is_order(&[p(3)]));
    assert!(!tie.is_order(&[p(3), p(3)]));
    assert!(!tie.is_order(&[p(3), p(2), p(4)]));
}

fn swiss(final_tie: FinalTie) -> StageFormat {
    StageFormat::Swiss(SwissConfig {
        best_of: 1,
        rounds: Some(2),
        tiebreakers: Tiebreakers::NONE,
        final_tie,
    })
}

/// Suíço de 4 em 2 rodadas, os favoritos vencendo: 2 e 3 terminam com 1
/// ponto, e só o seed os separa. Devolve a fase em cada passo (depois da
/// rodada 1 decidida e no fim).
fn swiss_played(final_tie: FinalTie, orders: Vec<TieOrder>) -> (StageBracket, StageBracket) {
    let format = swiss(final_tie);
    let mut history = StageHistory {
        tie_orders: orders,
        ..StageHistory::default()
    };
    let mut stage = StageBracket::new(format, seeds(4)).unwrap();
    let mut after_round_one = None;
    loop {
        if let Some(round) = stage.next_round() {
            if history.rounds.len() == 1 {
                after_round_one = Some(stage.clone());
            }
            history.rounds.push(round);
            stage = StageBracket::replay(format, seeds(4), history.clone()).unwrap();
            continue;
        }
        let Some(m) = stage.ready_matches().next().copied() else {
            break;
        };
        let MatchState::Ready { a, b } = m.state else {
            unreachable!()
        };
        let r = beat(stage.matches(), a.0.min(b.0), a.0.max(b.0));
        history.reports.push(r);
        stage = StageBracket::replay(format, seeds(4), history.clone()).unwrap();
    }
    (after_round_one.unwrap(), stage)
}

#[test]
fn the_swiss_pairs_and_waits_as_before() {
    let (seed_mid, seed_end) = swiss_played(FinalTie::Seed, Vec::new());
    let (mid, end) = swiss_played(FinalTie::Organizer, Vec::new());
    // A rodada 2 sai igual: no meio, o seed desempata o pareamento.
    assert_eq!(mid.next_round(), seed_mid.next_round());
    assert_eq!(
        mid.next_round(),
        Some(SwissRound {
            pairs: vec![[p(1), p(2)], [p(3), p(4)]],
            bye: None,
        })
    );
    assert!(mid.pending_ties().is_empty());
    assert!(seed_end.is_finished());
    assert!(!end.is_finished());
    assert_eq!(
        end.pending_ties(),
        [PendingTie {
            position: 2,
            participants: vec![p(2), p(3)],
        }]
    );
    let (_, decided) = swiss_played(FinalTie::Organizer, vec![order_of(&[3, 2])]);
    assert!(decided.is_finished());
    let order: Vec<i64> = decided
        .table()
        .unwrap()
        .iter()
        .map(|r| r.participant.0)
        .collect();
    assert_eq!(order, [1, 3, 2, 4]);
}

#[test]
fn elimination_and_placement_have_no_pending_tie() {
    use crate::formats::placement::PlacementConfig;
    let history = StageHistory {
        tie_orders: vec![order_of(&[2, 1])],
        ..StageHistory::default()
    };
    let stage = StageBracket::replay(
        StageFormat::Placement(PlacementConfig { best_of: 1 }),
        seeds(2),
        history,
    )
    .unwrap();
    assert!(stage.pending_ties().is_empty());
}
