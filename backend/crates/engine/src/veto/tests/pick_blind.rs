//! Às cegas: a janela em qualquer tamanho, inclusive no duelo, com o que fica
//! escondido até ela fechar, sem desfazer, e só com espelho.

use super::*;
use crate::champions::NoRepeatWho;

fn blind(bans: u8, size: u8) -> VetoRules {
    ordered(team_rules(bans, size, false), PickOrder::Blind)
}

/// Duelo às cegas, mapa combinado fora: A com P1, B com P3.
fn duel(bans: u8, best_of: u8) -> Veto {
    let mut v = begun(blind(bans, 1), best_of, A);
    v.set_rosters(open(&[P1], &[P3])).unwrap();
    v
}

#[test]
fn the_duel_has_the_window_too() {
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = duel(0, 3);
    assert_eq!(acting(&v).0, Action::TeamPick);
    assert_eq!(acting(&v).2, [A, B]);
    assert!(v.turn().is_window());
    // Sem vez: qualquer um, em qualquer partida e ordem, e não muda a vez.
    let window = v.turn_opened();
    pick(&mut v, B, P3, 3, 1, &ctx).unwrap();
    pick(&mut v, A, P1, 2, 1, &ctx).unwrap();
    assert_eq!(v.turn_opened(), window);
    assert_eq!(
        pick(&mut v, B, P3, 3, 2, &ctx),
        Err(VetoError::AlreadyPicked)
    );
    for (side, player, game) in [(A, P1, 1), (A, P1, 3), (B, P3, 1), (B, P3, 2)] {
        pick(&mut v, side, player, game, 2, &ctx).unwrap();
    }
    assert!(matches!(v.turn(), Turn::Done { game: 1 }));
    // Os picks do duelo às cegas ficam por jogador.
    assert_eq!(v.picks(1), [None, None]);
    assert_eq!(v.team_picks(1).len(), 2);
}

#[test]
fn the_duel_keeps_the_rule_of_the_side() {
    let champions = champions(12);
    let ctx = free(&champions);
    let rules = with_rule(blind(0, 1), NoRepeatWho::Player);
    let mut v = begun(rules, 3, A);
    v.set_rosters(open(&[P1], &[P3])).unwrap();
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    assert_eq!(
        pick(&mut v, A, P1, 2, 1, &ctx),
        Err(VetoError::ChampionForbidden)
    );
    // O outro lado pode pegar o mesmo: com espelho, o pick dele não muda nada.
    pick(&mut v, B, P3, 1, 1, &ctx).unwrap();
}

#[test]
fn the_games_are_sealed_only_while_the_window_is_open() {
    let champions = champions(20);
    let ctx = free(&champions);
    // Antes dos picks, nada a esconder.
    let mut v = two_v_two(blind(1, 2), 3);
    assert!(v.sealed_games().is_empty());
    for (side, c) in [(B, 1), (A, 2)] {
        step_in(&mut v, side, champ(c), &ctx).unwrap();
    }
    // Abriu: as três partidas.
    assert_eq!(v.sealed_games(), [1, 2, 3]);
    for (c, m) in (3..).zip(v.missing()) {
        assert_eq!(v.sealed_games(), [1, 2, 3]);
        pick(&mut v, m.side, m.player, m.game, c, &ctx).unwrap();
    }
    // Fechou: nada escondido.
    assert!(v.sealed_games().is_empty());
}

#[test]
fn the_other_orders_seal_nothing() {
    for order in [PickOrder::Alternating, PickOrder::Open] {
        let champions = champions(12);
        let ctx = free(&champions);
        let mut v = two_v_two(ordered(team_rules(0, 2, false), order), 1);
        pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
        assert!(v.sealed_games().is_empty(), "{order:?}");
    }
}

#[test]
fn a_reopened_window_seals_only_the_games_still_open() {
    let champions = champions(20);
    let ctx = free(&champions);
    let mut v = two_v_two(blind(0, 2), 3);
    for (c, m) in (1..).zip(v.missing()) {
        pick(&mut v, m.side, m.player, m.game, c, &ctx).unwrap();
    }
    assert!(v.sealed_games().is_empty());
    // A partida 1 foi enviada, e entra um substituto: reabre para as outras.
    let lineup: [&[PlayerId]; 2] = [&[P1, P2], &[P3, P4]];
    v.set_rosters(sent(&[P1, P2], &[P3, P4], 1, lineup))
        .unwrap();
    next_game(&mut v, 2);
    assert!(v.sealed_games().is_empty());
    v.set_rosters(sent(&[P1, P5], &[P3, P4], 1, lineup))
        .unwrap();
    assert_eq!(v.sealed_games(), [2, 3]);
    // A partida 1 continua à vista, por quem a jogou.
    assert_eq!(v.team_picks(1).len(), 4);
}

#[test]
fn a_finished_series_seals_nothing() {
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = two_v_two(blind(0, 2), 3);
    pick(&mut v, A, P1, 1, 1, &ctx).unwrap();
    v.set_rosters(Rosters {
        finished: true,
        ..open(&[P1, P2], &[P3, P4])
    })
    .unwrap();
    assert!(v.sealed_games().is_empty());
    // E a partida que não foi jogada não conta.
    assert!(v.team_picks(1).is_empty());
}

#[test]
fn what_one_side_picked_changes_nothing_for_the_other() {
    // O que o outro lado pode ver de si mesmo não depende do que foi escolhido
    // do outro lado: nem as opções, nem o motivo de um campeão estar fora.
    let champions = champions(8);
    let ctx = free(&champions);
    let rules = with_rule(blind(0, 2), NoRepeatWho::Team);
    let mut v = two_v_two(rules, 3);
    let picker = Picker {
        side: B,
        player: Some(P3),
    };
    let before = v.pick_options(picker, 1, &ctx);
    let reasons = v.unavailable(picker, 1, &ctx);
    for (player, game, c) in [(P1, 1, 1), (P2, 1, 2), (P1, 2, 3), (P2, 2, 4)] {
        pick(&mut v, A, player, game, c, &ctx).unwrap();
    }
    assert_eq!(v.pick_options(picker, 1, &ctx), before);
    assert_eq!(v.unavailable(picker, 1, &ctx), reasons);
}

#[test]
fn a_blind_pick_is_not_undone_but_what_came_before_is() {
    let champions = champions(12);
    let ctx = free(&champions);
    let mut v = two_v_two(blind(1, 2), 1);
    for (side, c) in [(B, 1), (A, 2)] {
        step_in(&mut v, side, champ(c), &ctx).unwrap();
    }
    // A janela abriu sem ninguém ter escolhido: o último ban ainda se desfaz.
    assert!(v.can_undo());
    let mut before = v.clone();
    before.apply(Event::Undo, None).unwrap();
    assert_eq!(before.bans().len(), 1);
    // Com um pick, não: nem o pick, nem nada antes dele.
    pick(&mut v, A, P1, 1, 3, &ctx).unwrap();
    assert!(!v.can_undo());
    assert_eq!(
        v.clone().apply(Event::Undo, None),
        Err(VetoError::NothingToUndo)
    );
    // Com a janela fechada, também não: ela reabriria já vista.
    for (c, m) in (4..).zip(v.missing()) {
        pick(&mut v, m.side, m.player, m.game, c, &ctx).unwrap();
    }
    assert!(matches!(v.turn(), Turn::Done { .. }));
    assert!(!v.can_undo());
}

#[test]
fn it_only_exists_with_a_mirror() {
    assert_eq!(PickOrder::Blind.check(true), Ok(()));
    assert_eq!(
        PickOrder::Blind.check(false),
        Err(VetoError::BlindNeedsMirror)
    );
    assert_eq!(PickOrder::Alternating.check(false), Ok(()));
    assert_eq!(PickOrder::Open.check(false), Ok(()));
    // O veto não começa assim, e o histórico antigo se refaz sem conferir.
    let champions = champions(12);
    let ctx = free(&champions);
    let rules = no_mirror(blind(0, 2));
    let begin = Event::Begin { seed: 1, first: A };
    assert_eq!(
        Veto::new(rules.clone(), 1).apply(begin, Some(&ctx)),
        Err(VetoError::BlindNeedsMirror)
    );
    assert!(Veto::new(rules, 1).apply(begin, None).is_ok());
}

#[test]
fn the_deadline_draws_everything_missing_at_once() {
    let champions = champions(12);
    let ctx = free(&champions);
    let v = two_v_two(blind(0, 2), 3);
    assert_eq!(v.due_picks(), v.missing());
    assert_eq!(v.due_picks().len(), 12);
    let duel = duel(0, 1);
    assert_eq!(
        duel.due_picks(),
        [
            Missing {
                side: A,
                player: P1,
                game: 1
            },
            Missing {
                side: B,
                player: P3,
                game: 1
            }
        ]
    );
    assert!(duel.random_team_choice(A, P1, 1, &ctx).is_some());
}
