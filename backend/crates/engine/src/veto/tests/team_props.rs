//! Propriedades da escolha simultânea: qualquer ordem, regra, uso de fora
//! da série e troca de elenco no meio do jogo fecha o jogo sem campeão
//! banido, sem repetir no time (sem `team_duplicates`), respeitando a regra
//! quando ela não cede, e o histórico refeito dá o mesmo veto.

use std::collections::HashSet;

use proptest::prelude::*;
use proptest::sample::Index;

use super::*;
use crate::champions::NoRepeatWho;

/// Uma troca no elenco: no jogo `game`, depois de `after` escolhas nele, o
/// jogador `who` do lado `side` sai; com `by`, entra outro no lugar.
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
        0usize..8,
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
        duplicates in any::<bool>(),
        used_a in proptest::collection::vec(1i64..=17, 0..6),
        used_b in proptest::collection::vec(1i64..=17, 0..6),
        used_by in proptest::collection::vec(
            (prop_oneof![1i64..=4, 11i64..=14, 21i64..=22], 1i64..=17),
            0..8,
        ),
        games in 1u8..=3,
        order in proptest::collection::vec(any::<Index>(), 0..60),
        changes in proptest::collection::vec(change(), 0..3),
    ) {
        let champions = champions(17);
        let used = [ids(&used_a), ids(&used_b)];
        let used_by: Vec<(PlayerId, ChampionId)> = used_by
            .iter()
            .map(|(p, c)| (PlayerId(*p), ChampionId(100 + c)))
            .collect();
        let ctx = Context {
            champions: &champions,
            no_repeat: who,
            used: [&used[0], &used[1]],
            used_by: &used_by,
        };
        let rules = team_rules(bans, 4, duplicates);
        let begin = Event::Begin { seed, first: A };
        let mut events = vec![begin];
        let mut v = Veto::replay(rules.clone(), 5, [begin]).unwrap();
        let mut rosters = Rosters {
            players: [
                (1..=size_a as i64).map(PlayerId).collect(),
                (11..=10 + size_b as i64).map(PlayerId).collect(),
            ],
            sent: 0,
            finished: false,
        };
        v.set_rosters(rosters.clone()).unwrap();
        let mut order = order.into_iter();
        let mut newcomer = 30;
        let mut applied = vec![false; changes.len()];
        for game in 1..=games {
            if game > 1 {
                rosters.sent = game - 1;
                v.set_rosters(rosters.clone()).unwrap();
                let e = Event::Game { game, starter: A, previous: None };
                v.apply(e, None).unwrap();
                events.push(e);
            }
            loop {
                for (i, c) in changes.iter().enumerate() {
                    let picked = v.team_picks(game).len();
                    let side = &mut rosters.players[index(c.side)];
                    if !applied[i] && c.game == game && picked == c.after && !side.is_empty() {
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
                        let missing = v.missing(game);
                        let (side, player) = match order.next() {
                            Some(i) => missing[i.index(missing.len())],
                            None => missing[0],
                        };
                        let options = v.team_options(side, player, &ctx);
                        let c = v.random_team_choice(side, player, &ctx).unwrap();
                        prop_assert!(options.champions.contains(&c));
                        if options.gave_way.is_none() {
                            prop_assert!(!v.forbidden_for(side, player, &ctx).contains(&c));
                        }
                        let e = Event::TeamPick { side, player, champion: c };
                        v.apply(e, Some(&ctx)).unwrap();
                        events.push(e);
                    }
                    Turn::Done { .. } => break,
                    other => prop_assert!(false, "vez inesperada: {other:?}"),
                }
            }
            let banned = v.banned(game);
            let picks = v.team_picks(game);
            prop_assert!(picks.iter().all(|a| matches!(a.choice, Choice::Champion(c) if !banned.contains(&c))));
            for side in [A, B] {
                // Cada um do elenco de agora escolheu uma vez; a escolha de
                // quem saiu fica no histórico e não conta para a repetição.
                let roster = &rosters.players[index(side)];
                let mine: Vec<ChampionId> = picks
                    .iter()
                    .filter(|a| a.side == side && a.player.is_some_and(|p| roster.contains(&p)))
                    .filter_map(|a| match a.choice { Choice::Champion(c) => Some(c), Choice::Map(_) => None })
                    .collect();
                prop_assert_eq!(mine.len(), roster.len());
                if !duplicates {
                    let distinct: HashSet<ChampionId> = mine.iter().copied().collect();
                    prop_assert_eq!(distinct.len(), mine.len());
                }
            }
        }
        // O histórico refeito, com o mesmo elenco de agora, dá o mesmo veto.
        let mut replayed = Veto::replay(rules, 5, events).unwrap();
        replayed.set_rosters(rosters).unwrap();
        prop_assert_eq!(replayed, v);
    }
}
