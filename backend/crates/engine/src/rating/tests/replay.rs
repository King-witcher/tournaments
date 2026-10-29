use proptest::prelude::*;

use super::*;

#[test]
fn first_game_between_new_players() {
    let replay = replay(&elo(), &[beats(A, B)]).unwrap();
    assert_eq!(ratings(&replay), [(A, 1540), (B, 1460)]);
    assert_eq!(replay.effects, [game((40, 1540), (-40, 1460))]);
    let a = replay.standings[&A];
    assert_eq!((a.games, a.wins, a.losses), (1, 1, 0));
    assert!(a.provisional());
}

#[test]
fn second_game_uses_the_new_ratings_and_a_smaller_k() {
    // K de 75,43 para quem já tem um jogo; o favorito ganha menos...
    let replay_ = replay(&elo(), &[beats(A, B), beats(A, B)]).unwrap();
    assert_eq!(ratings(&replay_), [(A, 1569), (B, 1431)]);
    assert_eq!(replay_.effects[1], game((29, 1569), (-29, 1431)));
    // ...e a zebra ganha mais.
    let upset = replay(&elo(), &[beats(A, B), beats(B, A)]).unwrap();
    assert_eq!(ratings(&upset), [(A, 1494), (B, 1506)]);
    let b = upset.standings[&B];
    assert_eq!((b.games, b.wins, b.losses), (2, 1, 1));
}

proptest! {
    #[test]
    fn total_is_initial_plus_adjustments_when_k_is_fixed(
        games in prop::collection::vec((0..4i64, 0..4i64), 0..40),
        adjustments in prop::collection::vec((0..4i64, prop_oneof![-100..0i32, 1..100i32]), 0..5),
    ) {
        let fixed = Elo::new(EloConfig { k_min: 32, k_max: 32, ..EloConfig::default() }).unwrap();
        let mut events: Vec<Event> = games
            .into_iter()
            .filter(|(w, l)| w != l)
            .map(|(w, l)| beats(PlayerId(w), PlayerId(l)))
            .collect();
        let adjusted: i32 = adjustments.iter().map(|(_, d)| d).sum();
        events.extend(adjustments.into_iter().map(|(p, delta)| Event::Adjustment {
            player: PlayerId(p),
            delta,
        }));
        let replay = replay(&fixed, &events).unwrap();
        let total: i32 = replay.standings.values().map(|s| s.rating).sum();
        prop_assert_eq!(total, 1500 * replay.standings.len() as i32 + adjusted);
    }
}

#[test]
fn adjustment_moves_the_rating_without_counting_a_game() {
    let events = [
        Event::Adjustment {
            player: A,
            delta: 50,
        },
        beats(A, B),
        Event::Adjustment {
            player: B,
            delta: -10,
        },
    ];
    let replay = replay(&elo(), &events).unwrap();
    assert_eq!(
        replay.effects,
        [
            Effect::Adjustment(Change {
                delta: 50,
                after: 1550
            }),
            // A ainda tem K 80: o ajuste não conta como jogo.
            game((34, 1584), (-34, 1466)),
            Effect::Adjustment(Change {
                delta: -10,
                after: 1456
            }),
        ]
    );
    assert_eq!(replay.standings[&A].games, 1);
    // Ajuste de quem nunca jogou parte do inicial e não vira jogo.
    let only = replay_of(&[Event::Adjustment {
        player: C,
        delta: -30,
    }]);
    assert_eq!(
        only.standings[&C],
        Standing {
            rating: 1470,
            games: 0,
            wins: 0,
            losses: 0
        }
    );
}

#[test]
fn order_matters_and_is_deterministic() {
    let first = [beats(A, B), beats(C, A)];
    let second = [beats(C, A), beats(A, B)];
    assert_eq!(
        ratings(&replay_of(&first)),
        [(A, 1498), (B, 1460), (C, 1545)]
    );
    assert_eq!(
        ratings(&replay_of(&second)),
        [(A, 1502), (B, 1455), (C, 1540)]
    );
    assert_eq!(replay_of(&first), replay_of(&first));
}

#[test]
fn empty_history() {
    let replay = replay_of(&[]);
    assert!(replay.standings.is_empty() && replay.effects.is_empty());
}

#[test]
fn self_game_is_refused() {
    assert_eq!(
        replay(&elo(), &[beats(A, B), beats(B, B)]),
        Err(RatingError::SelfGame {
            index: 1,
            player: B
        })
    );
}

#[test]
fn provisional_until_ten_games() {
    let events: Vec<Event> = (0..10)
        .map(|i| if i % 2 == 0 { beats(A, B) } else { beats(B, A) })
        .collect();
    assert!(replay_of(&events[..9]).standings[&A].provisional());
    let ten = replay_of(&events);
    assert!(!ten.standings[&A].provisional());
    assert_eq!(ten.standings[&A].games, PROVISIONAL_GAMES);
}

#[test]
fn adjustment_between_games_changes_the_rating_but_not_the_k() {
    let events = [
        beats(A, B),
        Event::Adjustment {
            player: A,
            delta: -40,
        },
        beats(A, B),
    ];
    // Depois do ajuste, 1500 contra 1460, e A ainda com K de quem tem 1 jogo.
    let replay = replay_of(&events);
    assert_eq!(replay.effects[2], game((33, 1533), (-33, 1427)));
    assert_eq!(replay.standings[&A].games, 2);
}

#[test]
fn new_player_beating_a_veteran_in_a_full_replay() {
    let veteran = PlayerId(10);
    let rookie = PlayerId(11);
    let mut events: Vec<Event> = (0..50)
        .map(|i| {
            if i % 2 == 0 {
                beats(veteran, A)
            } else {
                beats(A, veteran)
            }
        })
        .collect();
    events.push(beats(rookie, veteran));
    let replay = replay_of(&events);
    let Some(&Effect::Game { winner, loser }) = replay.effects.last() else {
        panic!("o último evento é um jogo");
    };
    let before = loser.after - loser.delta;
    assert_eq!(
        (winner.delta, -loser.delta),
        elo().points(1500, 0, before, 50)
    );
    assert!(winner.delta > -loser.delta, "{winner:?} {loser:?}");
}

#[test]
fn a_game_worth_zero_points_still_counts() {
    let events = [
        Event::Adjustment {
            player: A,
            delta: 900,
        },
        beats(A, B),
    ];
    let replay = replay_of(&events);
    assert_eq!(replay.effects[1], game((0, 2400), (0, 1500)));
    let (a, b) = (replay.standings[&A], replay.standings[&B]);
    assert_eq!((a.games, a.wins, b.games, b.losses), (1, 1, 1, 1));
}

#[test]
fn adjustment_limits() {
    for delta in [MAX_ADJUSTMENT, -MAX_ADJUSTMENT] {
        let events = [Event::Adjustment { player: A, delta }];
        assert!(replay(&elo(), &events).is_ok(), "{delta}");
    }
    for delta in [
        0,
        MAX_ADJUSTMENT + 1,
        -MAX_ADJUSTMENT - 1,
        i32::MAX,
        i32::MIN,
    ] {
        let events = [beats(A, B), Event::Adjustment { player: A, delta }];
        assert_eq!(
            replay(&elo(), &events),
            Err(RatingError::AdjustmentOutOfRange {
                index: 1,
                max: MAX_ADJUSTMENT
            }),
            "{delta}"
        );
    }
    // Muitos ajustes no máximo não estouram.
    let many = vec![
        Event::Adjustment {
            player: A,
            delta: MAX_ADJUSTMENT
        };
        5000
    ];
    assert_eq!(replay_of(&many).standings[&A].rating, 1500 + 5_000_000);
}

#[test]
fn self_game_after_adjustments() {
    let events = [
        Event::Adjustment {
            player: A,
            delta: 10,
        },
        beats(A, A),
    ];
    assert_eq!(
        replay(&elo(), &events),
        Err(RatingError::SelfGame {
            index: 1,
            player: A
        })
    );
}

/// Até 6 pessoas, jogos e ajustes misturados.
fn events() -> impl Strategy<Value = Vec<Event>> {
    let event = prop_oneof![
        4 => (0..6i64, 0..6i64)
            .prop_filter("dois lados", |(w, l)| w != l)
            .prop_map(|(w, l)| beats(PlayerId(w), PlayerId(l))),
        1 => (0..6i64, prop_oneof![-MAX_ADJUSTMENT..0, 1..=MAX_ADJUSTMENT])
            .prop_map(|(p, delta)| Event::Adjustment { player: PlayerId(p), delta }),
    ];
    prop::collection::vec(event, 0..60)
}

proptest! {
    #[test]
    fn effects_add_up_to_the_standings(events in events()) {
        let replay = replay_of(&events);
        prop_assert_eq!(replay.effects.len(), events.len());
        for (player, standing) in &replay.standings {
            let mut rating = 1500;
            let mut games = 0;
            for (event, effect) in events.iter().zip(&replay.effects) {
                let change = match (*event, *effect) {
                    (Event::Game { winner, .. }, Effect::Game { winner: c, .. }) if winner == *player => {
                        games += 1;
                        c
                    }
                    (Event::Game { loser, .. }, Effect::Game { loser: c, .. }) if loser == *player => {
                        games += 1;
                        c
                    }
                    (Event::Adjustment { player: p, .. }, Effect::Adjustment(c)) if p == *player => c,
                    _ => continue,
                };
                rating += change.delta;
                prop_assert_eq!(change.after, rating);
            }
            prop_assert_eq!(standing.rating, rating);
            prop_assert_eq!(standing.games, games);
            prop_assert_eq!(standing.wins + standing.losses, standing.games);
        }
    }

    #[test]
    fn a_prefix_replays_the_same(events in events(), cut in 0..60usize) {
        let all = replay_of(&events);
        let cut = cut.min(events.len());
        let prefix = replay_of(&events[..cut]);
        prop_assert_eq!(&prefix.effects[..], &all.effects[..cut]);
        prop_assert_eq!(replay_of(&events), all);
    }
}
