//! Votação de mapa a cada jogo.

use super::*;

fn voted(seed: u64) -> Veto {
    Veto::replay(
        rules(MapMode::Vote, 7, false, 0),
        5,
        [Event::Begin { seed, first: A }],
    )
    .unwrap()
}

fn vote(v: &mut Veto, side: Side, m: MapId) -> Result<(), VetoError> {
    v.apply(
        Event::Step {
            side,
            action: Action::Vote,
            choice: Choice::Map(m),
        },
        None,
    )
}

#[test]
fn three_maps_both_vote_in_any_order() {
    let mut v = voted(7);
    let offered = v.offered(1);
    assert_eq!(offered.len(), VOTE_OPTIONS);
    assert_eq!(acting(&v), (Action::Vote, Target::Map, vec![A, B]));
    let outside = pool(7).into_iter().find(|m| !offered.contains(m)).unwrap();
    assert_eq!(vote(&mut v, B, outside), Err(VetoError::OptionInvalid));
    vote(&mut v, B, offered[0]).unwrap();
    assert_eq!(acting(&v).2, vec![A]);
    assert_eq!(vote(&mut v, B, offered[1]), Err(VetoError::NotYourTurn));
    assert_eq!(v.map_of(1), None);
    vote(&mut v, A, offered[0]).unwrap();
    // Votos iguais: é esse.
    assert_eq!(v.map_of(1), Some(offered[0]));
    assert_eq!(v.turn(), Turn::Done { game: 1 });
}

#[test]
fn different_votes_draw_one_of_the_two() {
    let mut both = std::collections::HashSet::new();
    for seed in 0..40 {
        let mut v = voted(seed);
        let offered = v.offered(1);
        vote(&mut v, A, offered[0]).unwrap();
        vote(&mut v, B, offered[2]).unwrap();
        let m = v.map_of(1).unwrap();
        assert!(m == offered[0] || m == offered[2]);
        both.insert(m == offered[0]);
        // Reproduzível: o mesmo histórico dá o mesmo mapa.
        assert_eq!(v.clone().map_of(1), Some(m));
    }
    assert_eq!(both.len(), 2, "o desempate nunca escolheu um dos lados");
}

#[test]
fn played_maps_leave_the_vote_and_the_pool_can_run_short() {
    // md7 com 7 mapas: no sexto jogo sobram 2, no sétimo, 1 (sem votação).
    let mut v = Veto::replay(
        rules(MapMode::Vote, 7, false, 0),
        7,
        [Event::Begin { seed: 3, first: A }],
    )
    .unwrap();
    let mut played = Vec::new();
    for game in 1..=7u8 {
        if game > 1 {
            v.apply(
                Event::Game {
                    game,
                    previous: None,
                },
                None,
            )
            .unwrap();
        }
        let offered = v.offered(game);
        assert_eq!(offered.len(), (8 - usize::from(game)).min(VOTE_OPTIONS));
        assert!(offered.iter().all(|m| !played.contains(m)));
        if offered.len() > 1 {
            vote(&mut v, A, offered[0]).unwrap();
            vote(&mut v, B, offered[0]).unwrap();
        } else {
            assert_eq!(v.turn(), Turn::Done { game });
        }
        played.push(v.map_of(game).unwrap());
    }
    played.sort();
    assert_eq!(played, pool(7));
}

#[test]
fn best_of_five_always_has_three_to_vote() {
    let mut v = voted(11);
    for game in 1..=5u8 {
        if game > 1 {
            v.apply(
                Event::Game {
                    game,
                    previous: None,
                },
                None,
            )
            .unwrap();
        }
        assert_eq!(v.offered(game).len(), 3, "jogo {game}");
        let m = v.offered(game)[1];
        vote(&mut v, A, m).unwrap();
        vote(&mut v, B, m).unwrap();
    }
}

#[test]
fn a_timeout_votes_at_random_for_both() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = voted(5);
    let offered = v.offered(1);
    for side in [A, B] {
        let choice = v.random_choice(side, &ctx).unwrap();
        let Choice::Map(m) = choice else {
            panic!("voto sem mapa")
        };
        assert!(offered.contains(&m));
        vote(&mut v, side, m).unwrap();
    }
    assert!(v.map_of(1).is_some());
    assert_eq!(v.random_choice(A, &ctx), None);
}

#[test]
fn the_vote_is_pinned() {
    // Sorteios já gravados dependem deste resultado.
    let v = voted(2026);
    assert_eq!(v.offered(1), [MapId(1), MapId(3), MapId(6)]);
}

#[test]
fn the_map_actually_played_leaves_the_pool() {
    // O organizador lançou o jogo 1 sem a votação, no mapa 7: ele não volta
    // na votação do jogo 2, e no aleatório também não.
    for mode in [MapMode::Vote, MapMode::Random] {
        let mut v = Veto::replay(
            rules(mode, 3, false, 0),
            3,
            [
                Event::Begin { seed: 1, first: A },
                Event::Game {
                    game: 2,
                    previous: Some(MapId(3)),
                },
            ],
        )
        .unwrap();
        if mode == MapMode::Vote {
            assert_eq!(v.offered(2), [MapId(1), MapId(2)]);
            vote(&mut v, A, MapId(2)).unwrap();
            vote(&mut v, B, MapId(2)).unwrap();
        }
        let second = v.map_of(2).unwrap();
        assert_ne!(second, MapId(3));
        v.apply(
            Event::Game {
                game: 3,
                previous: Some(second),
            },
            None,
        )
        .unwrap();
        let mut left = pool(3);
        left.retain(|m| *m != MapId(3) && *m != second);
        assert_eq!(v.map_of(3).into_iter().collect::<Vec<_>>(), left);
    }
}
