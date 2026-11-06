//! Disputa de posições: os pares pela ordem da chave e a colocação de cada par.

use engine::formats::StageFormat;
use engine::formats::placement::PlacementConfig;

use crate::common::{
    assert_text, placement, placements_text, played, ready_text, replay, results, snapshot,
    walkover, with_frags,
};

/// 7 participantes, melhor de 3: P7 fica sem par, na posição dele. P4 desiste e
/// perde por W.O.; P2 vence P1 e fica em 1º; P6 vence P5. Os frags não mudam
/// nada (não há tabela).
#[test]
fn placement_with_odd_players_upset_and_walkover() {
    let format = StageFormat::Placement(PlacementConfig { best_of: 3 });
    let reports = [
        walkover(placement(2), 3),
        with_frags(played(placement(3), 6, (2, 0)), 50, 31),
        with_frags(played(placement(1), 2, (2, 1)), 77, 80),
    ];

    // Em andamento: as posições dos pares decididos e a do sem par.
    let midway = replay(format, 7, results(&reports[..2]));
    assert_eq!(ready_text(&midway), "PL1.1");
    assert_text(
        &placements_text(&midway),
        "#3 P3\n#4 P4\n#5 P6\n#6 P5\n#7 P7",
    );

    let stage = replay(format, 7, results(&reports));
    assert_eq!(stage.table(), None);
    assert_eq!(stage.total_rounds(), None);
    assert_text(
        &snapshot(&stage),
        "\
PL1.1 bo3 P1 v P2 -> P2 2-1
PL1.2 bo3 P3 v P4 -> P3 wo
PL1.3 bo3 P5 v P6 -> P6 2-0
PL1.4 bo3 bye P7
finished: yes
#1 P2
#2 P1
#3 P3
#4 P4
#5 P6
#6 P5
#7 P7",
    );
}
