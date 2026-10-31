//! O elenco da janela simultânea (`Veto::set_rosters`): sem ele, a vez é
//! explícita; com a partida enviada ou a série encerrada, nada reabre; quem
//! entra no time escolhe para as partidas ainda abertas; o elenco é
//! conferido.

use super::*;

/// 2v2, md3, depois do ban de cada lado (B, o seed pior, primeiro), sem o
/// elenco informado.
fn after_bans(ctx: &Context) -> Veto {
    let mut v = begun(team_rules(1, 2, false), 3, A);
    step_in(&mut v, B, champ(1), ctx).unwrap();
    step_in(&mut v, A, champ(2), ctx).unwrap();
    v
}

/// Todos do elenco de agora escolhem para as partidas abertas, um campeão
/// diferente por jogador (a partir de `from`).
fn fill(v: &mut Veto, from: i64, ctx: &Context) {
    for m in v.missing() {
        let c = from + m.player.0 * 3 + i64::from(m.game);
        pick(v, m.side, m.player, m.game, c, ctx).unwrap();
    }
}

#[test]
fn without_the_roster_the_turn_says_so() {
    let champions = champions(30);
    let ctx = free(&champions);
    let mut v = after_bans(&ctx);
    // Nunca uma partida acabada: o elenco não foi informado.
    assert_eq!(v.turn(), Turn::RostersUnknown { game: 1 });
    assert!(v.missing().is_empty());
    assert!(team_options(&v, A, P1, 1, &ctx).is_empty());
    assert_eq!(v.random_team_choice(A, P1, 1, &ctx), None);
    // Ao vivo, a escolha é recusada; um passo de lado também.
    assert_eq!(
        pick(&mut v, A, P1, 1, 3, &ctx),
        Err(VetoError::RostersUnknown)
    );
    assert_eq!(
        step_in(&mut v, A, champ(3), &ctx),
        Err(VetoError::WrongAction)
    );
    // No replay (sem `ctx`), a escolha gravada entra.
    v.apply(
        Event::TeamPick {
            side: A,
            player: P1,
            game: 2,
            champion: ChampionId(103),
            gave_way: false,
        },
        None,
    )
    .unwrap();
    assert_eq!(v.turn(), Turn::RostersUnknown { game: 1 });
    // Antes dos bans, a vez não depende do elenco.
    let w = begun(team_rules(1, 2, false), 3, A);
    assert_eq!(acting(&w), (Action::Ban, Target::Champion, vec![B]));
}

#[test]
fn an_empty_side_waits_for_the_other_and_two_close_the_draft() {
    let champions = champions(30);
    let ctx = free(&champions);
    // B sem ninguém na comunidade: só A falta.
    let mut v = after_bans(&ctx);
    v.set_rosters(open(&[P1, P2], &[])).unwrap();
    assert_eq!(acting(&v), (Action::TeamPick, Target::Champion, vec![A]));
    assert_eq!(missing(&v).len(), 6);
    assert_eq!(pick(&mut v, B, P3, 1, 3, &ctx), Err(VetoError::NotYourTurn));
    fill(&mut v, 3, &ctx);
    assert_eq!(v.turn(), Turn::Done { game: 1 });
    // Os dois vazios, informados: ninguém a esperar.
    let mut w = after_bans(&ctx);
    w.set_rosters(open(&[], &[])).unwrap();
    assert_eq!(w.turn(), Turn::Done { game: 1 });
    assert!(w.missing().is_empty());
}

#[test]
fn a_sent_game_does_not_reopen_for_who_joined() {
    let champions = champions(30);
    let ctx = free(&champions);
    let mut v = after_bans(&ctx);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    fill(&mut v, 3, &ctx);
    assert_eq!(v.turn(), Turn::Done { game: 1 });

    // Antes do envio, quem entra no time passa a faltar em todas.
    let mut open_game = v.clone();
    open_game.set_rosters(open(&[P5, P2], &[P3, P4])).unwrap();
    assert_eq!(missing(&open_game), [(A, P5, 1), (A, P5, 2), (A, P5, 3)]);

    // Enviada a partida 1: ela não reabre para quem entrou (e a 2 só começa
    // com as dele sorteadas: `DraftPending`).
    let lineup: [&[PlayerId]; 2] = [&[P1, P2], &[P3, P4]];
    let mut joined = v.clone();
    joined
        .set_rosters(sent(&[P5, P2], &[P3, P4], 1, lineup))
        .unwrap();
    assert_eq!(missing(&joined), [(A, P5, 2), (A, P5, 3)]);
    assert!(team_options(&joined, A, P5, 1, &ctx).is_empty());
    assert_eq!(joined.random_team_choice(A, P5, 1, &ctx), None);
    assert_eq!(
        pick(&mut joined, A, P5, 1, 20, &ctx),
        Err(VetoError::Finished)
    );
    assert_eq!(
        joined.apply(
            Event::Game {
                game: 2,
                previous: None
            },
            None
        ),
        Err(VetoError::DraftPending)
    );

    // Entrou depois de a partida 2 começar: a janela reabre para ele, só
    // nas que faltam.
    let mut later = v.clone();
    later
        .set_rosters(sent(&[P1, P2], &[P3, P4], 1, lineup))
        .unwrap();
    next_game(&mut later, 2);
    later
        .set_rosters(sent(&[P5, P2], &[P3, P4], 1, lineup))
        .unwrap();
    assert_eq!(
        acting(&later),
        (Action::TeamPick, Target::Champion, vec![A])
    );
    pick(&mut later, A, P5, 3, 20, &ctx).unwrap();
    pick(&mut later, A, P5, 2, 21, &ctx).unwrap();
    assert_eq!(later.turn(), Turn::Done { game: 2 });

    // A série encerrada (W.O. no meio da janela): também não reabre.
    let mut finished = after_bans(&ctx);
    finished
        .set_rosters(Rosters {
            finished: true,
            ..open(&[P1, P2], &[P3, P4])
        })
        .unwrap();
    assert_eq!(finished.turn(), Turn::Done { game: 1 });
    assert_eq!(
        pick(&mut finished, A, P1, 1, 3, &ctx),
        Err(VetoError::Finished)
    );
}

#[test]
fn a_pick_names_a_game_of_the_series() {
    let champions = champions(30);
    let ctx = free(&champions);
    let mut v = after_bans(&ctx);
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    assert_eq!(
        pick(&mut v, A, P1, 0, 3, &ctx),
        Err(VetoError::OptionInvalid)
    );
    assert_eq!(
        pick(&mut v, A, P1, 4, 3, &ctx),
        Err(VetoError::OptionInvalid)
    );
    pick(&mut v, A, P1, 3, 3, &ctx).unwrap();
    assert_eq!(
        pick(&mut v, A, P1, 3, 4, &ctx),
        Err(VetoError::AlreadyPicked)
    );
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
    v.set_rosters(open(&[P1], &[P3])).unwrap();
    assert_eq!(
        missing(&v),
        [
            (A, P1, 1),
            (A, P1, 2),
            (A, P1, 3),
            (B, P3, 1),
            (B, P3, 2),
            (B, P3, 3)
        ]
    );
}
