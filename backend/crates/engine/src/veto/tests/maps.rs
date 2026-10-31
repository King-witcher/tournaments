//! Veto de mapas, sorteio e modo desligado.

use super::*;

#[test]
fn best_of_three_bans_picks_and_decider() {
    let mut v = begun(rules(MapMode::Veto, 7, false, 0), 3, B);
    // O seed melhor (aqui o lado B) começa.
    assert_eq!(acting(&v), (Action::Ban, Target::Map, vec![B]));
    step(&mut v, B, map(1)).unwrap();
    assert_eq!(acting(&v).2, vec![A]);
    // Mapa já banido não volta.
    assert_eq!(step(&mut v, A, map(1)), Err(VetoError::OptionInvalid));
    // Fora do pool.
    assert_eq!(step(&mut v, A, map(9)), Err(VetoError::OptionInvalid));
    assert_eq!(step(&mut v, B, map(2)), Err(VetoError::NotYourTurn));
    step(&mut v, A, map(2)).unwrap();
    assert_eq!(acting(&v), (Action::Pick, Target::Map, vec![B]));
    step(&mut v, B, map(5)).unwrap();
    step(&mut v, A, map(3)).unwrap();
    assert_eq!(acting(&v).0, Action::Ban);
    step(&mut v, B, map(4)).unwrap();
    step(&mut v, A, map(7)).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert_eq!(
        [v.map_of(1), v.map_of(2), v.map_of(3)],
        [Some(MapId(5)), Some(MapId(3)), Some(MapId(6))]
    );
    assert_eq!(v.decider(), Some(MapId(6)));
    assert_eq!(
        (v.picker(1), v.picker(2), v.picker(3)),
        (Some(B), Some(A), None)
    );
}

#[test]
fn best_of_one_is_all_bans() {
    let mut v = begun(rules(MapMode::Veto, 7, false, 0), 1, A);
    for (i, m) in [1, 2, 3, 4, 5, 6].into_iter().enumerate() {
        let side = if i % 2 == 0 { A } else { B };
        assert_eq!(acting(&v).0, Action::Ban);
        step(&mut v, side, map(m)).unwrap();
    }
    assert_eq!(v.map_of(1), Some(MapId(7)));
    assert_eq!(v.turn(), Turn::Done { game: 1 });
}

#[test]
fn the_wrong_kind_of_step_is_refused() {
    let mut v = begun(rules(MapMode::Veto, 7, false, 0), 3, A);
    let pick = Event::Step {
        side: A,
        action: Action::Pick,
        choice: map(1),
    };
    assert_eq!(v.apply(pick, None), Err(VetoError::WrongAction));
    let champion = Event::Step {
        side: A,
        action: Action::Ban,
        choice: champ(1),
    };
    assert_eq!(v.apply(champion, None), Err(VetoError::OptionInvalid));
}

#[test]
fn not_begun_and_begun_twice() {
    let mut v = Veto::new(rules(MapMode::Veto, 7, true, 1), 3);
    assert_eq!(v.turn(), Turn::NotBegun);
    assert_eq!(step(&mut v, A, map(1)), Err(VetoError::NotBegun));
    assert_eq!(v.apply(Event::Undo, None), Err(VetoError::NotBegun));
    let game = Event::Game {
        game: 2,
        previous: None,
    };
    assert_eq!(v.apply(game, None), Err(VetoError::NotBegun));
    let begin = Event::Begin { seed: 1, first: A };
    v.apply(begin, None).unwrap();
    assert_eq!(v.apply(begin, None), Err(VetoError::AlreadyBegun));
}

#[test]
fn a_pool_smaller_than_the_series_cannot_begin() {
    let mut v = Veto::new(rules(MapMode::Veto, 3, false, 0), 5);
    assert_eq!(
        v.apply(Event::Begin { seed: 1, first: A }, None),
        Err(VetoError::PoolTooSmall {
            pool: 3,
            best_of: 5
        })
    );
    // Desligado não usa o pool.
    let mut v = Veto::new(rules(MapMode::Off, 0, true, 0), 5);
    assert!(v.apply(Event::Begin { seed: 1, first: A }, None).is_ok());
}

#[test]
fn rules_validate_the_pool() {
    assert!(rules(MapMode::Veto, 7, true, 1).validate(5).is_ok());
    assert_eq!(
        rules(MapMode::Vote, 4, true, 1).validate(5),
        Err(VetoError::PoolTooSmall {
            pool: 4,
            best_of: 5
        })
    );
    assert!(rules(MapMode::Off, 0, true, 1).validate(5).is_ok());
    let repeated = VetoRules {
        pool: vec![MapId(1), MapId(2), MapId(1)],
        ..rules(MapMode::Veto, 0, true, 1)
    };
    assert_eq!(repeated.validate(1), Err(VetoError::PoolRepeated(MapId(1))));
    assert!(!rules(MapMode::Off, 7, false, 0).has_phases());
    assert!(rules(MapMode::Off, 7, true, 0).has_phases());
}

#[test]
fn random_mode_draws_each_game_without_repeating() {
    let r = rules(MapMode::Random, 7, false, 0);
    let mut v = begun(r, 7, A);
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    let mut seen = vec![v.map_of(1).unwrap()];
    for game in 2..=7 {
        assert_eq!(v.map_of(game), None, "o jogo {game} ainda não começou");
        v.apply(
            Event::Game {
                game,
                previous: None,
            },
            None,
        )
        .unwrap();
        let m = v.map_of(game).unwrap();
        assert!(!seen.contains(&m), "{m:?} repetiu");
        seen.push(m);
    }
    seen.sort();
    assert_eq!(seen, pool(7));
    // Não há jogo depois do último.
    assert_eq!(
        v.apply(
            Event::Game {
                game: 8,
                previous: None
            },
            None
        ),
        Err(VetoError::GameOutOfOrder { next: 8 })
    );
}

#[test]
fn games_start_in_order() {
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 3, A);
    assert_eq!(
        v.apply(
            Event::Game {
                game: 3,
                previous: None
            },
            None
        ),
        Err(VetoError::GameOutOfOrder { next: 2 })
    );
}

#[test]
fn off_mode_has_no_map() {
    let v = begun(rules(MapMode::Off, 7, true, 0), 3, A);
    assert_eq!(v.map_of(1), None);
    assert_eq!(acting(&v).1, Target::Champion);
    let v = begun(rules(MapMode::Off, 7, false, 0), 3, A);
    assert_eq!(v.turn(), Turn::Done { game: 1 });
}
