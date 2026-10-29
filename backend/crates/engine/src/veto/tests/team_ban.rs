//! A proteção do ban nos times: quem bane precisa que sobre campeão para
//! cada jogador do elenco, contado jogador a jogador.

use super::*;
use crate::champions::NoRepeatWho;

/// Os eventos de um jogo já feito: o ban de cada lado (`[a, b]`, A
/// começa) e as escolhas.
fn played(events: &mut Vec<Event>, game: u8, bans: [i64; 2], picks: &[(Side, PlayerId, i64)]) {
    if game > 1 {
        events.push(Event::Game {
            game,
            starter: A,
            previous: None,
        });
    }
    for (side, c) in [(A, bans[0]), (B, bans[1])] {
        events.push(Event::Step {
            side,
            action: Action::Ban,
            choice: champ(c),
        });
    }
    for (side, player, c) in picks {
        events.push(Event::TeamPick {
            side: *side,
            player: *player,
            champion: ChampionId(100 + c),
        });
    }
}

/// 2v2 md5, 1 ban, no jogo 3 (A bane primeiro), depois de dois jogos em
/// que P1 escolheu `p1` e P2 escolheu `p2`.
fn third_game(p1: [i64; 2], p2: [i64; 2], bans: [[i64; 2]; 2]) -> Veto {
    let mut events = vec![Event::Begin { seed: 42, first: A }];
    for g in 0..2 {
        played(
            &mut events,
            g as u8 + 1,
            bans[g],
            &[
                (A, P1, p1[g]),
                (A, P2, p2[g]),
                (B, P3, p1[g]),
                (B, P4, p2[g]),
            ],
        );
    }
    events.push(Event::Game {
        game: 3,
        starter: A,
        previous: None,
    });
    let mut v = Veto::replay(team_rules(1, 2, false), 5, events).unwrap();
    v.set_rosters(open(&[P1, P2], &[P3, P4])).unwrap();
    v
}

fn player_rule(champions: &[ChampionId]) -> Context<'_> {
    Context {
        champions,
        no_repeat: Some(NoRepeatWho::Player),
        used: [&[], &[]],
        used_by: &[],
    }
}

#[test]
fn each_player_counts_only_what_he_used() {
    // Pool de 6. P1 usou o 1 e o 2; P2, o 3 e o 1. Cada um tem 4 livres, o
    // que basta: o ban de A (1), o de B (1) e os dois de A no mapa. Contar
    // o que o elenco todo usou (1, 2 e 3) deixaria só 3 e prenderia o ban.
    let v = third_game([1, 2], [3, 1], [[5, 6], [5, 6]]);
    let champions = champions(6);
    let ctx = player_rule(&champions);
    assert_eq!(acting(&v), (Action::Ban, Target::Champion, vec![A]));
    assert_eq!(v.options(A, &ctx), [1, 2, 3, 4, 5, 6].map(champ).to_vec());
}

#[test]
fn the_ban_spares_what_a_short_player_can_pick() {
    // Pool de 5, as mesmas escolhas: P1 tem 3, 4 e 5; P2, 2, 4 e 5. Os dois
    // ficariam sem campeão: A só bane o 1, que nenhum dos dois pode escolher
    // (o 2 é de P2; o 3, de P1).
    let v = third_game([1, 2], [3, 1], [[5, 4], [5, 4]]);
    let champions = champions(5);
    let ctx = player_rule(&champions);
    assert_eq!(v.options(A, &ctx), [champ(1)]);
}

#[test]
fn with_nothing_safe_the_protection_gives_way() {
    // Pool de 5: P1 usou o 1 e o 2; P2, o 3 e o 4. Cada um tem 3 (o 5 e os
    // dois do outro), e todo campeão serve a algum dos dois: a proteção
    // cede, como a regra, para o veto não travar.
    let v = third_game([1, 2], [3, 4], [[5, 4], [5, 3]]);
    let champions = champions(5);
    let ctx = player_rule(&champions);
    assert_eq!(v.options(A, &ctx).len(), 5);
}

#[test]
fn the_team_rule_counts_the_whole_team() {
    // Pool de 6, 1 ban de cada lado, 2v2 pela regra do time: A já usou 4
    // fora da série e tem 101 e 102 para os dois. B ainda bane uma vez: se A
    // banisse um dos dois, B tiraria o outro.
    let champions = champions(6);
    let used = [103, 104, 105, 106].map(ChampionId);
    let ctx = Context {
        champions: &champions,
        no_repeat: Some(NoRepeatWho::Team),
        used: [&used, &[]],
        used_by: &[],
    };
    let mut v = two_v_two(1, false);
    assert_eq!(v.options(A, &ctx), [champ(3), champ(4), champ(5), champ(6)]);
    step_in(&mut v, A, champ(3), &ctx).unwrap();
}
