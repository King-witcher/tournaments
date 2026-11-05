//! Duelo: veto de mapas e draft de campeões por lado.

use engine::champions::ChampionRules;
use engine::core::MapId;
use engine::veto::{Event, MapMode, PickOrder, Rosters, VetoOrder, VetoRules};

use super::{
    A, B, ban_champion, ban_map, next_game, pick_champion, pick_map, players, replay, turn_text,
    veto_text, with_rosters,
};
use crate::common::assert_text;

/// Duelo melhor de 3, pool de 7 mapas, veto de mapas com todos os bans
/// primeiro (ban ×4, pick, pick, decisivo), draft com 1 ban por lado e picks
/// alternados (quem abre a partida alterna). Um pick desfeito no meio do
/// draft e os mesmos campeões na partida 3 (sem regra de não repetição).
#[test]
fn duel_best_of_three_with_map_veto_and_champion_draft() {
    let rules = VetoRules {
        map_mode: MapMode::Veto,
        order: VetoOrder::BansFirst,
        pool: (1..=7).map(MapId).collect(),
        champion_phase: true,
        pick_order: PickOrder::Alternating,
        team_size: 1,
        champions: ChampionRules {
            bans_per_series: 1,
            mirror: true,
            same_team: false,
            no_repeat: None,
        },
    };
    let events = [
        Event::Begin { seed: 42, first: A },
        // Veto de mapas: o seed melhor (A) abre e os lados alternam.
        ban_map(A, 3),
        ban_map(B, 6),
        ban_map(A, 1),
        ban_map(B, 5),
        pick_map(A, 2),
        pick_map(B, 7),
        // Bans de campeão: o seed pior (B) abre.
        ban_champion(B, 4),
        ban_champion(A, 9),
        // Partida 1: A abre. B escolhe errado, o organizador desfaz.
        pick_champion(A, 1),
        pick_champion(B, 8),
        Event::Undo,
        pick_champion(B, 2),
        // Partida 2: B abre.
        pick_champion(B, 3),
        pick_champion(A, 5),
        // Partida 3: A abre, e os dois lados repetem o campeão (espelho).
        pick_champion(A, 1),
        pick_champion(B, 1),
        // As partidas 2 e 3 começam quando a anterior é enviada.
        next_game(2, 2),
        next_game(3, 7),
    ];

    let turn_after = |n: usize| turn_text(&replay(&rules, 3, &events[..n]).turn());
    assert_eq!(turn_after(1), "g1 A ban map");
    assert_eq!(turn_after(7), "g1 B ban champion");
    assert_eq!(turn_after(9), "g1 A pick champion for g1");
    assert_eq!(turn_after(10), "g1 B pick champion for g1");
    assert_eq!(turn_after(11), "g1 B pick champion for g2");
    assert_eq!(turn_after(12), "g1 B pick champion for g1");
    assert_eq!(turn_after(17), "done g1");
    assert_eq!(turn_after(18), "done g2");

    let seats = [players(&[1]), players(&[2])];
    let duel = |sent: usize, finished: bool, events: &[Event]| {
        let lineup = [seats[0].clone(), seats[1].clone()];
        with_rosters(
            replay(&rules, 3, events),
            Rosters {
                players: seats.clone(),
                lineups: vec![lineup; sent],
                finished,
            },
        )
    };

    // A série inteira, as três partidas enviadas.
    let stage = duel(3, true, &events);
    assert_text(
        &veto_text(&stage),
        "\
first: A
maps: A ban M3, B ban M6, A ban M1, B ban M5, A pick M2, B pick M7
decider: M4
game 1: M2 picked by A
game 2: M7 picked by B
game 3: M4 decider
bans: B ban C4, A ban C9
picks game 1: A pick C1, B pick C2
picks game 2: B pick C3, A pick C5
picks game 3: A pick C1, B pick C1
missing: -
turn: done g3
turn opened: 19",
    );

    // Série decidida em 2 x 0: a partida 3 não foi jogada, e o pick dela não
    // conta (nem reserva campeão fora da série).
    let sweep = duel(2, true, &events[..18]);
    assert_eq!(sweep.counted_picks(2).len(), 2);
    assert!(sweep.counted_picks(3).is_empty());
    assert_eq!(sweep.played_picks(2).len(), 2);
    assert!(sweep.played_picks(3).is_empty());
}
