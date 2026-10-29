//! Ban/pick de campeões e a regra de não repetição.

use super::*;

/// md3 em veto com 1 ban de campeão; o seed melhor é A. O veto de mapas fica
/// completo: A escolhe o mapa 1, B o 2, e o 3 é o decisivo.
fn after_map_veto(bans: u8) -> Veto {
    let mut v = begun(rules(MapMode::Veto, 7, true, bans), 3, A);
    for (side, m) in [(A, 1), (B, 2), (A, 3), (B, 4), (A, 5), (B, 6)] {
        step(&mut v, side, map(m)).unwrap();
    }
    v
}

#[test]
fn the_side_that_did_not_pick_the_map_starts() {
    let mut v = after_map_veto(1);
    // A escolheu o mapa 1: B começa os campeões.
    assert_eq!(v.starter(1), Some(B));
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![B]));
    step(&mut v, B, champ(1)).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    // O ban vale para os dois.
    assert_eq!(step(&mut v, A, champ(1)), Err(VetoError::OptionInvalid));
    step(&mut v, A, champ(2)).unwrap();
    assert_eq!(acting(&v), (Action::Pick, Target::Champion, vec![B]));
    assert_eq!(step(&mut v, B, champ(2)), Err(VetoError::OptionInvalid));
    step(&mut v, B, champ(3)).unwrap();
    // Espelho vale: os dois podem jogar com o mesmo.
    step(&mut v, A, champ(3)).unwrap();
    assert_eq!(v.picks(1), [Some(ChampionId(103)), Some(ChampionId(103))]);
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    // B escolheu o mapa 2: A começa; o decisivo, quem começou o veto.
    assert_eq!(v.next_starter(B), A);
    v.apply(
        Event::Game {
            game: 2,
            starter: A,
            previous: None,
        },
        None,
    )
    .unwrap();
    assert_eq!(v.starter(2), Some(A));
    for (side, c) in [(A, 1), (B, 2), (A, 4), (B, 5)] {
        step(&mut v, side, champ(c)).unwrap();
    }
    assert_eq!(v.next_starter(B), A);
}

#[test]
fn bans_repeat_per_ban_round() {
    let v = after_map_veto(3);
    let champions = champions(17);
    let mut v = v;
    let ctx = free(&champions);
    for i in 0..6 {
        let (action, _, sides) = acting(&v);
        assert_eq!(action, Action::Ban, "passo {i}");
        assert_eq!(sides, vec![if i % 2 == 0 { B } else { A }]);
        step_in(&mut v, sides[0], champ(i + 1), &ctx).unwrap();
    }
    assert_eq!(acting(&v).0, Action::Pick);
    assert_eq!(v.options(B, &ctx).len(), 11);
    assert!(v.options(A, &ctx).is_empty(), "não é a vez de A");
}

#[test]
fn outside_the_veto_the_better_seed_then_the_loser_starts() {
    let mut v = begun(rules(MapMode::Vote, 7, true, 0), 3, B);
    let offered = v.offered(1);
    for side in [A, B] {
        step(&mut v, side, Choice::Map(offered[0])).unwrap();
    }
    assert_eq!(v.starter(1), Some(B));
    assert_eq!(acting(&v).2, vec![B]);
    step(&mut v, B, champ(1)).unwrap();
    step(&mut v, A, champ(2)).unwrap();
    // Quem perdeu o jogo 1 (A) começa o 2.
    assert_eq!(v.next_starter(A), A);
    v.apply(
        Event::Game {
            game: 2,
            starter: A,
            previous: None,
        },
        None,
    )
    .unwrap();
    assert_eq!(v.starter(2), Some(A));
}

#[test]
fn no_repeat_excludes_what_the_side_already_used() {
    let champions = champions(5);
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 3, A);
    let elsewhere = [ChampionId(105)];
    let ctx = ctx(&champions, [&elsewhere, &[]]);
    // O 5 já foi usado por A em outra partida do escopo.
    assert_eq!(
        step_in(&mut v, A, champ(5), &ctx),
        Err(VetoError::ChampionForbidden)
    );
    assert_eq!(v.options(A, &ctx).len(), 4);
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    step_in(&mut v, B, champ(5), &ctx).unwrap();
    v.apply(
        Event::Game {
            game: 2,
            starter: B,
            previous: None,
        },
        None,
    )
    .unwrap();
    // No jogo 2, cada um perde o que escolheu no 1.
    assert_eq!(
        step_in(&mut v, B, champ(5), &ctx),
        Err(VetoError::ChampionForbidden)
    );
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    let options = v.options(A, &ctx);
    assert_eq!(options, [champ(2), champ(3), champ(4)]);
    // A regra desligada não tira nada.
    assert_eq!(v.options(A, &free(&champions)).len(), 5);
    // Fora do pool.
    assert_eq!(
        step_in(&mut v, A, champ(9), &ctx),
        Err(VetoError::OptionInvalid)
    );
}

#[test]
fn the_rule_gives_way_when_nothing_is_left() {
    let champions = champions(3);
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 5, A);
    let used = [ChampionId(101), ChampionId(102), ChampionId(103)];
    let ctx = ctx(&champions, [&used, &[]]);
    // A usou os três: a regra cede para o veto não travar.
    assert_eq!(v.options(A, &ctx).len(), 3);
    assert!(v.random_choice(A, &ctx).is_some());
    step_in(&mut v, A, champ(2), &ctx).unwrap();
}

#[test]
fn a_ban_cannot_leave_the_banner_without_a_pick() {
    // Pool de 6, 2 bans de cada lado. A já usou 101..=103 (fora da série):
    // pode escolher 104, 105 e 106. B ainda tem 2 bans; se A banisse um dos
    // três, B poderia tirar os outros dois.
    let champions = champions(6);
    let used = [ChampionId(101), ChampionId(102), ChampionId(103)];
    let ctx = ctx(&champions, [&used, &[]]);
    let mut v = begun(rules(MapMode::Off, 0, true, 2), 3, A);
    let options = v.options(A, &ctx);
    assert_eq!(options, [champ(1), champ(2), champ(3)]);
    assert_eq!(
        step_in(&mut v, A, champ(4), &ctx),
        Err(VetoError::OptionInvalid)
    );
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    // B bane um dos de A; os bans de B que sobram são 1.
    step_in(&mut v, B, champ(4), &ctx).unwrap();
    assert_eq!(v.options(A, &ctx), [champ(2), champ(3)]);
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    step_in(&mut v, B, champ(5), &ctx).unwrap();
    // Sobrou um para A.
    assert_eq!(v.options(A, &ctx), [champ(6)]);
}

#[test]
fn a_timeout_picks_at_random_within_the_rule() {
    let champions = champions(17);
    let used = [ChampionId(101), ChampionId(102)];
    let ctx = ctx(&champions, [&used, &[]]);
    for seed in 0..30 {
        let mut v = Veto::replay(
            rules(MapMode::Off, 0, true, 1),
            3,
            [Event::Begin { seed, first: A }],
        )
        .unwrap();
        while let Turn::Act { sides, .. } = v.turn() {
            let choice = v.random_choice(sides[0], &ctx).unwrap();
            assert_eq!(v.random_choice(sides[0], &ctx), Some(choice));
            step_in(&mut v, sides[0], choice, &ctx).unwrap();
        }
        let [Some(a), _] = v.picks(1) else {
            panic!("sem pick de A")
        };
        assert!(!used.contains(&a));
    }
}
