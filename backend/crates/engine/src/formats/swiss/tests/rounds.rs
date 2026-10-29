//! Configuração, rodadas e pareamentos.

use super::*;

#[test]
fn rounds_default_to_log2_and_stop_at_n_minus_one() {
    let auto = config(None);
    let cases = [
        (2, 1),
        (3, 2),
        (4, 2),
        (5, 3),
        (8, 3),
        (9, 4),
        (16, 4),
        (17, 5),
        (128, 7),
    ];
    for (n, rounds) in cases {
        assert_eq!(auto.total_rounds(n), rounds, "{n}");
    }
    assert_eq!(config(Some(10)).total_rounds(4), 3);
    assert_eq!(config(Some(2)).total_rounds(8), 2);
    assert_eq!(config(Some(5)).total_rounds(2), 1);
    // Não diminui com mais participantes.
    for r in [None, Some(1), Some(6), Some(127)] {
        let c = config(r);
        assert!((2..200).all(|n| c.total_rounds(n) <= c.total_rounds(n + 1)));
    }
}

#[test]
fn config_is_validated() {
    assert_eq!(config(Some(1)).validate(), Ok(()));
    assert_eq!(config(Some(127)).validate(), Ok(()));
    for got in [0, 128] {
        assert_eq!(
            config(Some(got)).validate(),
            Err(EngineError::InvalidRounds { max: 127, got })
        );
    }
    let even = SwissConfig {
        best_of: 4,
        rounds: None,
    };
    assert_eq!(even.validate(), Err(EngineError::InvalidBestOf(4)));
    assert_eq!(
        SwissBracket::new(config(None), seeds(129)),
        Err(EngineError::TooManyParticipants { max: 128, got: 129 })
    );
    assert_eq!(
        SwissBracket::new(config(None), seeds(1)),
        Err(EngineError::NotEnoughParticipants { min: 2, got: 1 })
    );
}

#[test]
fn worst_case_is_one_series_per_round() {
    assert_eq!(config(None).worst_case_games(8), Ok(9));
    assert_eq!(config(Some(5)).worst_case_games(4), Ok(9));
    assert_eq!(
        config(None).worst_case_games(129),
        Err(EngineError::TooManyParticipants { max: 128, got: 129 })
    );
}

#[test]
fn first_round_splits_the_seeds_in_halves() {
    let b = SwissBracket::new(config(None), seeds(8)).unwrap();
    assert!(b.matches().is_empty());
    let r = b.next_round().unwrap();
    assert_eq!(plain(&r), (vec![[1, 5], [2, 6], [3, 7], [4, 8]], None));
    let mut run = Run::new(8, None);
    run.pair();
    let b = run.bracket();
    assert_eq!(
        b.get(id(1, 2)).unwrap().state,
        MatchState::Ready { a: p(2), b: p(6) }
    );
    assert_eq!(b.matches().len(), 4);
}

#[test]
fn odd_count_gives_the_first_bye_to_the_last_seed() {
    let mut run = Run::new(7, None);
    let r = run.pair().unwrap();
    assert_eq!(plain(&r), (vec![[1, 4], [2, 5], [3, 6]], Some(7)));
    let b = run.bracket();
    let bye = b.get(id(1, 4)).unwrap();
    assert_eq!(
        bye.state,
        MatchState::Bye {
            advanced: Some(p(7))
        }
    );
    assert_eq!(
        b.occupants(id(1, 4)),
        Some((Occupant::Participant(p(7)), Occupant::Bye))
    );
    // O bye vale vitória já na rodada.
    let row = b.table().iter().find(|r| r.participant == p(7)).unwrap();
    assert_eq!((row.points, row.byes, row.maps_won), (1, 1, 0));
}

#[test]
fn next_round_waits_for_every_series() {
    let mut run = Run::new(4, None);
    run.pair();
    let b = run.bracket();
    assert_eq!(b.next_round(), None);
    assert!(!b.is_finished());
    let m = *b.get(id(1, 1)).unwrap();
    run.reports.push(played(&m, p(1), 1));
    assert_eq!(run.bracket().next_round(), None);
    run.decide(favorite);
    assert!(run.bracket().next_round().is_some());
}

#[test]
fn second_round_pairs_by_points() {
    let mut run = Run::new(8, None);
    run.pair();
    run.decide(favorite);
    let r = run.pair().unwrap();
    assert_eq!(plain(&r), (vec![[1, 3], [2, 4], [5, 7], [6, 8]], None));
}

#[test]
fn byes_go_to_the_bottom_without_repeating() {
    let mut run = Run::new(5, Some(4));
    run.play(favorite);
    let byes: Vec<i64> = run.rounds.iter().map(|r| r.bye.unwrap().0).collect();
    assert_eq!(byes[..2], [5, 4]);
    let distinct: HashSet<i64> = byes.iter().copied().collect();
    assert_eq!(distinct.len(), 4);
    assert!(run.bracket().is_finished());
}

#[test]
fn rematch_only_when_there_is_no_way_around() {
    // Rodadas 1 a 3: cada um de 1, 2 e 3 enfrentou 4, 5 e 6. Sem revanche,
    // sobram dois triângulos (1-2-3 e 4-5-6), que não pareiam.
    let mut run = Run::new(6, Some(5));
    run.rounds = vec![
        round(&[[1, 4], [2, 5], [3, 6]], None),
        round(&[[1, 5], [2, 6], [3, 4]], None),
        round(&[[1, 6], [2, 4], [3, 5]], None),
    ];
    run.decide(favorite);
    let r = run.pair().unwrap();
    assert_eq!(plain(&r), (vec![[1, 2], [3, 4], [5, 6]], None));
    // E a rodada 5 ainda sai, com o mínimo de revanches.
    run.decide(favorite);
    let fifth = run.pair().unwrap();
    assert_eq!(fifth.pairs.len(), 3);
}

#[test]
fn players_with_fewer_points_float_down() {
    // Depois de uma rodada com 6: 1, 2 e 3 com um ponto. O 3 desce e
    // enfrenta o melhor dos que não pontuaram.
    let mut run = Run::new(6, None);
    run.pair();
    run.decide(favorite);
    let r = run.pair().unwrap();
    assert_eq!(plain(&r), (vec![[1, 2], [3, 4], [5, 6]], None));
}
