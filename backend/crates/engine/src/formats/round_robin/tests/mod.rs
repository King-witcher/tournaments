//! Testes do round robin: rodadas, resultados e fim; propriedades em `props`.

mod props;

use std::collections::{HashMap, HashSet};

use super::*;
use crate::core::{BracketSide, MatchReport, MatchState, Outcome, SeriesFrags, SeriesScore, Slot};
use crate::series::wins_needed;

fn p(n: i64) -> ParticipantId {
    ParticipantId(n)
}

fn seeds(n: i64) -> Vec<ParticipantId> {
    (1..=n).map(p).collect()
}

fn id(round: u16, index: u16) -> MatchId {
    MatchId {
        side: BracketSide::RoundRobin,
        round,
        index,
    }
}

fn config() -> RoundRobinConfig {
    RoundRobinConfig { best_of: 3 }
}

fn played(m: &Match, winner: ParticipantId, loser_maps: u8) -> StageReport {
    StageReport {
        report: MatchReport {
            match_id: m.id,
            outcome: Outcome::Played {
                winner,
                score: SeriesScore {
                    winner: wins_needed(m.best_of),
                    loser: loser_maps,
                },
            },
        },
        frags: Some(SeriesFrags::default()),
    }
}

/// Vence o seed melhor. Devolve os resultados na ordem.
fn play_favorites(b: &mut RoundRobinBracket) -> Vec<StageReport> {
    let mut reports = Vec::new();
    let ready: Vec<Match> = b
        .matches()
        .iter()
        .filter(|m| matches!(m.state, MatchState::Ready { .. }))
        .copied()
        .collect();
    for m in ready {
        let MatchState::Ready { a, b: other } = m.state else {
            unreachable!()
        };
        let r = played(&m, a.min(other), 0);
        b.report(r).unwrap();
        reports.push(r);
    }
    reports
}

/// Os pares (a, b) de uma rodada e quem folga.
type Round = (Vec<(i64, i64)>, Option<i64>);

/// Cada rodada.
fn rounds(b: &RoundRobinBracket) -> Vec<Round> {
    let mut out: Vec<Round> = Vec::new();
    for m in b.matches() {
        let r = m.id.round as usize;
        if out.len() < r {
            out.push((Vec::new(), None));
        }
        match (m.a, m.b) {
            (Slot::Participant(a), Slot::Participant(b)) => out[r - 1].0.push((a.0, b.0)),
            (Slot::Participant(a), Slot::Bye) => out[r - 1].1 = Some(a.0),
            other => panic!("lado inesperado {other:?}"),
        }
    }
    out
}

#[test]
fn even_count_plays_n_minus_one_rounds_in_berger_order() {
    let b = RoundRobinBracket::new(config(), seeds(6)).unwrap();
    assert_eq!(b.rounds(), 5);
    let r = rounds(&b);
    assert_eq!(r.len(), 5);
    assert_eq!(r[0], (vec![(1, 6), (2, 5), (3, 4)], None));
    assert!(
        r.iter()
            .all(|(pairs, rest)| pairs.len() == 3 && rest.is_none())
    );
    assert_eq!(
        b.get(id(1, 1)).unwrap().state,
        MatchState::Ready { a: p(1), b: p(6) }
    );
    assert_eq!(b.get(id(1, 4)), None);
}

#[test]
fn odd_count_rests_once_each() {
    let b = RoundRobinBracket::new(config(), seeds(5)).unwrap();
    assert_eq!(b.rounds(), 5);
    let r = rounds(&b);
    let resting: Vec<i64> = r.iter().map(|(_, rest)| rest.unwrap()).collect();
    assert_eq!(resting, [1, 2, 3, 4, 5]);
    assert_eq!(r[0].0, vec![(2, 5), (3, 4)]);
    // A folga é a última partida da rodada, um bye sem jogo.
    let rest = b.get(id(1, 3)).unwrap();
    assert_eq!(
        rest.state,
        MatchState::Bye {
            advanced: Some(p(1))
        }
    );
    assert_eq!(
        b.occupants(id(1, 3)),
        Some((Occupant::Participant(p(1)), Occupant::Bye))
    );
}

#[test]
fn every_match_is_ready_from_the_start() {
    let b = RoundRobinBracket::new(config(), seeds(7)).unwrap();
    let ready = b
        .matches()
        .iter()
        .filter(|m| matches!(m.state, MatchState::Ready { .. }))
        .count();
    assert_eq!(ready, 7 * 6 / 2);
    assert!(b.matches().iter().all(|m| m.best_of == 3));
    // Lado `a` é o seed melhor.
    for m in b.matches() {
        if let MatchState::Ready { a, b } = m.state {
            assert!(a < b, "{:?}", m.id);
        }
    }
}

#[test]
fn limits_and_config() {
    assert_eq!(
        RoundRobinBracket::new(config(), seeds(1)),
        Err(EngineError::NotEnoughParticipants { min: 2, got: 1 })
    );
    assert_eq!(
        RoundRobinBracket::new(config(), seeds(33)),
        Err(EngineError::TooManyParticipants { max: 32, got: 33 })
    );
    assert_eq!(
        RoundRobinBracket::new(config(), vec![p(1), p(2), p(1)]),
        Err(EngineError::DuplicateParticipant(p(1)))
    );
    assert_eq!(
        RoundRobinBracket::new(RoundRobinConfig { best_of: 2 }, seeds(4)),
        Err(EngineError::InvalidBestOf(2))
    );
    assert!(RoundRobinBracket::new(config(), seeds(32)).is_ok());
}

#[test]
fn worst_case_is_one_series_per_opponent() {
    assert_eq!(config().worst_case_games(8), Ok(21));
    assert_eq!(
        RoundRobinConfig { best_of: 5 }.worst_case_games(32),
        Ok(155)
    );
    let b = RoundRobinBracket::new(config(), seeds(8)).unwrap();
    assert_eq!(b.worst_case_games(), 21);
    assert_eq!(
        config().worst_case_games(33),
        Err(EngineError::TooManyParticipants { max: 32, got: 33 })
    );
}

#[test]
fn report_errors() {
    let mut b = RoundRobinBracket::new(config(), seeds(3)).unwrap();
    let rest = *b.get(id(1, 2)).unwrap();
    assert_eq!(
        b.report(played(&rest, p(1), 0)),
        Err(EngineError::MatchNotReady(rest.id))
    );
    let m = *b.get(id(1, 1)).unwrap();
    assert_eq!(
        b.report(played(&m, p(1), 0)),
        Err(EngineError::NotInMatch {
            match_id: m.id,
            participant: p(1)
        })
    );
    let unknown = id(9, 1);
    let mut ghost = m;
    ghost.id = unknown;
    assert_eq!(
        b.report(played(&ghost, p(2), 0)),
        Err(EngineError::UnknownMatch(unknown))
    );
    assert_eq!(
        b.report(played(&m, p(2), 2)),
        Err(EngineError::InvalidScore {
            best_of: 3,
            score: SeriesScore {
                winner: 2,
                loser: 2
            }
        })
    );
    b.report(played(&m, p(3), 1)).unwrap();
    assert_eq!(
        b.report(played(&m, p(2), 0)),
        Err(EngineError::MatchAlreadyReported(m.id))
    );
}

#[test]
fn finishes_with_everyone_placed() {
    let mut b = RoundRobinBracket::new(config(), seeds(5)).unwrap();
    assert!(!b.is_finished());
    assert!(b.placements().is_empty());
    play_favorites(&mut b);
    assert!(b.is_finished());
    let placements: Vec<(i64, u16, u16)> = b
        .placements()
        .iter()
        .map(|x| (x.participant.0, x.rank, x.tied))
        .collect();
    assert_eq!(
        placements,
        [(1, 1, 1), (2, 2, 1), (3, 3, 1), (4, 4, 1), (5, 5, 1)]
    );
    let points: Vec<u16> = b.table().iter().map(|r| r.points).collect();
    assert_eq!(points, [4, 3, 2, 1, 0]);
}

#[test]
fn walkover_is_a_clean_win() {
    let mut b = RoundRobinBracket::new(config(), seeds(2)).unwrap();
    b.report(StageReport::from(MatchReport {
        match_id: id(1, 1),
        outcome: Outcome::Walkover { winner: p(2) },
    }))
    .unwrap();
    assert!(b.is_finished());
    let top = b.table()[0];
    let maps = (top.maps_won, top.maps_lost);
    assert_eq!((top.participant, top.points, maps), (p(2), 1, (2, 0)));
}

#[test]
fn correction_changes_the_table_only() {
    let mut b = RoundRobinBracket::new(config(), seeds(4)).unwrap();
    let reports = play_favorites(&mut b);
    // Corrige 1 x 4 (rodada 1): 4 venceu.
    let mut fixed = reports.clone();
    let first = b.get(id(1, 1)).unwrap();
    fixed[0] = played(first, p(4), 1);
    let corrected = RoundRobinBracket::replay(config(), seeds(4), fixed).unwrap();
    let ids = |x: &RoundRobinBracket| x.matches().iter().map(|m| m.id).collect::<Vec<_>>();
    assert_eq!(ids(&corrected), ids(&b));
    let points: HashMap<i64, u16> = corrected
        .table()
        .iter()
        .map(|r| (r.participant.0, r.points))
        .collect();
    assert_eq!((points[&1], points[&4]), (2, 1));
    assert_ne!(corrected.table(), b.table());
}

#[test]
fn replay_matches_incremental_state() {
    for n in [2i64, 3, 6, 9] {
        let mut b = RoundRobinBracket::new(config(), seeds(n)).unwrap();
        let reports = play_favorites(&mut b);
        assert_eq!(
            RoundRobinBracket::replay(config(), seeds(n), reports),
            Ok(b)
        );
    }
}

/// Todo par se enfrenta uma vez e ninguém joga duas vezes na rodada.
fn check_schedule(b: &RoundRobinBracket) {
    let n = b.participants().len();
    let mut pairs = HashSet::new();
    let mut rests: HashMap<i64, usize> = HashMap::new();
    for (round, (games, rest)) in rounds(b).iter().enumerate() {
        let mut seen = HashSet::new();
        for &(a, c) in games {
            assert!(a < c, "rodada {round}: {a} x {c}");
            assert!(seen.insert(a) && seen.insert(c), "rodada {round}: repetido");
            assert!(pairs.insert((a, c)), "{a} x {c} de novo");
        }
        if let Some(r) = rest {
            assert!(seen.insert(*r));
            *rests.entry(*r).or_default() += 1;
        }
        assert_eq!(seen.len(), n, "rodada {round}: falta alguém");
    }
    assert_eq!(pairs.len(), n * (n - 1) / 2);
    if n % 2 == 1 {
        assert_eq!(rests.len(), n);
        assert!(rests.values().all(|&c| c == 1));
    } else {
        assert!(rests.is_empty());
    }
}

#[test]
fn every_size_pairs_everyone_once() {
    for n in 2..=MAX_PARTICIPANTS as i64 {
        check_schedule(&RoundRobinBracket::new(config(), seeds(n)).unwrap());
    }
}
