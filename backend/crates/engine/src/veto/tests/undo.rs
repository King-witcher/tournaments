//! Desfazer o último passo e refazer pelo histórico.

use super::*;

#[test]
fn undo_goes_back_one_step_of_the_current_game() {
    let mut v = begun(rules(MapMode::Veto, 7, true, 1), 3, A);
    assert!(!v.can_undo());
    assert_eq!(v.apply(Event::Undo, None), Err(VetoError::NothingToUndo));
    step(&mut v, A, map(1)).unwrap();
    step(&mut v, B, map(2)).unwrap();
    v.apply(Event::Undo, None).unwrap();
    // A vez volta para quem tinha feito o passo, e o mapa volta ao pool.
    assert_eq!(acting(&v), (Action::Ban, Target::Map, vec![B]));
    assert!(v.remaining_maps().contains(&MapId(2)));
    step(&mut v, B, map(3)).unwrap();
    assert_eq!(v.map_steps().len(), 2);
}

#[test]
fn undo_crosses_from_champions_back_to_the_map_veto() {
    let mut v = begun(rules(MapMode::Veto, 7, true, 0), 1, A);
    for (side, m) in [(A, 1), (B, 2), (A, 3), (B, 4), (A, 5), (B, 6)] {
        step(&mut v, side, map(m)).unwrap();
    }
    assert_eq!(acting(&v).1, Target::Champion);
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Map, vec![B]));
    assert_eq!(v.decider(), None);
}

#[test]
fn a_finished_game_does_not_undo() {
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 3, A);
    step(&mut v, A, champ(1)).unwrap();
    step(&mut v, B, champ(2)).unwrap();
    v.apply(
        Event::Game {
            game: 2,
            starter: B,
            previous: None,
        },
        None,
    )
    .unwrap();
    assert!(!v.can_undo());
    assert_eq!(v.apply(Event::Undo, None), Err(VetoError::NothingToUndo));
    assert_eq!(v.picks(1), [Some(ChampionId(101)), Some(ChampionId(102))]);
}

#[test]
fn undoing_a_timeout_and_timing_out_again_draws_again() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 3, A);
    let first = v.random_choice(A, &ctx).unwrap();
    step_in(&mut v, A, first, &ctx).unwrap();
    v.apply(Event::Undo, None).unwrap();
    // A posição no histórico mudou: outro sorteio (que pode dar o mesmo).
    let again = v.random_choice(A, &ctx).unwrap();
    let replayed = Veto::replay(
        v.rules().clone(),
        3,
        [
            Event::Begin { seed: 42, first: A },
            Event::Step {
                side: A,
                action: Action::Pick,
                choice: first,
            },
            Event::Undo,
        ],
    )
    .unwrap();
    assert_eq!(replayed.random_choice(A, &ctx), Some(again));
}

#[test]
fn replay_rebuilds_the_same_state() {
    let events = [
        Event::Begin { seed: 9, first: B },
        Event::Step {
            side: B,
            action: Action::Ban,
            choice: map(4),
        },
        Event::Step {
            side: A,
            action: Action::Ban,
            choice: map(5),
        },
        Event::Undo,
        Event::Step {
            side: A,
            action: Action::Ban,
            choice: map(6),
        },
    ];
    let v = Veto::replay(rules(MapMode::Veto, 7, true, 1), 3, events).unwrap();
    assert_eq!(v.remaining_maps(), [1, 2, 3, 5, 7].map(MapId));
    assert_eq!(acting(&v), (Action::Pick, Target::Map, vec![B]));
    // Um histórico que não fecha é recusado.
    let bad = [
        Event::Begin { seed: 9, first: B },
        Event::Step {
            side: A,
            action: Action::Ban,
            choice: map(4),
        },
    ];
    assert_eq!(
        Veto::replay(rules(MapMode::Veto, 7, true, 1), 3, bad),
        Err(VetoError::NotYourTurn)
    );
}
