//! Propriedades da janela simultânea: qualquer ordem, regra, espelho, uso de
//! fora da série e troca de elenco no meio da série fecham o draft sem
//! campeão banido, com um campeão por jogador do elenco em cada partida
//! aberta, sem repetir no time nem entre times quando não vale (salvo o que
//! cedeu), respeitando a regra quando ela não cede, e o histórico refeito dá
//! o mesmo veto.

use std::collections::HashSet;

use proptest::prelude::*;
use proptest::sample::Index;

use super::*;
use crate::champions::NoRepeatWho;

/// Uma troca no elenco: com a partida `game` aberta, depois de `after`
/// escolhas nela, o jogador `who` do lado `side` sai; com `by`, entra outro
/// no lugar.
#[derive(Clone, Debug)]
struct Change {
    game: u8,
    after: usize,
    side: Side,
    who: Index,
    by: bool,
}

fn change() -> impl Strategy<Value = Change> {
    (
        1u8..=3,
        0usize..20,
        any::<bool>(),
        any::<Index>(),
        any::<bool>(),
    )
        .prop_map(|(game, after, a, who, by)| Change {
            game,
            after,
            side: if a { A } else { B },
            who,
            by,
        })
}

proptest! {
    #[test]
    fn any_order_completes_and_replays(
        seed in any::<u64>(),
        size_a in 1usize..=4,
        size_b in 1usize..=4,
        bans in 0u8..=3,
        who in prop_oneof![Just(None), Just(Some(NoRepeatWho::Player)), Just(Some(NoRepeatWho::Team))],
        same_team in any::<bool>(),
        mirror in any::<bool>(),
        used_a in proptest::collection::vec(1i64..=17, 0..6),
        used_b in proptest::collection::vec(1i64..=17, 0..6),
        used_by in proptest::collection::vec(
            (prop_oneof![1i64..=4, 11i64..=14, 21i64..=22], 1i64..=17),
            0..8,
        ),
        best_of in prop_oneof![Just(1u8), Just(3)],
        played in 1u8..=3,
        order in proptest::collection::vec(any::<Index>(), 0..80),
        changes in proptest::collection::vec(change(), 0..3),
    ) {
        let played = played.min(best_of);
        let champions = champions(17);
        let used = [ids(&used_a), ids(&used_b)];
        let used_by: Vec<(PlayerId, ChampionId)> = used_by
            .iter()
            .map(|(p, c)| (PlayerId(*p), ChampionId(100 + c)))
            .collect();
        let ctx = Context {
            champions: &champions,
            used: [&used[0], &used[1]],
            used_by: &used_by,
        };
        let mut rules = team_rules(bans, 4, same_team);
        rules.champions.mirror = mirror;
        if let Some(who) = who {
            rules = with_rule(rules, who);
        }
        let begin = Event::Begin { seed, first: A };
        let mut events = vec![begin];
        let mut v = Veto::replay(rules.clone(), best_of, [begin]).unwrap();
        let mut rosters = Rosters {
            players: [
                (1..=size_a as i64).map(PlayerId).collect(),
                (11..=10 + size_b as i64).map(PlayerId).collect(),
            ],
            lineups: Vec::new(),
            finished: false,
        };
        v.set_rosters(rosters.clone()).unwrap();
        let mut order = order.into_iter();
        let mut newcomer = 30;
        let mut applied = vec![false; changes.len()];
        let mut gave_way = false;
        for game in 1..=played {
            if game > 1 {
                let lineup = rosters.players.clone();
                rosters.lineups.push(lineup);
                v.set_rosters(rosters.clone()).unwrap();
                let e = Event::Game { game, previous: None };
                v.apply(e, None).unwrap();
                events.push(e);
            }
            loop {
                for (i, c) in changes.iter().enumerate() {
                    let picked = v.champion_steps().len();
                    let side = &mut rosters.players[index(c.side)];
                    if !applied[i] && c.game == game && picked >= c.after && !side.is_empty() {
                        applied[i] = true;
                        let at = c.who.index(side.len());
                        if c.by {
                            side[at] = PlayerId(newcomer);
                            newcomer += 1;
                        } else {
                            side.remove(at);
                        }
                        v.set_rosters(rosters.clone()).unwrap();
                    }
                }
                match v.turn() {
                    Turn::Act { action: Action::Ban, sides, .. } => {
                        let choice = v.random_choice(sides[0], &ctx).unwrap();
                        let e = Event::Step { side: sides[0], action: Action::Ban, choice };
                        v.apply(e, Some(&ctx)).unwrap();
                        events.push(e);
                    }
                    Turn::Act { action: Action::TeamPick, .. } => {
                        let missing = v.missing();
                        let m = match order.next() {
                            Some(i) => missing[i.index(missing.len())],
                            None => missing[0],
                        };
                        let options = v.team_options(m.side, m.player, m.game, &ctx);
                        let c = v.random_team_choice(m.side, m.player, m.game, &ctx).unwrap();
                        prop_assert!(options.champions.contains(&c));
                        let picker = Picker { side: m.side, player: Some(m.player) };
                        let reason = v.unavailable(picker, m.game, &ctx)
                            .into_iter()
                            .find(|(id, _)| *id == c)
                            .map(|(_, r)| r);
                        match options.gave_way {
                            None => prop_assert_eq!(reason, None),
                            Some(GaveWay::Rule) => {
                                gave_way = true;
                                prop_assert!(matches!(reason, None | Some(Unavailable::Repeat)));
                            }
                            Some(GaveWay::Game) => gave_way = true,
                        }
                        let e = Event::TeamPick {
                            side: m.side,
                            player: m.player,
                            game: m.game,
                            champion: c,
                            gave_way: options.gave_way.is_some(),
                        };
                        v.apply(e, Some(&ctx)).unwrap();
                        events.push(e);
                    }
                    Turn::Done { .. } => break,
                    other => prop_assert!(false, "vez inesperada: {other:?}"),
                }
            }
            let banned: HashSet<ChampionId> = v.bans().iter().filter_map(Applied::champion).collect();
            prop_assert_eq!(banned.len(), 2 * usize::from(bans));
            // Cada um do elenco de agora tem um campeão em cada partida
            // aberta, nenhum banido.
            for h in game..=best_of {
                let live: Vec<Applied> = v
                    .team_picks(h)
                    .into_iter()
                    .filter(|a| a.player.is_some_and(|p| rosters.players[index(a.side)].contains(&p)))
                    .collect();
                for side in [A, B] {
                    let roster = &rosters.players[index(side)];
                    let mine: Vec<ChampionId> = live
                        .iter()
                        .filter(|a| a.side == side)
                        .filter_map(Applied::champion)
                        .collect();
                    prop_assert_eq!(mine.len(), roster.len());
                    prop_assert!(mine.iter().all(|c| !banned.contains(c)));
                    if !same_team && !gave_way {
                        let distinct: HashSet<ChampionId> = mine.iter().copied().collect();
                        prop_assert_eq!(distinct.len(), mine.len());
                    }
                }
                if !mirror && !gave_way {
                    let a: HashSet<ChampionId> = live.iter().filter(|x| x.side == A).filter_map(Applied::champion).collect();
                    let b: HashSet<ChampionId> = live.iter().filter(|x| x.side == B).filter_map(Applied::champion).collect();
                    prop_assert!(a.is_disjoint(&b));
                }
            }
        }
        // O histórico refeito, com o mesmo elenco de agora, dá o mesmo veto.
        let mut replayed = Veto::replay(rules, best_of, events).unwrap();
        replayed.set_rosters(rosters).unwrap();
        prop_assert_eq!(replayed, v);
    }
}
