//! A janela simultânea de campeões nos times (TDM): todos escolhem o
//! campeão de cada partida da série ao mesmo tempo; regras, cessão, troca de
//! elenco e sorteio.

use std::collections::HashSet;

use super::*;
use crate::champions::NoRepeatWho;

/// 2v2 sem bans, sem repetir no time, com a regra de `who`.
fn ruled_teams(who: NoRepeatWho, same_team: bool) -> VetoRules {
    with_rule(team_rules(0, 2, same_team), who)
}

fn picker(side: Side, player: PlayerId) -> Picker {
    Picker {
        side,
        player: Some(player),
    }
}

#[test]
fn bans_then_everyone_picks_every_game_at_once() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(team_rules(1, 2, false), 3);
    // Os bans alternam como no duelo, a partir do seed pior (B).
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![B]));
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    // A janela: os dois lados, cada jogador em cada partida.
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A, B]));
    assert_eq!(v.missing().len(), 12);
    // O lado não escolhe por `options`: é por jogador e partida.
    assert!(v.options(A, &ctx).is_empty());
    assert_eq!(team_options(&v, A, P1, 1, &ctx).len(), 15);
    // Sem ordem: B começa pela partida 3.
    pick(&mut v, B, P4, 3, 3, &ctx).unwrap();
    pick(&mut v, A, P2, 1, 3, &ctx).unwrap();
    // O ban vale para todas as partidas.
    assert_eq!(
        pick(&mut v, A, P1, 2, 1, &ctx),
        Err(VetoError::OptionInvalid)
    );
    // Sem repetir no time: P1 não pega o 3 de P2 na partida 1, mas pega na 2.
    assert_eq!(
        pick(&mut v, A, P1, 1, 3, &ctx),
        Err(VetoError::ChampionTaken)
    );
    assert!(!team_options(&v, A, P1, 1, &ctx).contains(&ChampionId(103)));
    pick(&mut v, A, P1, 2, 3, &ctx).unwrap();
    // Escolha definitiva.
    assert_eq!(
        pick(&mut v, A, P1, 2, 4, &ctx),
        Err(VetoError::AlreadyPicked)
    );
    for (side, player, game, c) in [
        (A, P1, 1, 4),
        (A, P1, 3, 5),
        (A, P2, 2, 4),
        (A, P2, 3, 6),
        (B, P3, 1, 3),
        (B, P3, 2, 3),
        (B, P3, 3, 4),
        (B, P4, 1, 4),
    ] {
        pick(&mut v, side, player, game, c, &ctx).unwrap();
    }
    // A acabou; B ainda tem P4 na partida 2.
    assert_eq!(missing(&v), [(B, P4, 2)]);
    assert_eq!(acting(&v).2, vec![B]);
    pick(&mut v, B, P4, 2, 5, &ctx).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert!(v.missing().is_empty());
    assert_eq!(picks_of(&v, 1, A), ids(&[3, 4]));
    assert_eq!(picks_of(&v, 1, B), ids(&[3, 4]));
    // Picks de lado (duelo) não existem nos times.
    assert_eq!(v.picks(1), [None, None]);
    // Ninguém volta entre as partidas.
    next_game(&mut v, 2);
    v.set_rosters(sent(&[P1, P2], &[P3, P4], 1, [&[P1, P2], &[P3, P4]]))
        .unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 2 });
}

#[test]
fn without_mirror_whoever_confirms_first_keeps_it_across_teams() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(no_mirror(team_rules(0, 2, false)), 1);
    pick(&mut v, B, P3, 1, 1, &ctx).unwrap();
    assert_eq!(
        pick(&mut v, A, P1, 1, 1, &ctx),
        Err(VetoError::ChampionTaken)
    );
    assert_eq!(
        v.unavailable(picker(A, P1), 1, &ctx),
        [(ChampionId(101), Unavailable::Opponent)]
    );
    pick(&mut v, A, P1, 1, 2, &ctx).unwrap();
    assert_eq!(
        v.unavailable(picker(A, P2), 1, &ctx),
        [
            (ChampionId(101), Unavailable::Opponent),
            (ChampionId(102), Unavailable::Teammate)
        ]
    );
}

#[test]
fn same_team_lets_teammates_repeat_in_a_game() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(team_rules(0, 2, true), 1);
    for (side, player) in [(A, P1), (A, P2), (B, P3), (B, P4)] {
        pick(&mut v, side, player, 1, 1, &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert_eq!(picks_of(&v, 1, A), ids(&[1, 1]));
}

#[test]
fn the_player_rule_follows_each_player_across_the_series() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(ruled_teams(NoRepeatWho::Player, false), 3);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    // P1 não repete o 1 em outra partida; P2 pode, numa em que P1 não o tem.
    assert_eq!(
        pick(&mut v, A, P1, 3, 1, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    pick(&mut v, A, P2, 2, 1, &ctx).unwrap();
    // De fora da série, o que cada um jogou.
    let used_by = [(P3, ChampionId(105))];
    let outside = Context {
        used_by: &used_by,
        ..ctx
    };
    assert_eq!(
        pick(&mut v, B, P3, 1, 5, &outside),
        Err(VetoError::ChampionForbidden)
    );
    pick(&mut v, B, P4, 1, 5, &outside).unwrap();
}

#[test]
fn the_team_rule_takes_what_anyone_of_the_team_picked_for_another_game() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(ruled_teams(NoRepeatWho::Team, true), 3);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    // Na mesma partida, o colega pode repetir (`same_team`)...
    pick(&mut v, A, P2, 1, 1, &ctx).unwrap();
    // ...mas ninguém do time o usa em outra.
    assert_eq!(
        pick(&mut v, A, P2, 2, 1, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    assert_eq!(
        v.unavailable(picker(A, P1), 3, &ctx),
        [(ChampionId(101), Unavailable::Repeat)]
    );
    // O outro time não tem nada com isso.
    pick(&mut v, B, P3, 2, 1, &ctx).unwrap();
    // De fora da série, o que o time usou.
    let used = [ChampionId(107)];
    let outside = Context {
        used: [&used, &[]],
        ..ctx
    };
    assert_eq!(
        pick(&mut v, A, P2, 2, 7, &outside),
        Err(VetoError::ChampionForbidden)
    );
}

#[test]
fn the_rule_gives_way_for_one_player_only() {
    // Pool de 4. P1 já jogou com 1, 2 e 3 em outras séries: só sobra o 4.
    let champions = champions(4);
    let used_by = [
        (P1, ChampionId(101)),
        (P1, ChampionId(102)),
        (P1, ChampionId(103)),
    ];
    let ctx = Context {
        used_by: &used_by,
        ..free(&champions)
    };
    let mut v = two_v_two(ruled_teams(NoRepeatWho::Player, false), 1);
    assert_eq!(team_options(&v, A, P1, 1, &ctx), ids(&[4]));
    // P2 pega o 4 primeiro: para P1, a regra cede (o 4 é do colega).
    pick(&mut v, A, P2, 1, 4, &ctx).unwrap();
    let options = v.team_options(A, P1, 1, &ctx);
    assert_eq!(options.champions, ids(&[1, 2, 3]));
    assert_eq!(options.gave_way, Some(GaveWay::Rule));
    // P3, sem nada usado, escolhe livre.
    assert_eq!(v.team_options(B, P3, 1, &ctx).gave_way, None);
    pick(&mut v, A, P1, 1, 2, &ctx).unwrap();
    assert_eq!(picks_of(&v, 1, A), ids(&[4, 2]));
}

#[test]
fn with_nothing_left_the_team_repeats_and_both_count() {
    // Pool de 3, 1 ban de cada lado: sobra um para dois colegas.
    let champions = champions(3);
    let ctx = free(&champions);
    let mut v = two_v_two(team_rules(1, 2, false), 1);
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    pick(&mut v, A, P1, 1, 3, &ctx).unwrap();
    let options = v.team_options(A, P2, 1, &ctx);
    assert_eq!(options.champions, ids(&[3]));
    assert_eq!(options.gave_way, Some(GaveWay::Game));
    pick(&mut v, A, P2, 1, 3, &ctx).unwrap();
    // O pick que cedeu não derruba o do colega: os dois contam.
    assert_eq!(picks_of(&v, 1, A), ids(&[3, 3]));
    assert!(v.team_picks(1).iter().any(|a| a.gave_way));
    assert_eq!(missing(&v), [(B, P3, 1), (B, P4, 1)]);
}

#[test]
fn who_left_does_not_count_in_an_open_game_but_does_in_a_sent_one() {
    // P1 escolhe o 1, o 2 e o 3; P2, o 4, o 5 e o 6. Depois do envio da
    // partida 1 (jogada por P1 e P2), P1 sai e P5 entra.
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(ruled_teams(NoRepeatWho::Team, false), 3);
    for (side, player, game, c) in [
        (A, P1, 1, 1),
        (A, P1, 2, 2),
        (A, P1, 3, 3),
        (A, P2, 1, 4),
        (A, P2, 2, 5),
        (A, P2, 3, 6),
        (B, P3, 1, 1),
        (B, P3, 2, 2),
        (B, P3, 3, 3),
        (B, P4, 1, 4),
        (B, P4, 2, 5),
        (B, P4, 3, 6),
    ] {
        pick(&mut v, side, player, game, c, &ctx).unwrap();
    }
    next_game(&mut v, 2);
    v.set_rosters(sent(&[P5, P2], &[P3, P4], 1, [&[P1, P2], &[P3, P4]]))
        .unwrap();
    assert_eq!(missing(&v), [(A, P5, 2), (A, P5, 3)]);
    // Pela regra do time, o 1 de P1 foi jogado e conta; o 2 e o 3, não: P1
    // não joga mais. Os de P2 contam nas outras partidas.
    assert_eq!(
        v.unavailable(picker(A, P5), 2, &ctx),
        [
            (ChampionId(101), Unavailable::Repeat),
            (ChampionId(104), Unavailable::Repeat),
            (ChampionId(105), Unavailable::Teammate),
            (ChampionId(106), Unavailable::Repeat),
        ]
    );
    pick(&mut v, A, P5, 2, 2, &ctx).unwrap();
    assert_eq!(picks_of(&v, 2, A), ids(&[5, 2]));
    assert_eq!(v.played_picks(1).len(), 4);
    assert!(v.played_picks(2).is_empty());
}

#[test]
fn who_left_before_the_send_does_not_count_after_it() {
    // P1 escolhe o 1 para a partida 1 e sai antes do envio; P5 entra e
    // escolhe o 7. A partida 1 é jogada por P5 e P2; depois, P1 volta.
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(ruled_teams(NoRepeatWho::Team, false), 3);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    v.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    pick(&mut v, A, P5, 1, 7, &ctx).unwrap();
    v.set_rosters(sent(&[P1, P2], &[P3, P4], 1, [&[P5, P2], &[P3, P4]]))
        .unwrap();
    // O 1 de P1 não foi jogado: nem a API o conta fora da série, nem a regra
    // do time o tira das outras partidas; o 7 de P5 foi e tira.
    let played: Vec<ChampionId> = v
        .played_picks(1)
        .iter()
        .filter_map(Applied::champion)
        .collect();
    assert_eq!(played, ids(&[7]));
    let reasons = v.unavailable(picker(A, P1), 2, &ctx);
    assert_eq!(reasons, [(ChampionId(107), Unavailable::Repeat)]);
}

#[test]
fn who_comes_back_loses_a_pick_taken_meanwhile() {
    // P1 escolhe o 1 para a partida 2 e sai; P5 entra e, sem P1, pega o 1
    // para a mesma partida. P1 volta no lugar de P2: o pick antigo não cabe
    // mais (sem repetir no time) e P1 passa a faltar na partida 2.
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(team_rules(0, 2, false), 3);
    pick(&mut v, A, P1, 2, 1, &ctx).unwrap();
    v.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    pick(&mut v, A, P5, 2, 1, &ctx).unwrap();
    v.set_rosters(open(&[P1, P5], &[P3, P4])).unwrap();
    assert!(missing(&v).contains(&(A, P1, 2)));
    assert_eq!(picks_of(&v, 2, A), ids(&[1]));
    // Ele escolhe de novo, e o replay aceita o novo pick dele.
    assert_eq!(
        pick(&mut v, A, P1, 2, 1, &ctx),
        Err(VetoError::ChampionTaken)
    );
    pick(&mut v, A, P1, 2, 8, &ctx).unwrap();
    assert_eq!(picks_of(&v, 2, A), ids(&[1, 8]));
    // Com o pick antigo ainda cabendo, ele volta a contar.
    let mut w = two_v_two(team_rules(0, 2, false), 3);
    pick(&mut w, A, P1, 2, 1, &ctx).unwrap();
    w.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    pick(&mut w, A, P5, 2, 9, &ctx).unwrap();
    w.set_rosters(open(&[P1, P5], &[P3, P4])).unwrap();
    assert!(!missing(&w).contains(&(A, P1, 2)));
    assert_eq!(picks_of(&w, 2, A), ids(&[1, 9]));
}

#[test]
fn a_timeout_draws_for_whoever_is_missing() {
    let champions = champions(17);
    let ctx = free(&champions);
    for seed in 0..30 {
        let rules = no_mirror(with_rule(team_rules(1, 3, false), NoRepeatWho::Player));
        let mut v = Veto::replay(rules, 3, [Event::Begin { seed, first: A }]).unwrap();
        v.set_rosters(open(&[P1, P2, P3], &[P4, P5])).unwrap();
        step_in(&mut v, B, champ(1), &ctx).unwrap();
        step_in(&mut v, A, champ(2), &ctx).unwrap();
        pick(&mut v, A, P2, 2, 3, &ctx).unwrap();
        // O prazo acaba: um de cada vez, na ordem de `missing`.
        for m in v.missing() {
            let c = v
                .random_team_choice(m.side, m.player, m.game, &ctx)
                .unwrap();
            assert_eq!(
                v.random_team_choice(m.side, m.player, m.game, &ctx),
                Some(c)
            );
            let event = Event::TeamPick {
                side: m.side,
                player: m.player,
                game: m.game,
                champion: c,
                gave_way: false,
            };
            v.apply(event, Some(&ctx)).unwrap();
        }
        assert_eq!(v.turn(), Turn::Done { game: 1 });
        for game in 1..=3 {
            let all: Vec<ChampionId> = v
                .team_picks(game)
                .iter()
                .filter_map(Applied::champion)
                .collect();
            let distinct: HashSet<ChampionId> = all.iter().copied().collect();
            assert_eq!(distinct.len(), 5, "repetiu na partida {game}: {all:?}");
            assert!(!distinct.contains(&ChampionId(101)) && !distinct.contains(&ChampionId(102)));
        }
        for p in [P1, P2, P3, P4, P5] {
            let mine: HashSet<ChampionId> = (1..=3)
                .flat_map(|g| v.team_picks(g))
                .filter(|a| a.player == Some(p))
                .filter_map(|a| a.champion())
                .collect();
            assert_eq!(mine.len(), 3, "{p:?} repetiu");
        }
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
    assert_eq!(v.random_team_choice(A, P1, 1, &ctx), Some(ChampionId(112)));
}

#[test]
fn the_roster_changes_but_the_history_replays() {
    let first = |player, champion| Event::TeamPick {
        side: A,
        player,
        game: 1,
        champion: ChampionId(champion),
        gave_way: false,
    };
    let events = [
        Event::Begin { seed: 42, first: A },
        first(P1, 101),
        first(P2, 102),
    ];
    // P1 foi substituído por P5: o histórico refaz sem olhar o elenco, e P5
    // passa a faltar.
    let mut r = Veto::replay(team_rules(0, 2, false), 1, events).unwrap();
    r.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    assert_eq!(missing(&r), [(A, P5, 1), (B, P3, 1), (B, P4, 1)]);
    // Quem saiu da comunidade sai do elenco: B só com P3.
    r.set_rosters(open(&[P1, P2], &[P3])).unwrap();
    assert_eq!(missing(&r), [(B, P3, 1)]);
    let champions = champions(17);
    pick(&mut r, B, P3, 1, 1, &free(&champions)).unwrap();
    assert_eq!(r.turn(), Turn::Done { game: 1 });
    // Um segundo pick do mesmo jogador (de quem voltou e escolheu de novo)
    // refaz, e é o que conta.
    let again = [events[0], events[1], first(P1, 103)];
    let mut twice = Veto::replay(team_rules(0, 2, false), 1, again).unwrap();
    twice.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    assert_eq!(picks_of(&twice, 1, A), ids(&[3]));
}

#[test]
fn undo_takes_back_the_last_pick_then_the_bans() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = two_v_two(team_rules(1, 2, false), 1);
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    pick(&mut v, B, P3, 1, 5, &ctx).unwrap();
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(v.missing().len(), 4);
    v.apply(Event::Undo, None).unwrap();
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
}

#[test]
fn the_map_veto_comes_first_in_teams() {
    // Modo veto, md1 com 3 mapas: dois bans de mapa, um ban de campeão de
    // cada lado e a janela.
    let champions = champions(17);
    let ctx = free(&champions);
    let rules = VetoRules {
        team_size: 2,
        ..rules(MapMode::Veto, 3, true, 1)
    };
    let mut v = two_v_two(rules, 1);
    assert_eq!(acting(&v), (Action::Ban, Target::Map, vec![A]));
    assert_eq!(pick(&mut v, A, P1, 1, 1, &ctx), Err(VetoError::WrongAction));
    step_in(&mut v, A, map(1), &ctx).unwrap();
    step_in(&mut v, B, map(2), &ctx).unwrap();
    assert_eq!(v.map_of(1), Some(MapId(3)));
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![B]));
    step_in(&mut v, B, champ(1), &ctx).unwrap();
    step_in(&mut v, A, champ(2), &ctx).unwrap();
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A, B]));
    for (side, player, c) in [(A, P1, 3), (A, P2, 4), (B, P3, 3), (B, P4, 4)] {
        pick(&mut v, side, player, 1, c, &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    assert_eq!(v.map_steps().len(), 2);
}

#[test]
fn the_vote_of_each_game_comes_back_in_teams() {
    // Votação, md3 com 5 mapas: votam, depois a janela (sem bans); na
    // partida 2, só a votação.
    let champions = champions(17);
    let ctx = free(&champions);
    let rules = VetoRules {
        team_size: 2,
        ..rules(MapMode::Vote, 5, true, 0)
    };
    let mut v = two_v_two(rules, 3);
    assert_eq!(acting(&v), (Action::Vote, Target::Map, vec![A, B]));
    assert!(v.missing().is_empty());
    let offered = v.offered(1);
    step_in(&mut v, B, Choice::Map(offered[0]), &ctx).unwrap();
    step_in(&mut v, A, Choice::Map(offered[0]), &ctx).unwrap();
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A, B]));
    for m in v.missing() {
        let c = m.player.0 * 3 + i64::from(m.game);
        pick(&mut v, m.side, m.player, m.game, c, &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    next_game(&mut v, 2);
    v.set_rosters(sent(&[P1, P2], &[P3, P4], 1, [&[P1, P2], &[P3, P4]]))
        .unwrap();
    assert_eq!(acting(&v), (Action::Vote, Target::Map, vec![A, B]));
    assert!(!v.offered(2).contains(&offered[0]));
    let second = v.offered(2)[0];
    step_in(&mut v, A, Choice::Map(second), &ctx).unwrap();
    step_in(&mut v, B, Choice::Map(second), &ctx).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 2 });
}

#[test]
fn the_three_scenarios_of_the_user() {
    let champions = champions(17);
    let ctx = free(&champions);
    // Pode repetir entre times, mas não no time (o padrão).
    let mut v = two_v_two(team_rules(0, 2, false), 1);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    pick(&mut v, B, P3, 1, 1, &ctx).unwrap();
    assert_eq!(
        pick(&mut v, A, P2, 1, 1, &ctx),
        Err(VetoError::ChampionTaken)
    );
    // Ninguém repete na partida.
    let mut v = two_v_two(no_mirror(team_rules(0, 2, false)), 1);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    assert_eq!(
        pick(&mut v, B, P3, 1, 1, &ctx),
        Err(VetoError::ChampionTaken)
    );
    assert_eq!(
        pick(&mut v, A, P2, 1, 1, &ctx),
        Err(VetoError::ChampionTaken)
    );
    // Todos contra todos com personagens diferentes: cada jogador não
    // repete no torneio (a API passa o que ele jogou nas outras séries).
    let used_by = [(P1, ChampionId(102))];
    let tournament = Context {
        used_by: &used_by,
        ..ctx
    };
    let mut v = two_v_two(ruled_teams(NoRepeatWho::Player, false), 1);
    assert_eq!(
        pick(&mut v, A, P1, 1, 2, &tournament),
        Err(VetoError::ChampionForbidden)
    );
    pick(&mut v, A, P2, 1, 2, &tournament).unwrap();
}
