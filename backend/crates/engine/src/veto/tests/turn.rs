//! O evento que abriu a vez de agora (`turn_opened`).

use super::*;

#[test]
fn each_turn_has_its_own_number_even_for_the_same_side_twice() {
    let champions = champions(8);
    let ctx = free(&champions);
    let mut v = Veto::new(rules(MapMode::Vote, 7, true, 1), 3);
    assert_eq!(v.turn_opened(), 0);
    v.apply(Event::Begin { seed: 7, first: A }, None).unwrap();
    assert_eq!(v.turn_opened(), 1);
    // O primeiro voto espera o do outro: a vez é a mesma para os dois.
    let offered = v.offered(1);
    step(&mut v, B, Choice::Map(offered[0])).unwrap();
    assert_eq!(v.turn_opened(), 1);
    step(&mut v, A, Choice::Map(offered[1])).unwrap();
    assert_eq!(v.turn_opened(), 3);
    // Bans (B, A) e a partida 1 (A, B); na partida 2, B escolhe primeiro.
    for (side, c) in [(B, 1), (A, 2), (A, 3)] {
        step_in(&mut v, side, champ(c), &ctx).unwrap();
    }
    assert_eq!(picking(&v), (B, 1));
    let first = v.turn_opened();
    step_in(&mut v, B, champ(4), &ctx).unwrap();
    assert_eq!(picking(&v), (B, 2));
    assert_eq!(v.turn_opened(), first + 1);
    // Desfazer abre outra vez, mesmo de volta ao mesmo passo.
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(picking(&v), (B, 1));
    assert_eq!(v.turn_opened(), first + 2);
}

#[test]
fn the_simultaneous_window_is_a_single_turn() {
    let champions = champions(9);
    let ctx = free(&champions);
    let mut v = two_v_two(team_rules(0, 2, false), 1);
    let window = v.turn_opened();
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    pick(&mut v, B, P3, 1, 2, &ctx).unwrap();
    assert_eq!(v.turn_opened(), window);
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(v.turn_opened(), window + 3);
}
