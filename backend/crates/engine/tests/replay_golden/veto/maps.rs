//! Votação e sorteio do mapa: o que sai da seed gravada no começo da série.

use engine::champions::ChampionRules;
use engine::core::MapId;
use engine::veto::{Event, MapMode, PickOrder, VetoOrder, VetoRules};

use super::{A, B, draws_text, next_game, replay, vote_map};
use crate::common::assert_text;

fn rules(map_mode: MapMode) -> VetoRules {
    VetoRules {
        map_mode,
        order: VetoOrder::BansFirst,
        pool: (1..=7).map(MapId).collect(),
        champion_phase: false,
        pick_order: PickOrder::Alternating,
        team_size: 1,
        champions: ChampionRules {
            bans_per_series: 0,
            mirror: true,
            same_team: false,
            no_repeat: None,
        },
    }
}

/// Votação, seed 2026: os três mapas oferecidos de cada partida saem da seed
/// (o da partida 1, `[M1, M3, M6]`, é o mesmo que o teste unitário de
/// `draw.rs` fixa), sem os já jogados na série. Votos iguais valem; votos
/// diferentes são decididos por sorteio da seed (partida 2).
///
/// Os mapas oferecidos das partidas 2 e 3 e o desempate foram congelados
/// executando o engine: o ChaCha8 não se confere à mão.
#[test]
fn vote_offers_and_tie_break_come_from_the_recorded_seed() {
    let events = [
        Event::Begin {
            seed: 2026,
            first: A,
        },
        vote_map(A, 3),
        vote_map(B, 3),
        next_game(2, 3),
        vote_map(A, 1),
        vote_map(B, 5),
        next_game(3, 5),
        vote_map(B, 4),
        vote_map(A, 4),
    ];
    let stage = replay(&rules(MapMode::Vote), 3, &events);
    assert_text(
        &draws_text(&stage),
        "\
game 1: offered [M1 M3 M6], votes M3/M3, map M3
game 2: offered [M1 M2 M5], votes M1/M5, map M5
game 3: offered [M2 M4 M7], votes M4/M4, map M4",
    );
}

/// Sorteio do mapa, seed 2026: sai da seed e não repete o mapa em que a
/// partida anterior foi enviada. Congelado executando o engine.
#[test]
fn random_map_comes_from_the_recorded_seed() {
    let events = [
        Event::Begin {
            seed: 2026,
            first: A,
        },
        next_game(2, 5),
        next_game(3, 3),
    ];
    let stage = replay(&rules(MapMode::Random), 3, &events);
    assert_text(
        &draws_text(&stage),
        "\
game 1: offered [], votes -/-, map M5
game 2: offered [], votes -/-, map M3
game 3: offered [], votes -/-, map M2",
    );
}
