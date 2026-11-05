//! As séries pares (md2, md4, md6): no veto, só bans até sobrarem N mapas;
//! votação, sorteio, desligado e o draft, por partida, como em qualquer série.

use super::*;

/// Bane os mapas `maps`, alternando a partir de `first`.
fn ban_maps(v: &mut Veto, first: Side, maps: &[i64]) {
    for (i, m) in maps.iter().enumerate() {
        let side = alternate(first, i);
        assert_eq!(acting(v), (Action::Ban, Target::Map, vec![side]));
        step(v, side, map(*m)).unwrap();
    }
}

/// Bane os `n` primeiros do pool, alternando a partir de `first`.
fn ban_first(v: &mut Veto, first: Side, n: i64) {
    let maps: Vec<i64> = (1..=n).collect();
    ban_maps(v, first, &maps);
}

fn maps_of(v: &Veto, games: u8) -> Vec<Option<MapId>> {
    (1..=games).map(|g| v.map_of(g)).collect()
}

#[test]
fn only_bans_alternating_until_two_are_left() {
    let mut v = begun(rules(MapMode::Veto, 7, false, 0), 2, B);
    // O seed melhor (aqui o lado B) começa.
    ban_first(&mut v, B, 4);
    // Com 3 mapas ainda não há mapa de nenhuma partida.
    assert_eq!(v.remaining_maps().len(), 3);
    assert_eq!((v.map_of(1), v.map_of(2)), (None, None));
    // Sempre ban: nada de pick.
    let pick = Event::Step {
        side: B,
        action: Action::Pick,
        choice: map(5),
    };
    assert_eq!(v.apply(pick, None), Err(VetoError::WrongAction));
    step(&mut v, B, map(6)).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    // Na ordem do pool, e não na ordem em que sobraram.
    assert_eq!(v.remaining_maps(), [MapId(5), MapId(7)]);
    assert_eq!((v.map_of(1), v.map_of(2)), (Some(MapId(5)), Some(MapId(7))));
    assert_eq!(v.map_of(3), None);
    // Sem quem escolheu nem decisivo.
    assert_eq!((v.picker(1), v.picker(2), v.decider()), (None, None, None));
}

#[test]
fn best_of_four_bans_three_and_plays_the_rest_in_pool_order() {
    let mut v = begun(rules(MapMode::Veto, 7, false, 0), 4, A);
    ban_maps(&mut v, A, &[2, 5]);
    assert_eq!(maps_of(&v, 4), [None, None, None, None]);
    ban_maps(&mut v, A, &[7]);
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert_eq!(
        maps_of(&v, 4),
        [
            Some(MapId(1)),
            Some(MapId(3)),
            Some(MapId(4)),
            Some(MapId(6))
        ]
    );
    assert_eq!(v.map_of(5), None);
    assert_eq!(v.decider(), None);
    assert!((1..=4).all(|g| v.picker(g).is_none()));
}

#[test]
fn best_of_six_bans_one_and_plays_the_rest_in_pool_order() {
    let mut v = begun(rules(MapMode::Veto, 7, false, 0), 6, B);
    ban_maps(&mut v, B, &[4]);
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    let want: Vec<Option<MapId>> = [1, 2, 3, 5, 6, 7].map(|m| Some(MapId(m))).to_vec();
    assert_eq!(maps_of(&v, 6), want);
    assert_eq!(v.map_of(7), None);
    assert_eq!(v.decider(), None);
}

#[test]
fn the_veto_order_makes_no_difference() {
    for (best_of, bans) in [(2u8, 5), (4, 3), (6, 1)] {
        let play = |r: VetoRules| {
            let mut v = begun(r, best_of, A);
            ban_first(&mut v, A, bans);
            (maps_of(&v, best_of), v.turn())
        };
        let alternating = rules(MapMode::Veto, 7, false, 0);
        let bans_only = bans_first(alternating.clone());
        assert_eq!(play(alternating), play(bans_only), "md{best_of}");
    }
}

#[test]
fn every_pool_ends_with_the_last_n_in_pool_order() {
    for best_of in [2u8, 4, 6] {
        let games = i64::from(best_of);
        for maps in games..=9 {
            let mut v = begun(rules(MapMode::Veto, maps, false, 0), best_of, A);
            ban_first(&mut v, A, maps - games);
            assert_eq!(v.turn(), Turn::Done { game: 1 });
            assert_eq!(v.map_steps().len() as i64, maps - games);
            let want: Vec<Option<MapId>> =
                (maps - games + 1..=maps).map(|m| Some(MapId(m))).collect();
            assert_eq!(maps_of(&v, best_of), want, "md{best_of}, pool {maps}");
        }
    }
}

#[test]
fn the_pool_needs_as_many_maps_as_games() {
    for best_of in [2u8, 4, 6] {
        let short = i64::from(best_of) - 1;
        let too_small = VetoError::PoolTooSmall {
            pool: short as usize,
            best_of,
        };
        let begin = [Event::Begin { seed: 1, first: A }];
        assert_eq!(
            Veto::replay(rules(MapMode::Veto, short, false, 0), best_of, begin),
            Err(too_small.clone())
        );
        let r = rules(MapMode::Veto, short, false, 0);
        assert_eq!(r.validate(best_of), Err(too_small));
        let enough = rules(MapMode::Veto, short + 1, false, 0);
        assert_eq!(enough.validate(best_of), Ok(()));
    }
}

#[test]
fn the_games_start_in_order_and_stop_at_n() {
    for best_of in [2u8, 4, 6] {
        let mut v = begun(rules(MapMode::Veto, 7, false, 0), best_of, A);
        ban_first(&mut v, A, 7 - i64::from(best_of));
        let maps = maps_of(&v, best_of);
        for game in 2..=best_of {
            next_game(&mut v, game);
            assert_eq!(v.turn(), Turn::Done { game });
        }
        // Os mapas não mudam com o avanço da série.
        assert_eq!(maps_of(&v, best_of), maps);
        let beyond = Event::Game {
            game: best_of + 1,
            previous: None,
        };
        assert_eq!(
            v.apply(beyond, None),
            Err(VetoError::GameOutOfOrder { next: best_of + 1 })
        );
    }
}

#[test]
fn undoing_the_last_ban_hides_the_maps_again() {
    let mut v = begun(rules(MapMode::Veto, 3, false, 0), 2, A);
    ban_first(&mut v, A, 1);
    assert_eq!(v.map_of(1), Some(MapId(2)));
    v.apply(Event::Undo, None).unwrap();
    assert_eq!((v.map_of(1), v.map_of(2)), (None, None));
    assert_eq!(acting(&v), (Action::Ban, Target::Map, vec![A]));
}

#[test]
fn the_vote_runs_game_by_game() {
    let vote = |v: &mut Veto, side: Side, m: MapId| {
        v.apply(
            Event::Step {
                side,
                action: Action::Vote,
                choice: Choice::Map(m),
            },
            None,
        )
    };
    for best_of in [2u8, 4, 6] {
        let mut v = begun(rules(MapMode::Vote, 7, false, 0), best_of, A);
        let mut played = Vec::new();
        for game in 1..=best_of {
            if game > 1 {
                v.apply(
                    Event::Game {
                        game,
                        previous: played.last().copied(),
                    },
                    None,
                )
                .unwrap();
            }
            // Os mapas já jogados saem dos oferecidos.
            let offered = v.offered(game);
            assert!(offered.iter().all(|m| !played.contains(m)), "md{best_of}");
            vote(&mut v, A, offered[0]).unwrap();
            vote(&mut v, B, offered[0]).unwrap();
            assert_eq!(v.map_of(game), Some(offered[0]));
            assert_eq!(v.turn(), Turn::Done { game });
            played.push(offered[0]);
        }
    }
}

#[test]
fn the_random_map_comes_game_by_game() {
    for best_of in [2u8, 4, 6] {
        let mut v = begun(
            rules(MapMode::Random, i64::from(best_of), false, 0),
            best_of,
            A,
        );
        let mut seen = Vec::new();
        for game in 1..=best_of {
            if game > 1 {
                assert_eq!(v.map_of(game), None);
                v.apply(
                    Event::Game {
                        game,
                        previous: seen.last().copied(),
                    },
                    None,
                )
                .unwrap();
            }
            assert_eq!(v.turn(), Turn::Done { game });
            let m = v.map_of(game).unwrap();
            assert!(!seen.contains(&m), "md{best_of}: mapa repetido");
            seen.push(m);
        }
    }
}

#[test]
fn with_maps_off_there_is_no_map_and_no_step() {
    for best_of in [2u8, 4, 6] {
        let v = begun(rules(MapMode::Off, 0, false, 0), best_of, A);
        assert_eq!(v.turn(), Turn::Done { game: 1 });
        assert!(maps_of(&v, best_of).iter().all(Option::is_none));
        assert_eq!(v.map_steps(), &[]);
    }
}

#[test]
fn the_duel_draft_has_a_pick_each_per_game() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = begun(rules(MapMode::Veto, 3, true, 1), 2, A);
    step(&mut v, A, map(1)).unwrap();
    // O draft vem depois do mapa; os bans, a partir do seed pior.
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![B]));
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    for (side, game, c) in [(A, 1, 3), (B, 1, 4), (B, 2, 5), (A, 2, 6)] {
        assert_eq!(picking(&v), (side, game));
        step_in(&mut v, side, champ(c), &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert_eq!(v.picks(1), [Some(ChampionId(103)), Some(ChampionId(104))]);
    assert_eq!(v.picks(2), [Some(ChampionId(106)), Some(ChampionId(105))]);
    next_game(&mut v, 2);
    assert_eq!(v.turn(), Turn::Done { game: 2 });
}

#[test]
fn the_duel_draft_of_a_best_of_four_picks_for_all_four() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = begun(rules(MapMode::Off, 0, true, 0), 4, A);
    let order = [
        (A, 1, 1),
        (B, 1, 2),
        (B, 2, 3),
        (A, 2, 4),
        (A, 3, 5),
        (B, 3, 6),
        (B, 4, 7),
        (A, 4, 8),
    ];
    for (side, game, c) in order {
        assert_eq!(picking(&v), (side, game));
        step_in(&mut v, side, champ(c), &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
}

#[test]
fn the_team_draft_picks_for_every_game() {
    let champions = champions(40);
    let ctx = free(&champions);
    for (best_of, want) in [
        (2u8, vec![1, 1, 1, 1, 2, 2, 2, 2]),
        (4, vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4]),
    ] {
        let mut r = team_rules(0, 2, false);
        r.pick_order = PickOrder::Alternating;
        let mut v = two_v_two(r, best_of);
        let mut c = 1;
        let mut games = Vec::new();
        while matches!(v.turn(), Turn::Act { .. }) {
            let m = v.due_picks()[0];
            games.push(m.game);
            pick(&mut v, m.side, m.player, m.game, c, &ctx).unwrap();
            c += 1;
        }
        // Quatro jogadores em cada partida, e nada além da última.
        assert_eq!(games, want, "md{best_of}");
    }
}
