//! O draft de campeões da série no duelo: bans da série, picks partida a
//! partida antes da partida 1, espelho, não repetição e o que cede.

use super::*;

/// md3 em veto de mapas, A no seed melhor: A escolhe o mapa 1, B o 2, e o 3
/// é o decisivo.
fn after_map_veto(bans: u8) -> Veto {
    let mut v = begun(rules(MapMode::Veto, 7, true, bans), 3, A);
    for (side, m) in [(A, 1), (B, 2), (A, 3), (B, 4), (A, 5), (B, 6)] {
        step(&mut v, side, map(m)).unwrap();
    }
    v
}

#[test]
fn the_whole_series_is_drafted_before_game_one() {
    // Como no regulamento da KeenCon: o seed pior (B) bane primeiro; na
    // partida 1, o melhor escolhe e o pior responde; na 2, o contrário.
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = after_map_veto(1);
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![B]));
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    // O ban vale para os dois.
    assert_eq!(
        step_in(&mut v, A, champ(1), &ctx),
        Err(VetoError::OptionInvalid)
    );
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    for (side, game, c) in [
        (A, 1, 3),
        (B, 1, 3),
        (B, 2, 4),
        (A, 2, 5),
        (A, 3, 6),
        (B, 3, 7),
    ] {
        assert_eq!(picking(&v), (side, game));
        // Fora da vez.
        assert_eq!(
            step_in(&mut v, other(side), champ(c), &ctx),
            Err(VetoError::NotYourTurn)
        );
        step_in(&mut v, side, champ(c), &ctx).unwrap();
    }
    // Espelho vale: os dois com o mesmo na partida 1.
    assert_eq!(v.picks(1), [Some(ChampionId(103)), Some(ChampionId(103))]);
    assert_eq!(v.picks(2), [Some(ChampionId(105)), Some(ChampionId(104))]);
    assert_eq!(v.picks(3), [Some(ChampionId(106)), Some(ChampionId(107))]);
    assert_eq!(v.bans().len(), 2);
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    // Ninguém volta entre as partidas.
    next_game(&mut v, 2);
    assert_eq!(v.turn(), Turn::Done { game: 2 });
    assert!(!v.can_undo());
    next_game(&mut v, 3);
    assert_eq!(v.turn(), Turn::Done { game: 3 });
    assert_eq!(v.map_of(3), Some(MapId(7)));
}

#[test]
fn bans_hold_for_every_game_of_the_series() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = begun(rules(MapMode::Off, 0, true, 2), 5, A);
    for (side, c) in [(B, 1), (A, 2), (B, 3), (A, 4)] {
        step_in(&mut v, side, champ(c), &ctx).unwrap();
    }
    for _ in 0..10 {
        let (side, _) = picking(&v);
        let options = v.options(side, &ctx);
        assert_eq!(options.len(), 13);
        for c in 1..=4 {
            assert!(!options.contains(&champ(c)));
        }
        step_in(&mut v, side, options[0], &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
}

#[test]
fn without_mirror_whoever_picks_first_keeps_it() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = begun(no_mirror(rules(MapMode::Off, 0, true, 0)), 3, A);
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    // B não pega o 1 na partida 1...
    assert_eq!(
        step_in(&mut v, B, champ(1), &ctx),
        Err(VetoError::ChampionTaken)
    );
    assert!(!v.options(B, &ctx).contains(&champ(1)));
    let picker = Picker {
        side: B,
        player: None,
    };
    assert_eq!(
        v.unavailable(picker, 1, &ctx),
        [(ChampionId(101), Unavailable::Opponent)]
    );
    step_in(&mut v, B, champ(2), &ctx).unwrap();
    // ...mas pega na 2, sem regra de não repetição.
    assert_eq!(picking(&v), (B, 2));
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    assert_eq!(
        step_in(&mut v, A, champ(1), &ctx),
        Err(VetoError::ChampionTaken)
    );
    step_in(&mut v, A, champ(2), &ctx).unwrap();
}

#[test]
fn no_repeat_in_the_series_takes_the_picks_of_the_other_games() {
    let champions = champions(5);
    let ctx = ctx(&champions, [&[], &[]]);
    let mut v = begun(ruled(MapMode::Off, 0, true, 0), 3, A);
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    // Partida 2: B não repete o 1; A também não, mas o espelho vale.
    assert_eq!(
        step_in(&mut v, B, champ(1), &ctx),
        Err(VetoError::ChampionForbidden)
    );
    let picker = Picker {
        side: B,
        player: None,
    };
    assert_eq!(
        v.unavailable(picker, 2, &ctx),
        [(ChampionId(101), Unavailable::Repeat)]
    );
    step_in(&mut v, B, champ(2), &ctx).unwrap();
    assert_eq!(v.options(A, &ctx), [2, 3, 4, 5].map(champ));
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    // Partida 3: cada um perde os dois que escolheu.
    assert_eq!(v.options(A, &ctx), [3, 4, 5].map(champ));
    // Fora do pool.
    assert_eq!(
        step_in(&mut v, A, champ(9), &ctx),
        Err(VetoError::OptionInvalid)
    );
}

#[test]
fn the_scope_outside_the_series_comes_from_the_api() {
    // Todos contra todos com personagens diferentes: A já jogou com o 5 em
    // outra série (a API só passa o que foi jogado).
    let champions = champions(17);
    let elsewhere = [ChampionId(105)];
    let ctx = ctx(&champions, [&elsewhere, &[]]);
    let mut v = begun(ruled(MapMode::Off, 0, true, 0), 1, A);
    assert_eq!(
        step_in(&mut v, A, champ(5), &ctx),
        Err(VetoError::ChampionForbidden)
    );
    assert_eq!(v.options(A, &ctx).len(), 16);
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    step_in(&mut v, B, champ(5), &ctx).unwrap();
}

#[test]
fn the_rule_gives_way_when_nothing_is_left() {
    let champions = champions(3);
    let used = [ChampionId(101), ChampionId(102), ChampionId(103)];
    let ctx = ctx(&champions, [&used, &[]]);
    let v = begun(ruled(MapMode::Off, 0, true, 0), 5, A);
    // A usou os três: a regra cede para o veto não travar.
    let options = v.pick_options(
        Picker {
            side: A,
            player: None,
        },
        1,
        &ctx,
    );
    assert_eq!(options.champions.len(), 3);
    assert_eq!(options.gave_way, Some(GaveWay::Rule));
    assert!(v.random_choice(A, &ctx).is_some());
    // Sem espelho e com um campeão só, cede também a da partida.
    let one = champions[..1].to_vec();
    let mut w = begun(no_mirror(rules(MapMode::Off, 0, true, 0)), 1, A);
    step_in(&mut w, A, champ(1), &free(&one)).unwrap();
    let options = w.pick_options(
        Picker {
            side: B,
            player: None,
        },
        1,
        &free(&one),
    );
    assert_eq!(options.champions, [ChampionId(101)]);
    assert_eq!(options.gave_way, Some(GaveWay::Game));
    step_in(&mut w, B, champ(1), &free(&one)).unwrap();
    assert_eq!(w.turn(), Turn::Done { game: 1 });
}

#[test]
fn a_ban_cannot_leave_the_banner_without_picks_for_the_series() {
    // Pool de 6, 2 bans de cada lado, md3 sem repetir. B é o seed melhor:
    // A bane primeiro. A já usou 101 e 102 fora da série: pode escolher 103
    // a 106, e precisa de 3. B ainda tem 2 bans; se A banisse um dos seus,
    // B poderia deixá-lo com 1.
    let champions = champions(6);
    let used = [ChampionId(101), ChampionId(102)];
    let ctx = ctx(&champions, [&used, &[]]);
    let mut v = begun(ruled(MapMode::Off, 0, true, 2), 3, B);
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    assert_eq!(v.options(A, &ctx), [champ(1), champ(2)]);
    assert_eq!(
        step_in(&mut v, A, champ(3), &ctx),
        Err(VetoError::OptionInvalid)
    );
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    // B bane um dos de A.
    step_in(&mut v, B, champ(3), &ctx).unwrap();
    assert_eq!(v.options(A, &ctx), [champ(2)]);
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    // B também ficaria no limite (3 para 3 partidas), mas todo campeão que
    // sobrou é dele: a proteção cede.
    assert_eq!(v.options(B, &ctx), [4, 5, 6].map(champ));
}

#[test]
fn a_timeout_picks_at_random_within_the_rules() {
    let champions = champions(17);
    let used = [ChampionId(101), ChampionId(102)];
    let ctx = ctx(&champions, [&used, &[]]);
    for seed in 0..30 {
        let mut v = Veto::replay(
            no_mirror(ruled(MapMode::Off, 0, true, 1)),
            3,
            [Event::Begin { seed, first: A }],
        )
        .unwrap();
        while let Turn::Act { sides, .. } = v.turn() {
            let choice = v.random_choice(sides[0], &ctx).unwrap();
            assert_eq!(v.random_choice(sides[0], &ctx), Some(choice));
            step_in(&mut v, sides[0], choice, &ctx).unwrap();
        }
        let mine: Vec<ChampionId> = (1..=3).filter_map(|g| v.picks(g)[0]).collect();
        assert_eq!(mine.len(), 3);
        assert!(mine.iter().all(|c| !used.contains(c)));
        assert!(mine[0] != mine[1] && mine[1] != mine[2] && mine[0] != mine[2]);
        for g in 1..=3 {
            let [a, b] = v.picks(g);
            assert_ne!(a, b, "espelho na partida {g}");
        }
    }
}

#[test]
fn every_length_drafts_every_game() {
    let champions = champions(17);
    let ctx = ctx(&champions, [&[], &[]]);
    for best_of in [1u8, 3, 5, 7] {
        for bans in 0..=3u8 {
            let mut v = begun(ruled(MapMode::Off, 0, true, bans), best_of, A);
            let mut steps = 0;
            while let Turn::Act { sides, .. } = v.turn() {
                let choice = v.options(sides[0], &ctx)[0];
                step_in(&mut v, sides[0], choice, &ctx).unwrap();
                steps += 1;
            }
            assert_eq!(steps, 2 * usize::from(bans) + 2 * usize::from(best_of));
            assert_eq!(v.turn(), Turn::Done { game: 1 });
            assert_eq!(v.bans().len(), 2 * usize::from(bans));
            for side in [A, B] {
                let mut mine: Vec<ChampionId> = (1..=best_of)
                    .filter_map(|g| v.picks(g)[index(side)])
                    .collect();
                assert_eq!(mine.len(), usize::from(best_of));
                mine.sort();
                mine.dedup();
                assert_eq!(mine.len(), usize::from(best_of), "{side:?} repetiu");
            }
            // As partidas seguintes não esperam nada.
            for g in 2..=best_of {
                next_game(&mut v, g);
                assert_eq!(v.turn(), Turn::Done { game: g });
            }
        }
    }
}

#[test]
fn the_vote_of_each_game_comes_back_but_the_draft_does_not() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = begun(rules(MapMode::Vote, 7, true, 0), 3, B);
    let offered = v.offered(1);
    // O mapa da partida 1 sai antes do draft.
    assert_eq!(acting(&v).1, Target::Map);
    for side in [A, B] {
        step_in(&mut v, side, Choice::Map(offered[0]), &ctx).unwrap();
    }
    // B é o seed melhor: escolhe primeiro na partida 1.
    assert_eq!(picking(&v), (B, 1));
    for _ in 0..6 {
        let (side, _) = picking(&v);
        step_in(&mut v, side, champ(1), &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    next_game(&mut v, 2);
    assert_eq!(acting(&v), (Action::Vote, Target::Map, vec![A, B]));
    let offered = v.offered(2);
    for side in [A, B] {
        step_in(&mut v, side, Choice::Map(offered[1]), &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 2 });
    assert_eq!(v.picks(2), [Some(ChampionId(101)), Some(ChampionId(101))]);
}

#[test]
fn the_replay_rebuilds_the_draft() {
    let champions = champions(17);
    let ctx = ctx(&champions, [&[], &[]]);
    let rules = no_mirror(ruled(MapMode::Veto, 5, true, 1));
    let mut events = vec![Event::Begin { seed: 7, first: A }];
    let mut v = Veto::replay(rules.clone(), 3, events.clone()).unwrap();
    while let Turn::Act { sides, action, .. } = v.turn() {
        let choice = v.random_choice(sides[0], &ctx).unwrap();
        let e = Event::Step {
            side: sides[0],
            action,
            choice,
        };
        v.apply(e, Some(&ctx)).unwrap();
        events.push(e);
    }
    events.push(Event::Game {
        game: 2,
        previous: v.map_of(1),
    });
    v.apply(*events.last().unwrap(), None).unwrap();
    assert_eq!(Veto::replay(rules.clone(), 3, events.clone()).unwrap(), v);
    // Um pick do lado errado não fecha.
    let mut bad = events[..events.len() - 1].to_vec();
    let last = bad.len() - 1;
    if let Event::Step { side, .. } = &mut bad[last] {
        *side = other(*side);
    }
    assert_eq!(Veto::replay(rules, 3, bad), Err(VetoError::NotYourTurn));
}

#[test]
fn the_veto_does_not_begin_with_champions_only_for_the_bans() {
    // 3 bans de cada lado e 6 campeões: nada sobraria para escolher.
    let six = champions(6);
    let seven = champions(7);
    let begin = Event::Begin { seed: 1, first: A };
    let mut v = Veto::new(rules(MapMode::Off, 0, true, 3), 3);
    assert_eq!(
        v.apply(begin, Some(&free(&six))),
        Err(VetoError::ChampionPoolExhausted {
            required: 7,
            available: 6
        })
    );
    assert!(!v.begun());
    v.apply(begin, Some(&free(&seven))).unwrap();
    // Sem o draft no site, não confere; no replay, também não.
    let mut w = Veto::new(rules(MapMode::Off, 0, false, 3), 3);
    w.apply(begin, Some(&free(&six))).unwrap();
    assert!(Veto::replay(rules(MapMode::Off, 0, true, 3), 3, [begin]).is_ok());
}
