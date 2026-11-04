//! Os picks alternados: por jogador nos times (O, X, X, O..., quem abre troca a
//! cada partida, qualquer um do lado da vez que ainda não escolheu), um passo
//! por pick, o sorteio do primeiro do lado no fim do prazo; e, com um jogador
//! por lado, o mesmo draft do duelo de sempre.

use super::*;

fn alternating(bans: u8, size: u8) -> VetoRules {
    ordered(team_rules(bans, size, false), PickOrder::Alternating)
}

/// A vez dos picks por jogador: o lado e a partida.
fn turn(v: &Veto) -> (Side, u8) {
    match v.turn() {
        Turn::Act {
            action: Action::TeamPick,
            sides,
            pick_for: Some(game),
            ..
        } => (sides[0], game),
        other => panic!("esperava um pick alternado, veio {other:?}"),
    }
}

/// Elenco de `size` em cada lado: A com 1, 2..., B com 11, 12...
fn rosters(size: i64) -> Rosters {
    let a: Vec<PlayerId> = (1..=size).map(PlayerId).collect();
    let b: Vec<PlayerId> = (11..=10 + size).map(PlayerId).collect();
    open(&a, &b)
}

/// Faz cada pick com o primeiro que pode e devolve os lados, na ordem.
fn play(v: &mut Veto, ctx: &Context) -> Vec<(Side, u8)> {
    let mut seen = Vec::new();
    let mut next = 1;
    while matches!(v.turn(), Turn::Act { .. }) {
        seen.push(turn(v));
        let m = v.due_picks()[0];
        pick(v, m.side, m.player, m.game, next, ctx).unwrap();
        next += 1;
    }
    seen
}

#[test]
fn the_order_by_team_size_alternates_who_opens_each_game() {
    let champions = champions(40);
    let ctx = free(&champions);
    let expected: [&[Side]; 3] = [
        &[A, B, B, A],
        &[A, B, B, A, A, B],
        &[A, B, B, A, A, B, B, A],
    ];
    for (size, order) in (2u8..).zip(expected) {
        let mut v = begun(alternating(0, size), 3, A);
        v.set_rosters(rosters(i64::from(size))).unwrap();
        let flipped = |s: &Side| if *s == A { B } else { A };
        let want: Vec<(Side, u8)> = (1u8..=3)
            .flat_map(|game| {
                order.iter().map(move |side| {
                    let side = if game % 2 == 1 { *side } else { flipped(side) };
                    (side, game)
                })
            })
            .collect();
        assert_eq!(play(&mut v, &ctx), want, "times de {size}");
    }
}

#[test]
fn the_better_seed_opens_odd_games_whichever_side_it_is() {
    let mut v = begun(alternating(0, 2), 3, B);
    v.set_rosters(rosters(2)).unwrap();
    assert_eq!(turn(&v), (B, 1));
    let champions = champions(20);
    let ctx = free(&champions);
    let seen = play(&mut v, &ctx);
    assert_eq!(seen[0], (B, 1));
    assert_eq!(seen[3], (B, 1));
    assert_eq!(seen[4], (A, 2));
    assert_eq!(seen[8], (B, 3));
}

#[test]
fn with_one_player_a_side_it_is_the_duel_of_always() {
    // A mesma sequência, os mesmos eventos e o mesmo sorteio do duelo.
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = begun(rules(MapMode::Off, 0, true, 1), 3, A);
    let mut seen = Vec::new();
    loop {
        match v.turn() {
            Turn::Act {
                action,
                sides,
                pick_for,
                ..
            } => {
                seen.push((action, sides[0], pick_for));
                let choice = v.random_choice(sides[0], &ctx).unwrap();
                step_in(&mut v, sides[0], choice, &ctx).unwrap();
            }
            Turn::Done { .. } => break,
            other => panic!("vez inesperada: {other:?}"),
        }
    }
    use Action::{Ban, Pick};
    assert_eq!(
        seen,
        [
            (Ban, B, None),
            (Ban, A, None),
            (Pick, A, Some(1)),
            (Pick, B, Some(1)),
            (Pick, B, Some(2)),
            (Pick, A, Some(2)),
            (Pick, A, Some(3)),
            (Pick, B, Some(3)),
        ]
    );
    assert!(v.missing().is_empty());
    assert!(v.team_picks(1).is_empty());
    assert!(v.picks(1)[0].is_some() && v.picks(1)[1].is_some());
    // Duelo não tem elenco: um histórico antigo se refaz sem ele até o fim.
    assert!(matches!(v.turn(), Turn::Done { game: 1 }));
}

#[test]
fn anyone_on_the_side_picks_their_own_once() {
    let champions = champions(20);
    let ctx = free(&champions);
    let mut v = two_v_two(alternating(0, 2), 3);
    assert_eq!(turn(&v), (A, 1));
    assert_eq!(
        v.pickable()
            .iter()
            .map(|m| (m.player, m.game))
            .collect::<Vec<_>>(),
        [(P1, 1), (P2, 1)]
    );
    // O outro lado espera; a outra partida também.
    assert_eq!(pick(&mut v, B, P3, 1, 1, &ctx), Err(VetoError::NotYourTurn));
    assert_eq!(pick(&mut v, A, P1, 2, 1, &ctx), Err(VetoError::NotYourTurn));
    // Quem é do lado e não do elenco, não.
    assert_eq!(pick(&mut v, A, P5, 1, 1, &ctx), Err(VetoError::NotYourTurn));
    // Qualquer um do lado: aqui, o segundo.
    pick(&mut v, A, P2, 1, 1, &ctx).unwrap();
    // Agora é de B, duas vezes.
    assert_eq!(turn(&v), (B, 1));
    pick(&mut v, B, P4, 1, 2, &ctx).unwrap();
    assert_eq!(turn(&v), (B, 1));
    assert_eq!(
        pick(&mut v, B, P4, 1, 3, &ctx),
        Err(VetoError::AlreadyPicked)
    );
    pick(&mut v, B, P3, 1, 3, &ctx).unwrap();
    // A volta: só falta P1, e P2 já escolheu nesta partida.
    assert_eq!(turn(&v), (A, 1));
    assert_eq!(
        pick(&mut v, A, P2, 1, 4, &ctx),
        Err(VetoError::AlreadyPicked)
    );
    assert_eq!(
        v.pickable().iter().map(|m| m.player).collect::<Vec<_>>(),
        [P1]
    );
    assert!(v.team_options(A, P2, 1, &ctx).champions.is_empty());
    assert!(!v.team_options(A, P1, 1, &ctx).champions.is_empty());
    pick(&mut v, A, P1, 1, 4, &ctx).unwrap();
    // Na partida 2 quem abre é B.
    assert_eq!(turn(&v), (B, 2));
}

#[test]
fn each_pick_opens_a_new_turn() {
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = two_v_two(alternating(0, 2), 1);
    let first = v.turn_opened();
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    assert_eq!(v.turn_opened(), first + 1);
    pick(&mut v, B, P3, 1, 2, &ctx).unwrap();
    assert_eq!(v.turn_opened(), first + 2);
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(v.turn_opened(), first + 3);
    assert_eq!(turn(&v), (B, 1));
}

#[test]
fn the_deadline_draws_only_for_the_first_of_the_side_in_roster_order() {
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = two_v_two(alternating(0, 2), 3);
    assert_eq!(
        v.due_picks(),
        [Missing {
            side: A,
            player: P1,
            game: 1
        }]
    );
    // O primeiro do lado escolheu antes: o sorteio é do colega.
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    pick(&mut v, B, P3, 1, 2, &ctx).unwrap();
    assert_eq!(
        v.due_picks(),
        [Missing {
            side: B,
            player: P4,
            game: 1
        }]
    );
    let drawn = v.random_team_choice(B, P4, 1, &ctx).unwrap();
    assert!(v.team_options(B, P4, 1, &ctx).champions.contains(&drawn));
    // Nada para quem não é da vez, nem para a partida seguinte.
    assert_eq!(v.random_team_choice(A, P2, 1, &ctx), None);
    assert_eq!(v.random_team_choice(B, P4, 2, &ctx), None);
    // O mesmo histórico sorteia o mesmo.
    assert_eq!(v.clone().random_team_choice(B, P4, 1, &ctx), Some(drawn));
}

#[test]
fn the_draw_gives_way_like_the_others() {
    // Pela regra do jogador, com o pool no limite: o último fica sem opção
    // pela regra, e ela cede.
    let champions = champions(2);
    let ctx = free(&champions);
    let rules = with_rule(alternating(0, 2), crate::champions::NoRepeatWho::Player);
    let mut v = two_v_two(rules, 3);
    let mut gave_way = false;
    while matches!(v.turn(), Turn::Act { .. }) {
        let m = v.due_picks()[0];
        let options = v.team_options(m.side, m.player, m.game, &ctx);
        gave_way |= options.gave_way.is_some();
        let c = v
            .random_team_choice(m.side, m.player, m.game, &ctx)
            .unwrap();
        v.apply(
            Event::TeamPick {
                side: m.side,
                player: m.player,
                game: m.game,
                champion: c,
                gave_way: options.gave_way.is_some(),
            },
            Some(&ctx),
        )
        .unwrap();
    }
    assert!(gave_way);
    assert!(matches!(v.turn(), Turn::Done { game: 1 }));
}

#[test]
fn without_a_mirror_the_first_to_pick_keeps_the_champion() {
    let champions = champions(8);
    let ctx = free(&champions);
    let mut v = two_v_two(no_mirror(alternating(0, 2)), 1);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    assert_eq!(
        pick(&mut v, B, P3, 1, 1, &ctx),
        Err(VetoError::ChampionTaken)
    );
    let picker = Picker {
        side: B,
        player: Some(P3),
    };
    assert_eq!(
        v.unavailable(picker, 1, &ctx),
        [(ChampionId(101), Unavailable::Opponent)]
    );
}

#[test]
fn a_short_side_gives_up_the_positions_it_cannot_fill() {
    let champions = champions(12);
    let ctx = free(&champions);
    // A só com um jogador num time de 2: A, B, (B), A vira A, B, B.
    let mut v = begun(alternating(0, 2), 1, A);
    v.set_rosters(open(&[P1], &[P3, P4])).unwrap();
    assert_eq!(turn(&v), (A, 1));
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    assert_eq!(turn(&v), (B, 1));
    pick(&mut v, B, P3, 1, 2, &ctx).unwrap();
    assert_eq!(turn(&v), (B, 1));
    pick(&mut v, B, P4, 1, 3, &ctx).unwrap();
    assert!(matches!(v.turn(), Turn::Done { game: 1 }));
    // Um lado sem ninguém: só o outro escolhe.
    let mut v = begun(alternating(0, 2), 1, A);
    v.set_rosters(open(&[], &[P3, P4])).unwrap();
    assert_eq!(turn(&v), (B, 1));
    pick(&mut v, B, P4, 1, 1, &ctx).unwrap();
    assert_eq!(turn(&v), (B, 1));
}

#[test]
fn a_substitute_takes_the_position_of_who_left() {
    let champions = champions(20);
    let ctx = free(&champions);
    let mut v = two_v_two(alternating(0, 2), 1);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    pick(&mut v, B, P3, 1, 2, &ctx).unwrap();
    // P1 sai, com o pick dele, e P5 entra: A volta a dever a posição dele.
    v.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    assert_eq!(turn(&v), (A, 1));
    assert_eq!(pick(&mut v, B, P4, 1, 3, &ctx), Err(VetoError::NotYourTurn));
    assert_eq!(v.due_picks()[0].player, P5);
    pick(&mut v, A, P5, 1, 1, &ctx).unwrap();
    assert_eq!(picks_of(&v, 1, A), ids(&[1]));
    // Dois de B por fazer um: B, depois A.
    assert_eq!(turn(&v), (B, 1));
    pick(&mut v, B, P4, 1, 3, &ctx).unwrap();
    assert_eq!(turn(&v), (A, 1));
}

#[test]
fn undo_gives_the_turn_back_to_who_picked() {
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = two_v_two(alternating(0, 2), 1);
    pick(&mut v, A, P2, 1, 1, &ctx).unwrap();
    assert!(v.can_undo());
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(turn(&v), (A, 1));
    assert!(picks_of(&v, 1, A).is_empty());
    assert_eq!(
        v.pickable().iter().map(|m| m.player).collect::<Vec<_>>(),
        [P1, P2]
    );
}

#[test]
fn the_draft_never_stays_half_done() {
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = two_v_two(alternating(0, 2), 3);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    assert_eq!(
        v.clone().apply(
            Event::Game {
                game: 2,
                previous: None
            },
            None
        ),
        Err(VetoError::DraftPending)
    );
}

#[test]
fn a_replay_without_the_roster_applies_the_picks_and_gives_the_same_veto() {
    let champions = champions(20);
    let ctx = free(&champions);
    let rules = alternating(1, 2);
    let mut v = two_v_two(rules.clone(), 3);
    let mut events = vec![Event::Begin { seed: 42, first: A }];
    while let Turn::Act { action, sides, .. } = v.turn() {
        let e = if action == Action::Ban {
            Event::Step {
                side: sides[0],
                action,
                choice: v.random_choice(sides[0], &ctx).unwrap(),
            }
        } else {
            let m = v.due_picks()[0];
            let c = v
                .random_team_choice(m.side, m.player, m.game, &ctx)
                .unwrap();
            Event::TeamPick {
                side: m.side,
                player: m.player,
                game: m.game,
                champion: c,
                gave_way: false,
            }
        };
        v.apply(e, Some(&ctx)).unwrap();
        events.push(e);
    }
    let mut replayed = Veto::replay(rules, 3, events).unwrap();
    replayed.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    assert_eq!(replayed, v);
}
