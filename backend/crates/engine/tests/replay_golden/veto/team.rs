//! TDM 2v2: veto de mapas, bans e escolha de campeão por jogador.

use engine::champions::{ChampionRules, NoRepeatRule, NoRepeatScope, NoRepeatWho};
use engine::core::MapId;
use engine::veto::{Event, MapMode, PickOrder, VetoOrder, VetoRules};

use super::{
    A, B, ban_champion, ban_map, next_game, pick_map, replay, rosters, team_pick, turn_text,
    veto_text, with_rosters,
};
use crate::common::assert_text;

/// TDM 2v2 melhor de 3, pool de 5 mapas (veto alternado: ban, ban, pick,
/// pick, decisivo), 1 ban de campeão por lado, escolha por jogador em ordem
/// alternada (O, X, X, O; quem abre troca a cada partida) e regra de não
/// repetição por jogador: cada um usa três campeões diferentes. Espelho ligado
/// (os adversários repetem) e sem repetir no time.
fn team_rules() -> VetoRules {
    VetoRules {
        map_mode: MapMode::Veto,
        order: VetoOrder::Alternating,
        pool: (1..=5).map(MapId).collect(),
        champion_phase: true,
        pick_order: PickOrder::Alternating,
        team_size: 2,
        champions: ChampionRules {
            bans_per_series: 1,
            mirror: true,
            same_team: false,
            no_repeat: Some(NoRepeatRule {
                who: NoRepeatWho::Player,
                scope: NoRepeatScope::Series,
            }),
        },
    }
}

/// O histórico do TDM: A tem os jogadores 1 e 2; B, 3 e 4.
fn team_events() -> [Event; 21] {
    [
        Event::Begin { seed: 7, first: A },
        ban_map(A, 2),
        ban_map(B, 4),
        pick_map(A, 1),
        pick_map(B, 5),
        ban_champion(B, 6),
        ban_champion(A, 5),
        // Partida 1: A abre (A, B, B, A).
        team_pick(A, 1, 1, 1),
        team_pick(B, 3, 1, 1),
        team_pick(B, 4, 1, 2),
        team_pick(A, 2, 1, 2),
        // Partida 2: B abre (B, A, A, B).
        team_pick(B, 3, 2, 3),
        team_pick(A, 1, 2, 2),
        team_pick(A, 2, 2, 3),
        team_pick(B, 4, 2, 1),
        // Partida 3: A abre.
        team_pick(A, 1, 3, 3),
        team_pick(B, 3, 3, 2),
        team_pick(B, 4, 3, 3),
        team_pick(A, 2, 3, 4),
        next_game(2, 1),
        next_game(3, 5),
    ]
}

#[test]
fn team_veto_with_per_player_picks_and_no_repeat() {
    let rules = team_rules();
    let events = team_events();

    // O elenco vem da API depois do replay; durante o draft, ninguém foi enviado.
    let draft = |n: usize| {
        let open = rosters(&[1, 2], &[3, 4], 0, [&[], &[]]);
        turn_text(&with_rosters(replay(&rules, 3, &events[..n]), open).turn())
    };
    assert_eq!(draft(5), "g1 B ban champion");
    assert_eq!(draft(7), "g1 A team_pick champion for g1");
    assert_eq!(draft(8), "g1 B team_pick champion for g1");
    assert_eq!(draft(9), "g1 B team_pick champion for g1");
    assert_eq!(draft(10), "g1 A team_pick champion for g1");
    assert_eq!(draft(11), "g1 B team_pick champion for g2");
    assert_eq!(draft(19), "done g1");
    assert_eq!(draft(20), "done g2");

    // Duas partidas enviadas, a terceira em jogo, todos os picks feitos.
    let sent = rosters(&[1, 2], &[3, 4], 2, [&[1, 2], &[3, 4]]);
    let stage = with_rosters(replay(&rules, 3, &events), sent);
    assert_text(
        &veto_text(&stage),
        "\
first: A
maps: A ban M2, B ban M4, A pick M1, B pick M5
decider: M3
game 1: M1 picked by A
game 2: M5 picked by B
game 3: M3 decider
bans: B ban C6, A ban C5
picks game 1: A P1 C1, B P3 C1, B P4 C2, A P2 C2
picks game 2: B P3 C3, A P1 C2, A P2 C3, B P4 C1
picks game 3: A P1 C3, B P3 C2, B P4 C3, A P2 C4
missing: -
turn: done g3
turn opened: 21",
    );
}

/// O mesmo histórico, mas o jogador 2 saiu do time antes da partida 3 e o 5
/// entrou no lugar dele: as partidas enviadas guardam os picks de quem as
/// jogou, e a 3 perde o pick do 2 e passa a esperar o 5.
#[test]
fn team_veto_after_a_substitution_before_the_last_game() {
    let sent = rosters(&[1, 5], &[3, 4], 2, [&[1, 2], &[3, 4]]);
    let stage = with_rosters(replay(&team_rules(), 3, &team_events()), sent);
    assert_text(
        &veto_text(&stage),
        "\
first: A
maps: A ban M2, B ban M4, A pick M1, B pick M5
decider: M3
game 1: M1 picked by A
game 2: M5 picked by B
game 3: M3 decider
bans: B ban C6, A ban C5
picks game 1: A P1 C1, B P3 C1, B P4 C2, A P2 C2
picks game 2: B P3 C3, A P1 C2, A P2 C3, B P4 C1
picks game 3: A P1 C3, B P3 C2, B P4 C3
missing: A P5 g3
turn: g3 A team_pick champion for g3
turn opened: 21",
    );
}
