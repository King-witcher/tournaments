//! A escolha simultânea de campeões nos times (TDM): opções, regras e
//! sorteio.

use std::collections::HashSet;

use super::*;
use crate::champions::NoRepeatWho;

fn rule(who: NoRepeatWho) -> Option<NoRepeatWho> {
    Some(who)
}

#[test]
fn captains_ban_then_everyone_picks_at_once() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(1, false);
    // Os bans alternam como no duelo (começa o seed melhor, fora do veto).
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    step_in(&mut v, B, champ(2), &ctx).unwrap();
    // Escolha simultânea: os dois lados, todos faltando.
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A, B]));
    assert_eq!(v.missing(1), [(A, P1), (A, P2), (B, P3), (B, P4)]);
    // O lado não escolhe por `options`: é por jogador.
    assert!(v.options(A, &ctx).is_empty());
    let options = v.team_options(A, P1, &ctx);
    assert_eq!(options.champions.len(), 15);
    assert_eq!(options.gave_way, None);
    // Sem ordem: B primeiro.
    pick(&mut v, B, P4, 3, &ctx).unwrap();
    pick(&mut v, A, P2, 3, &ctx).unwrap();
    // O ban vale para todos.
    assert_eq!(pick(&mut v, A, P1, 1, &ctx), Err(VetoError::OptionInvalid));
    // Sem repetir no time: P1 não pega o 3 de P2; espelho entre times vale.
    assert_eq!(pick(&mut v, A, P1, 3, &ctx), Err(VetoError::OptionInvalid));
    assert!(
        !v.team_options(A, P1, &ctx)
            .champions
            .contains(&ChampionId(103))
    );
    pick(&mut v, A, P1, 4, &ctx).unwrap();
    // A acabou; B ainda tem o 3.
    assert_eq!(acting(&v).2, vec![B]);
    // Escolha definitiva.
    assert_eq!(pick(&mut v, A, P1, 5, &ctx), Err(VetoError::AlreadyPicked));
    pick(&mut v, B, P3, 4, &ctx).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert!(v.missing(1).is_empty());
    let picks: Vec<(Side, Option<PlayerId>)> =
        v.team_picks(1).iter().map(|a| (a.side, a.player)).collect();
    assert_eq!(
        picks,
        [(B, Some(P4)), (A, Some(P2)), (A, Some(P1)), (B, Some(P3))]
    );
    // Picks de lado (duelo) não existem nos times.
    assert_eq!(v.picks(1), [None, None]);
}

#[test]
fn only_the_roster_picks_and_a_side_pick_is_refused() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(0, false);
    assert_eq!(pick(&mut v, A, P3, 1, &ctx), Err(VetoError::NotYourTurn));
    assert_eq!(pick(&mut v, A, P5, 1, &ctx), Err(VetoError::NotYourTurn));
    assert_eq!(
        step_in(&mut v, A, champ(1), &ctx),
        Err(VetoError::WrongAction)
    );
    // No duelo não há escolha simultânea.
    let mut duel = begun(rules(MapMode::Off, 0, true, 0), 3, A);
    duel.set_rosters(open(&[P1], &[P3])).unwrap();
    assert_eq!(pick(&mut duel, A, P1, 1, &ctx), Err(VetoError::WrongAction));
}

#[test]
fn the_roster_is_checked_before_a_repeated_pick() {
    // P3 é de B: escolhendo por A depois de já ter escolhido por B, a
    // resposta é que ele não é do elenco de A, não que já escolheu.
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(0, false);
    pick(&mut v, B, P3, 1, &ctx).unwrap();
    assert_eq!(pick(&mut v, A, P3, 2, &ctx), Err(VetoError::NotYourTurn));
    // Quem saiu do elenco depois de escolher também.
    pick(&mut v, A, P1, 2, &ctx).unwrap();
    v.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    assert_eq!(pick(&mut v, A, P1, 3, &ctx), Err(VetoError::NotYourTurn));
    assert_eq!(pick(&mut v, B, P3, 3, &ctx), Err(VetoError::AlreadyPicked));
}

#[test]
fn duplicates_in_the_team_can_be_allowed() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(0, true);
    pick(&mut v, A, P1, 1, &ctx).unwrap();
    assert!(
        v.team_options(A, P2, &ctx)
            .champions
            .contains(&ChampionId(101))
    );
    pick(&mut v, A, P2, 1, &ctx).unwrap();
}

#[test]
fn the_player_rule_follows_each_player() {
    let champions = champions(17);
    let elsewhere = [(P2, ChampionId(109))];
    let ctx = Context {
        champions: &champions,
        no_repeat: rule(NoRepeatWho::Player),
        used: [&[], &[]],
        used_by: &elsewhere,
    };
    let mut v = two_v_two(0, false);
    // P2 usou o 9 em outra partida do escopo.
    assert_eq!(
        pick(&mut v, A, P2, 9, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    pick(&mut v, A, P1, 9, &ctx).unwrap();
    pick(&mut v, A, P2, 2, &ctx).unwrap();
    pick(&mut v, B, P3, 1, &ctx).unwrap();
    pick(&mut v, B, P4, 2, &ctx).unwrap();
    next_game(&mut v, 2);
    // No jogo 2, cada um perde o próprio: P1 o 9, P2 o 2 (e o 9 de fora).
    assert_eq!(
        pick(&mut v, A, P1, 9, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    pick(&mut v, A, P2, 1, &ctx).unwrap();
    pick(&mut v, A, P1, 2, &ctx).unwrap();
    let both: HashSet<ChampionId> = [ChampionId(109), ChampionId(102)].into();
    assert_eq!(v.forbidden_for(A, P2, &ctx), both);
    // O lado, pela regra do jogador: o que ela tira de algum do elenco.
    assert!(v.forbidden(A, &ctx).is_superset(&both));
}

#[test]
fn the_team_rule_takes_what_anyone_of_the_team_used() {
    let champions = champions(17);
    let elsewhere = [ChampionId(109)];
    let ctx = Context {
        champions: &champions,
        no_repeat: rule(NoRepeatWho::Team),
        used: [&elsewhere, &[]],
        used_by: &[],
    };
    let mut v = two_v_two(0, false);
    assert_eq!(
        pick(&mut v, A, P1, 9, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    pick(&mut v, A, P1, 1, &ctx).unwrap();
    pick(&mut v, A, P2, 2, &ctx).unwrap();
    pick(&mut v, B, P3, 1, &ctx).unwrap();
    pick(&mut v, B, P4, 2, &ctx).unwrap();
    next_game(&mut v, 2);
    // O 1 de P1 sai também de P2.
    assert_eq!(
        pick(&mut v, A, P2, 1, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    let options = v.team_options(A, P2, &ctx).champions;
    assert!(!options.contains(&ChampionId(101)) && !options.contains(&ChampionId(102)));
    // B escolheu os mesmos, mas a regra é por time.
    assert!(
        v.team_options(B, P3, &ctx)
            .champions
            .contains(&ChampionId(109))
    );
}

#[test]
fn the_team_rule_uses_the_team_and_ignores_each_player() {
    // Pela regra do time, `used` (o do time) vale e `used_by` (por jogador,
    // de quem jogou por outro time antes) não.
    let champions = champions(17);
    let team = [ChampionId(105)];
    let by_player = [(P1, ChampionId(106)), (P3, ChampionId(105))];
    let ctx = Context {
        champions: &champions,
        no_repeat: rule(NoRepeatWho::Team),
        used: [&team, &[]],
        used_by: &by_player,
    };
    let mut v = two_v_two(0, false);
    assert_eq!(
        v.forbidden_for(A, P1, &ctx),
        HashSet::from([ChampionId(105)])
    );
    assert_eq!(
        pick(&mut v, A, P1, 5, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    pick(&mut v, A, P1, 6, &ctx).unwrap();
    pick(&mut v, B, P3, 5, &ctx).unwrap();
    pick(&mut v, A, P2, 7, &ctx).unwrap();
    pick(&mut v, B, P4, 7, &ctx).unwrap();
    next_game(&mut v, 2);
    // No jogo 2, A perde o de fora e os dois do jogo 1.
    assert_eq!(
        v.forbidden_for(A, P2, &ctx),
        HashSet::from([ChampionId(105), ChampionId(106), ChampionId(107)])
    );
    assert_eq!(
        v.forbidden_for(B, P4, &ctx),
        HashSet::from([ChampionId(105), ChampionId(107)])
    );
}

#[test]
fn the_rule_gives_way_but_the_team_still_does_not_repeat() {
    // Pool de 3, regra do time: A já usou os três fora da série.
    let champions = champions(3);
    let used = [ChampionId(101), ChampionId(102), ChampionId(103)];
    let ctx = Context {
        champions: &champions,
        no_repeat: rule(NoRepeatWho::Team),
        used: [&used, &[]],
        used_by: &[],
    };
    let mut v = two_v_two(0, false);
    assert_eq!(v.team_options(A, P1, &ctx).gave_way, Some(GaveWay::Rule));
    pick(&mut v, A, P1, 1, &ctx).unwrap();
    // A regra cede, mas o 1 de P1 continua fora.
    assert_eq!(
        v.team_options(A, P2, &ctx),
        TeamOptions {
            champions: ids(&[2, 3]),
            gave_way: Some(GaveWay::Rule),
        }
    );
}

#[test]
fn the_rule_gives_way_for_part_of_the_team_only() {
    // Pool de 3, regra do time: A usou o 1 e o 2 fora da série. Sobra o 3
    // para um só: quem escolhe primeiro fica com ele, pela regra; o outro
    // fica com a regra cedida (os dois que ela tiraria), e a tela avisa só
    // ele.
    let champions = champions(3);
    let used = [ChampionId(101), ChampionId(102)];
    let ctx = Context {
        champions: &champions,
        no_repeat: rule(NoRepeatWho::Team),
        used: [&used, &[]],
        used_by: &[],
    };
    let mut v = two_v_two(0, false);
    for p in [P1, P2] {
        assert_eq!(
            v.team_options(A, p, &ctx),
            TeamOptions {
                champions: ids(&[3]),
                gave_way: None,
            }
        );
    }
    pick(&mut v, A, P2, 3, &ctx).unwrap();
    assert_eq!(
        v.team_options(A, P1, &ctx),
        TeamOptions {
            champions: ids(&[1, 2]),
            gave_way: Some(GaveWay::Rule),
        }
    );
    // B, sem nada usado, não cede.
    assert_eq!(v.team_options(B, P3, &ctx).gave_way, None);
    pick(&mut v, A, P1, 1, &ctx).unwrap();
}

#[test]
fn with_nothing_left_the_team_repeats() {
    // 3 campeões e 1 ban de cada lado: sobra 1 para os dois de A, que
    // repetem.
    let champions = champions(3);
    let ctx = free(&champions);
    let mut v = two_v_two(1, false);
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    step_in(&mut v, B, champ(2), &ctx).unwrap();
    pick(&mut v, A, P1, 3, &ctx).unwrap();
    assert_eq!(
        v.team_options(A, P2, &ctx),
        TeamOptions {
            champions: ids(&[3]),
            gave_way: Some(GaveWay::Team),
        }
    );
    pick(&mut v, A, P2, 3, &ctx).unwrap();
}

#[test]
fn a_substitute_may_take_the_champion_of_who_left() {
    // P1 escolheu o 1 e saiu antes do envio; P5 entra. A escolha de P1 não
    // foi jogada: no mesmo mapa, não conta para ninguém do time.
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(0, false);
    pick(&mut v, A, P1, 1, &ctx).unwrap();
    assert!(
        !v.team_options(A, P2, &ctx)
            .champions
            .contains(&ChampionId(101))
    );
    v.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    assert_eq!(v.missing(1), [(A, P5), (A, P2), (B, P3), (B, P4)]);
    assert!(
        v.team_options(A, P2, &ctx)
            .champions
            .contains(&ChampionId(101))
    );
    pick(&mut v, A, P5, 1, &ctx).unwrap();
    // P5 e P2 não repetem entre si.
    assert_eq!(pick(&mut v, A, P2, 1, &ctx), Err(VetoError::OptionInvalid));
    pick(&mut v, A, P2, 2, &ctx).unwrap();
    // A escolha de P1 continua no histórico.
    assert_eq!(picks_of(&v, 1, A), ids(&[1, 1, 2]));
}

#[test]
fn across_maps_each_rule_counts_who_left_its_own_way() {
    // Jogo 1: P1 escolhe o 1 e P2 o 2; depois do envio, P1 sai e P5 entra.
    let champions = champions(17);
    let player = Context {
        champions: &champions,
        no_repeat: rule(NoRepeatWho::Player),
        used: [&[], &[]],
        used_by: &[],
    };
    let team = Context {
        no_repeat: rule(NoRepeatWho::Team),
        ..player
    };
    let mut v = two_v_two(0, false);
    pick(&mut v, A, P1, 1, &player).unwrap();
    pick(&mut v, A, P2, 2, &player).unwrap();
    pick(&mut v, B, P3, 1, &player).unwrap();
    pick(&mut v, B, P4, 2, &player).unwrap();
    next_game(&mut v, 2);
    v.set_rosters(Rosters {
        sent: 1,
        ..open(&[P5, P2], &[P3, P4])
    })
    .unwrap();
    // Pela do jogador, P5 não usou nada; pela do time, o 1 de P1 conta.
    assert!(v.forbidden_for(A, P5, &player).is_empty());
    assert_eq!(
        v.forbidden_for(A, P5, &team),
        HashSet::from([ChampionId(101), ChampionId(102)])
    );
}

#[test]
fn a_timeout_draws_for_whoever_is_missing() {
    let champions = champions(17);
    let ctx = free(&champions);
    for seed in 0..30 {
        let mut v = Veto::replay(
            team_rules(1, 3, false),
            3,
            [Event::Begin { seed, first: A }],
        )
        .unwrap();
        v.set_rosters(open(&[P1, P2, P3], &[P4, P5])).unwrap();
        step_in(&mut v, A, champ(1), &ctx).unwrap();
        step_in(&mut v, B, champ(2), &ctx).unwrap();
        pick(&mut v, A, P2, 3, &ctx).unwrap();
        // O prazo acaba: um de cada vez, na ordem de `missing`.
        for (side, player) in v.missing(1) {
            let c = v.random_team_choice(side, player, &ctx).unwrap();
            assert_eq!(v.random_team_choice(side, player, &ctx), Some(c));
            assert!(![ChampionId(101), ChampionId(102)].contains(&c));
            v.apply(
                Event::TeamPick {
                    side,
                    player,
                    champion: c,
                },
                Some(&ctx),
            )
            .unwrap();
        }
        assert_eq!(v.turn(), Turn::Done { game: 1 });
        let mine = picks_of(&v, 1, A);
        let distinct: HashSet<ChampionId> = mine.iter().copied().collect();
        assert_eq!(distinct.len(), 3, "A repetiu no time: {mine:?}");
    }
}

#[test]
fn the_draw_is_pinned() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = Veto::replay(
        team_rules(0, 2, false),
        1,
        [Event::Begin {
            seed: 2026,
            first: A,
        }],
    )
    .unwrap();
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    // Sorteios gravados dependem deste resultado.
    assert_eq!(v.random_team_choice(A, P1, &ctx), Some(ChampionId(112)));
}

#[test]
fn the_roster_changes_but_the_history_replays() {
    let events = [
        Event::Begin { seed: 42, first: A },
        Event::TeamPick {
            side: A,
            player: P1,
            champion: ChampionId(101),
        },
        Event::TeamPick {
            side: A,
            player: P2,
            champion: ChampionId(102),
        },
    ];
    // P1 foi substituído por P5: o histórico refaz sem olhar o elenco, e P5
    // passa a faltar.
    let mut r = Veto::replay(team_rules(0, 2, false), 3, events).unwrap();
    r.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    assert_eq!(r.missing(1), [(A, P5), (B, P3), (B, P4)]);
    // Quem saiu da comunidade sai do elenco: B só com P3.
    r.set_rosters(open(&[P1, P2], &[P3])).unwrap();
    assert_eq!(r.missing(1), [(B, P3)]);
    let champions = champions(17);
    pick(&mut r, B, P3, 1, &free(&champions)).unwrap();
    assert_eq!(r.turn(), Turn::Done { game: 1 });
    // A mesma escolha repetida não refaz.
    let twice = [
        Event::Begin { seed: 42, first: A },
        Event::TeamPick {
            side: A,
            player: P1,
            champion: ChampionId(101),
        },
        Event::TeamPick {
            side: A,
            player: P1,
            champion: ChampionId(103),
        },
    ];
    assert_eq!(
        Veto::replay(team_rules(0, 2, false), 3, twice),
        Err(VetoError::AlreadyPicked)
    );
}

#[test]
fn undo_takes_back_the_last_pick() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(1, false);
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    step_in(&mut v, B, champ(2), &ctx).unwrap();
    pick(&mut v, B, P3, 5, &ctx).unwrap();
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(v.missing(1).len(), 4);
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![B]));
}

#[test]
fn undo_after_everyone_picked_reopens_the_last() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(0, false);
    pick(&mut v, A, P1, 1, &ctx).unwrap();
    pick(&mut v, B, P3, 1, &ctx).unwrap();
    pick(&mut v, B, P4, 2, &ctx).unwrap();
    pick(&mut v, A, P2, 2, &ctx).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert!(v.can_undo());
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A]));
    assert_eq!(v.missing(1), [(A, P2)]);
    // P2 escolhe outro.
    pick(&mut v, A, P2, 3, &ctx).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert_eq!(picks_of(&v, 1, A), ids(&[1, 3]));
}

#[test]
fn the_map_veto_comes_first_in_teams() {
    // Modo veto, md1 com 3 mapas: os capitães banem dois mapas, depois um
    // ban de campeão de cada lado e a escolha simultânea.
    let champions = champions(17);
    let ctx = free(&champions);
    let rules = VetoRules {
        team_size: 2,
        ..rules(MapMode::Veto, 3, true, 1)
    };
    let mut v = begun(rules, 1, A);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Map, vec![A]));
    assert_eq!(pick(&mut v, A, P1, 1, &ctx), Err(VetoError::WrongAction));
    step_in(&mut v, A, map(1), &ctx).unwrap();
    step_in(&mut v, B, map(2), &ctx).unwrap();
    assert_eq!(v.map_of(1), Some(MapId(3)));
    // O decisivo: começa quem começou o veto.
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    step_in(&mut v, A, champ(1), &ctx).unwrap();
    step_in(&mut v, B, champ(2), &ctx).unwrap();
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A, B]));
    for (side, player, c) in [(A, P1, 3), (A, P2, 4), (B, P3, 3), (B, P4, 4)] {
        pick(&mut v, side, player, c, &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert_eq!(v.map_steps().len(), 2);
}

#[test]
fn the_vote_comes_first_in_teams() {
    // Votação, md3 com 5 mapas: os capitães votam, depois a escolha
    // simultânea (sem bans).
    let champions = champions(17);
    let ctx = free(&champions);
    let rules = VetoRules {
        team_size: 2,
        ..rules(MapMode::Vote, 5, true, 0)
    };
    let mut v = begun(rules, 3, A);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    assert_eq!(acting(&v), (Action::Vote, Target::Map, vec![A, B]));
    assert!(v.missing(1).is_empty());
    let offered = v.offered(1);
    assert_eq!(offered.len(), 3);
    step_in(&mut v, B, Choice::Map(offered[0]), &ctx).unwrap();
    step_in(&mut v, A, Choice::Map(offered[0]), &ctx).unwrap();
    assert_eq!(v.map_of(1), Some(offered[0]));
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A, B]));
    for (side, player, c) in [(B, P4, 1), (A, P1, 1), (A, P2, 2), (B, P3, 2)] {
        pick(&mut v, side, player, c, &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    // O jogo 2 vota de novo, sem o mapa do 1.
    next_game(&mut v, 2);
    assert_eq!(acting(&v), (Action::Vote, Target::Map, vec![A, B]));
    assert!(!v.offered(2).contains(&offered[0]));
}
