//! Propriedades do draft do duelo: com qualquer escolha dentro das opções,
//! qualquer tamanho de série, bans, espelho, regra e uso de fora, o draft
//! acaba antes da partida 1 com um pick de cada lado por partida, nenhum
//! banido, sem espelho quando ele não vale e sem repetir pela regra quando
//! ela não cede; e o histórico refeito dá o mesmo veto.

use proptest::prelude::*;
use proptest::sample::Index;

use super::*;

proptest! {
    #[test]
    fn any_draft_completes_and_replays(
        seed in any::<u64>(),
        best_of in prop_oneof![Just(1u8), Just(3), Just(5), Just(7)],
        bans in 0u8..=3,
        mirror in any::<bool>(),
        rule in any::<bool>(),
        first_a in any::<bool>(),
        used_a in proptest::collection::vec(1i64..=17, 0..8),
        used_b in proptest::collection::vec(1i64..=17, 0..8),
        order in proptest::collection::vec(any::<Index>(), 0..30),
    ) {
        let champions = champions(17);
        let used = [ids(&used_a), ids(&used_b)];
        let ctx = Context {
            champions: &champions,
            used: [&used[0], &used[1]],
            used_by: &[],
        };
        let mut rules = rules(MapMode::Off, 0, true, bans);
        rules.champions.mirror = mirror;
        if rule {
            rules = with_rule(rules, crate::champions::NoRepeatWho::Player);
        }
        let begin = Event::Begin { seed, first: if first_a { A } else { B } };
        let mut events = vec![begin];
        let mut v = Veto::replay(rules.clone(), best_of, [begin]).unwrap();
        let mut order = order.into_iter();
        let mut gave_way = [false, false];
        while let Turn::Act { sides, action, pick_for, .. } = v.turn() {
            let side = sides[0];
            let options = v.options(side, &ctx);
            prop_assert!(!options.is_empty());
            let choice = match order.next() {
                Some(i) => options[i.index(options.len())],
                None => v.random_choice(side, &ctx).unwrap(),
            };
            if let Some(game) = pick_for {
                let picker = Picker { side, player: None };
                gave_way[index(side)] |= v.pick_options(picker, game, &ctx).gave_way.is_some();
            }
            let e = Event::Step { side, action, choice };
            v.apply(e, Some(&ctx)).unwrap();
            events.push(e);
        }
        prop_assert_eq!(v.turn(), Turn::Done { game: 1 });
        let banned: Vec<ChampionId> = v.bans().iter().filter_map(Applied::champion).collect();
        prop_assert_eq!(banned.len(), 2 * usize::from(bans));
        for side in [A, B] {
            let mine: Vec<ChampionId> =
                (1..=best_of).filter_map(|g| v.picks(g)[index(side)]).collect();
            prop_assert_eq!(mine.len(), usize::from(best_of));
            prop_assert!(mine.iter().all(|c| !banned.contains(c)));
            if rule && !gave_way[index(side)] {
                let mut distinct = mine.clone();
                distinct.sort();
                distinct.dedup();
                prop_assert_eq!(distinct.len(), mine.len());
                prop_assert!(mine.iter().all(|c| !used[index(side)].contains(c)));
            }
        }
        if !mirror && gave_way == [false, false] {
            for g in 1..=best_of {
                let [a, b] = v.picks(g);
                prop_assert_ne!(a, b);
            }
        }
        prop_assert_eq!(Veto::replay(rules, best_of, events).unwrap(), v);
    }
}
