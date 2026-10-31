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
fn the_draft_undoes_back_to_the_bans() {
    let mut v = begun(rules(MapMode::Off, 0, true, 1), 3, A);
    step(&mut v, B, champ(1)).unwrap();
    step(&mut v, A, champ(2)).unwrap();
    step(&mut v, A, champ(3)).unwrap();
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(picking(&v), (A, 1));
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    assert_eq!(v.bans().len(), 1);
}

#[test]
fn a_finished_game_does_not_undo() {
    // O draft inteiro fica com a partida 1: enviada ela, nada se desfaz.
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 3, A);
    for (side, c) in [(A, 1), (B, 2), (B, 3), (A, 4), (A, 5), (B, 6)] {
        step(&mut v, side, champ(c)).unwrap();
    }
    next_game(&mut v, 2);
    assert!(!v.can_undo());
    assert_eq!(v.apply(Event::Undo, None), Err(VetoError::NothingToUndo));
    assert_eq!(v.picks(1), [Some(ChampionId(101)), Some(ChampionId(102))]);
    assert_eq!(v.picks(3), [Some(ChampionId(105)), Some(ChampionId(106))]);
}

#[test]
fn game_two_waits_for_the_whole_draft() {
    // O organizador lançou a partida 1 no meio do draft: a 2 só começa
    // depois que o resto sai pelo sorteio do tempo esgotado (a API faz).
    let champions = champions(17);
    let ctx = free(&champions);
    let game = Event::Game {
        game: 2,
        previous: None,
    };
    let mut v = begun(rules(MapMode::Off, 0, true, 1), 3, A);
    step(&mut v, B, champ(1)).unwrap();
    assert_eq!(v.apply(game, None), Err(VetoError::DraftPending));
    while let Turn::Act { sides, .. } = v.turn() {
        let choice = v.random_choice(sides[0], &ctx).unwrap();
        step_in(&mut v, sides[0], choice, &ctx).unwrap();
    }
    v.apply(game, None).unwrap();
    assert!(v.picks(3).iter().all(Option::is_some));

    // Nos times, também a janela, pelo elenco: com a partida 1 enviada,
    // faltam a 2 e a 3.
    let mut t = two_v_two(team_rules(0, 2, false), 3);
    t.set_rosters(sent(&[P1, P2], &[P3, P4], 1, [&[P1, P2], &[P3, P4]]))
        .unwrap();
    assert_eq!(t.apply(game, None), Err(VetoError::DraftPending));
    assert_eq!(missing(&t).len(), 8);
    for m in t.missing() {
        let champion = t
            .random_team_choice(m.side, m.player, m.game, &ctx)
            .unwrap();
        let pick = Event::TeamPick {
            side: m.side,
            player: m.player,
            game: m.game,
            champion,
            gave_way: false,
        };
        t.apply(pick, Some(&ctx)).unwrap();
    }
    t.apply(game, None).unwrap();

    // Sem o draft no site, a partida 2 começa mesmo com o veto de mapas pela
    // metade, como antes.
    let mut off = begun(rules(MapMode::Veto, 7, false, 0), 3, A);
    step(&mut off, A, map(1)).unwrap();
    off.apply(game, None).unwrap();
}

#[test]
fn a_sent_or_finished_game_does_not_undo() {
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 1, A);
    step(&mut v, A, champ(1)).unwrap();
    step(&mut v, B, champ(2)).unwrap();
    v.set_rosters(open(&[P1], &[P3])).unwrap();
    assert!(v.can_undo());
    // A partida 1 enviada (a série md1 acabou com ela).
    let mut played = v.clone();
    played
        .set_rosters(Rosters {
            finished: true,
            ..sent(&[P1], &[P3], 1, [&[P1], &[P3]])
        })
        .unwrap();
    assert!(!played.can_undo());
    assert_eq!(played.apply(Event::Undo, None), Err(VetoError::Finished));
    // Enviada, antes de a série seguir.
    let mut sent_only = v.clone();
    sent_only
        .set_rosters(sent(&[P1], &[P3], 1, [&[P1], &[P3]]))
        .unwrap();
    assert_eq!(sent_only.apply(Event::Undo, None), Err(VetoError::Finished));
    // W.O. no meio do veto.
    let mut wo = v.clone();
    wo.set_rosters(Rosters {
        finished: true,
        ..open(&[P1], &[P3])
    })
    .unwrap();
    assert!(!wo.can_undo());
    assert_eq!(wo.apply(Event::Undo, None), Err(VetoError::Finished));
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
