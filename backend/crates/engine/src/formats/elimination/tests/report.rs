//! Reports, W.O. e replay.

use super::*;

// ---- report ----

#[test]
fn report_errors() {
    let mut b = EliminationBracket::new(single(3, 5), seeds(4)).unwrap();
    let m1 = *b.get(id(W, 1, 1)).unwrap();
    let unknown = id(W, 9, 9);
    assert_eq!(
        b.report(MatchReport {
            match_id: unknown,
            outcome: Outcome::Walkover { winner: p(1) }
        }),
        Err(EngineError::UnknownMatch(unknown))
    );
    let final_ = *b.get(id(W, 2, 1)).unwrap();
    assert_eq!(
        b.report(played(&final_, p(1))),
        Err(EngineError::MatchNotReady(final_.id))
    );
    assert_eq!(
        b.report(played(&m1, p(2))),
        Err(EngineError::NotInMatch {
            match_id: m1.id,
            participant: p(2)
        })
    );
    let bad = SeriesScore {
        winner: 1,
        loser: 0,
    };
    assert_eq!(
        b.report(MatchReport {
            match_id: m1.id,
            outcome: Outcome::Played {
                winner: p(1),
                score: bad
            }
        }),
        Err(EngineError::InvalidScore {
            best_of: 3,
            score: bad
        })
    );
    // erros não alteram o estado
    assert!(matches!(
        b.get(m1.id).unwrap().state,
        MatchState::Ready { .. }
    ));
    b.report(played(&m1, p(1))).unwrap();
    assert_eq!(
        b.report(played(&m1, p(1))),
        Err(EngineError::MatchAlreadyReported(m1.id))
    );
}

#[test]
fn report_on_bye_match_is_not_ready() {
    let mut b = EliminationBracket::new(single(3, 5), seeds(3)).unwrap();
    let bye = *b.get(id(W, 1, 1)).unwrap();
    assert_eq!(
        b.report(played(&bye, p(1))),
        Err(EngineError::MatchNotReady(bye.id))
    );
}

#[test]
fn walkover_skips_score_validation() {
    let mut b = EliminationBracket::new(single(3, 5), seeds(4)).unwrap();
    b.report(MatchReport {
        match_id: id(W, 1, 1),
        outcome: Outcome::Walkover { winner: p(4) },
    })
    .unwrap();
    assert_eq!(b.get(id(W, 2, 1)).unwrap().a, Slot::WinnerOf(id(W, 1, 1)));
    assert_eq!(
        b.standings(),
        vec![Placement {
            participant: p(1),
            rank: 3,
            tied: 2
        }]
    );
    assert_eq!(
        b.report(MatchReport {
            match_id: id(W, 1, 2),
            outcome: Outcome::Walkover { winner: p(9) }
        }),
        Err(EngineError::NotInMatch {
            match_id: id(W, 1, 2),
            participant: p(9)
        })
    );
}

#[test]
fn replay_matches_incremental_state() {
    for config in [single(3, 5), double(true), double(false)] {
        for n in [2i64, 3, 5, 8, 11] {
            let mut inc = EliminationBracket::new(config, seeds(n)).unwrap();
            let reports = play_favorites(&mut inc, false);
            let replayed = EliminationBracket::replay(config, seeds(n), reports).unwrap();
            assert_eq!(inc, replayed);
        }
    }
}

#[test]
fn replay_returns_error_of_contradicted_match() {
    let config = single(3, 5);
    let mut b = EliminationBracket::new(config, seeds(4)).unwrap();
    let reports = play_favorites(&mut b, false);
    // corrige o primeiro resultado: p(4) vence p(1); a semifinal reportada depois contradiz
    let m = *b.get(id(W, 1, 1)).unwrap();
    let mut fixed = reports.clone();
    fixed[0] = played(&m, p(4));
    let err = EliminationBracket::replay(config, seeds(4), fixed).unwrap_err();
    assert_eq!(
        err,
        EngineError::NotInMatch {
            match_id: id(W, 2, 1),
            participant: p(1)
        }
    );
}

#[test]
fn occupants_follow_the_results() {
    use Occupant::{Bye, Participant, Unknown};
    // 3 participantes: o seed 1 tem bye na primeira rodada.
    let mut b = EliminationBracket::new(single(3, 5), seeds(3)).unwrap();
    assert_eq!(b.occupants(id(W, 1, 1)), Some((Participant(p(1)), Bye)));
    assert_eq!(b.occupants(id(W, 2, 1)), Some((Participant(p(1)), Unknown)));
    let m = *b.get(id(W, 1, 2)).unwrap();
    b.report(played(&m, p(3))).unwrap();
    assert_eq!(
        b.occupants(id(W, 2, 1)),
        Some((Participant(p(1)), Participant(p(3))))
    );
    assert_eq!(b.occupants(id(W, 9, 9)), None);
}
