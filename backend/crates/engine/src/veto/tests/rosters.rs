//! O elenco da escolha simultânea (`Veto::set_rosters`): sem ele, a vez é
//! explícita; com o jogo enviado ou a série encerrada, nada reabre; o
//! elenco é conferido.

use super::*;

/// 2v2 depois do ban de cada lado, sem o elenco informado.
fn after_bans(ctx: &Context) -> Veto {
    let mut v = begun(team_rules(1, 2, false), 3, A);
    step_in(&mut v, A, champ(1), ctx).unwrap();
    step_in(&mut v, B, champ(2), ctx).unwrap();
    v
}

#[test]
fn without_the_roster_the_turn_says_so() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = after_bans(&ctx);
    // Nunca um jogo acabado: o elenco não foi informado.
    assert_eq!(v.turn(), Turn::RostersUnknown { game: 1 });
    assert!(v.missing(1).is_empty());
    assert_eq!(v.team_options(A, P1, &ctx), TeamOptions::default());
    assert_eq!(v.random_team_choice(A, P1, &ctx), None);
    // Ao vivo, a escolha é recusada; um passo de lado também.
    assert_eq!(pick(&mut v, A, P1, 3, &ctx), Err(VetoError::RostersUnknown));
    assert_eq!(
        step_in(&mut v, A, champ(3), &ctx),
        Err(VetoError::WrongAction)
    );
    // No replay (sem `ctx`), a escolha gravada entra.
    v.apply(
        Event::TeamPick {
            side: A,
            player: P1,
            champion: ChampionId(103),
        },
        None,
    )
    .unwrap();
    assert_eq!(v.turn(), Turn::RostersUnknown { game: 1 });
    // Antes dos bans, a vez não depende do elenco.
    let w = begun(team_rules(1, 2, false), 3, A);
    assert_eq!(acting(&w), (Action::Ban, Target::Champion, vec![A]));
}

#[test]
fn an_empty_side_waits_for_the_other_and_two_close_the_game() {
    let champions = champions(17);
    let ctx = free(&champions);
    // B sem ninguém na comunidade: só A falta.
    let mut v = after_bans(&ctx);
    v.set_rosters(open(&[P1, P2], &[])).unwrap();
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A]));
    assert_eq!(v.missing(1), [(A, P1), (A, P2)]);
    assert_eq!(pick(&mut v, B, P3, 3, &ctx), Err(VetoError::NotYourTurn));
    pick(&mut v, A, P1, 3, &ctx).unwrap();
    pick(&mut v, A, P2, 4, &ctx).unwrap();
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    // Os dois vazios, informados: ninguém a esperar.
    let mut w = after_bans(&ctx);
    w.set_rosters(open(&[], &[])).unwrap();
    assert_eq!(w.turn(), Turn::Done { game: 1 });
    assert!(w.missing(1).is_empty());
}

#[test]
fn a_sent_game_does_not_reopen_for_who_joined() {
    let champions = champions(17);
    let ctx = free(&champions);
    let mut v = after_bans(&ctx);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    for (side, player, c) in [(A, P1, 3), (A, P2, 4), (B, P3, 3), (B, P4, 4)] {
        pick(&mut v, side, player, c, &ctx).unwrap();
    }
    assert_eq!(v.turn(), Turn::Done { game: 1 });

    // Antes do envio, quem entra no time passa a faltar.
    let mut open_game = v.clone();
    open_game.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    assert_eq!(open_game.missing(1), [(A, P5)]);
    pick(&mut open_game, A, P5, 5, &ctx).unwrap();
    assert_eq!(open_game.turn(), Turn::Done { game: 1 });

    // Enviado o jogo 1: continua acabado, e quem entrou não escolhe nele.
    let mut sent = v.clone();
    sent.set_rosters(Rosters {
        sent: 1,
        ..open(&[P5, P2], &[P3, P4])
    })
    .unwrap();
    assert_eq!(sent.turn(), Turn::Done { game: 1 });
    assert!(sent.missing(1).is_empty());
    assert!(sent.team_options(A, P5, &ctx).champions.is_empty());
    assert_eq!(sent.random_team_choice(A, P5, &ctx), None);
    assert_eq!(pick(&mut sent, A, P5, 5, &ctx), Err(VetoError::Finished));

    // A série encerrada (W.O. no meio da escolha): também não reabre.
    let mut finished = after_bans(&ctx);
    finished
        .set_rosters(Rosters {
            finished: true,
            ..open(&[P1, P2], &[P3, P4])
        })
        .unwrap();
    assert_eq!(finished.turn(), Turn::Done { game: 1 });
    assert_eq!(
        pick(&mut finished, A, P1, 3, &ctx),
        Err(VetoError::Finished)
    );

    // No jogo seguinte, o enviado é o anterior: a escolha abre normalmente.
    next_game(&mut sent, 2);
    step_in(&mut sent, A, champ(1), &ctx).unwrap();
    step_in(&mut sent, B, champ(2), &ctx).unwrap();
    assert_eq!(sent.missing(2), [(A, P5), (A, P2), (B, P3), (B, P4)]);
}

#[test]
fn the_roster_is_checked() {
    let mut v = begun(team_rules(0, 2, false), 3, A);
    assert_eq!(
        v.set_rosters(open(&[P1, P1], &[P3])),
        Err(VetoError::RosterRepeated(P1))
    );
    assert_eq!(
        v.set_rosters(open(&[P1, P2], &[P3, P2])),
        Err(VetoError::RosterRepeated(P2))
    );
    assert_eq!(
        v.set_rosters(open(&[P1], &[P3, P4, P5])),
        Err(VetoError::RosterTooLarge { max: 2, got: 3 })
    );
    // Recusado, o elenco anterior fica.
    assert_eq!(v.turn(), Turn::RostersUnknown { game: 1 });
    v.set_rosters(open(&[P1], &[P3, P4])).unwrap();
    assert_eq!(v.missing(1), [(A, P1), (B, P3), (B, P4)]);
}
