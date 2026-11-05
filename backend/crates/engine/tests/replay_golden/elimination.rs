//! Eliminação simples e dupla, do primeiro resultado ao campeão.

use engine::formats::elimination::{EliminationConfig, EliminationKind};
use engine::formats::{StageFormat, StageHistory};
use engine::series::SeriesLength;

use crate::common::{
    assert_text, grand, lower, placements_text, played, ready_text, replay, results, snapshot,
    upper, walkover,
};

fn format(kind: EliminationKind, default: u8, finals: u8) -> StageFormat {
    StageFormat::Elimination(EliminationConfig {
        kind,
        series: SeriesLength { default, finals },
    })
}

/// 5 participantes numa chave de 8: os seeds 1, 2 e 3 entram com bye, e só o 4
/// contra o 5 joga a primeira rodada (o 2 contra o 3 já nasce pronto na
/// segunda). Melhor de 3, final melhor de 5; um W.O.
/// nas semifinais.
#[test]
fn single_elimination_with_byes_up_to_the_champion() {
    let format = format(EliminationKind::Single, 3, 5);
    let reports = [
        played(upper(1, 2), 5, (2, 1)),
        played(upper(2, 1), 1, (2, 0)),
        walkover(upper(2, 2), 3),
        played(upper(3, 1), 3, (3, 2)),
    ];

    // Torneio recém-iniciado: o 4 contra o 5 e, entre os que avançaram sem
    // jogar, o 2 contra o 3.
    let start = replay(format, 5, StageHistory::default());
    assert_eq!(ready_text(&start), "W1.2, W2.2");

    // Em andamento, depois de dois resultados.
    let midway = replay(format, 5, results(&reports[..2]));
    assert_eq!(ready_text(&midway), "W2.2");
    assert!(!midway.is_finished());
    assert_text(&placements_text(&midway), "#3-4 P5\n#5 P4");

    let stage = replay(format, 5, results(&reports));
    assert_text(
        &snapshot(&stage),
        "\
W1.1 bo3 bye P1
W1.2 bo3 P4 v P5 -> P5 2-1
W1.3 bo3 bye P2
W1.4 bo3 bye P3
W2.1 bo3 P1 v P5 -> P1 2-0
W2.2 bo3 P2 v P3 -> P3 wo
W3.1 bo5 P1 v P3 -> P3 3-2
finished: yes
#1 P3
#2 P1
#3-4 P5
#3-4 P2
#5 P4",
    );
}

/// 6 participantes numa chave de 8 (os seeds 1 e 2 com bye), melhor de 3 e
/// grande final melhor de 5. O campeão da chave de baixo (P3) vence a grande
/// final, o reset é jogado e ele vence de novo. Um W.O. na chave de baixo.
#[test]
fn double_elimination_with_reset_played_by_the_lower_champion() {
    let format = format(
        EliminationKind::Double {
            grand_final_reset: true,
        },
        3,
        5,
    );
    let reports = [
        played(upper(1, 2), 4, (2, 0)),
        played(upper(1, 4), 3, (2, 1)),
        played(upper(2, 1), 1, (2, 0)),
        played(upper(2, 2), 3, (2, 1)),
        played(lower(2, 1), 2, (2, 0)),
        walkover(lower(2, 2), 4),
        played(upper(3, 1), 1, (2, 1)),
        played(lower(3, 1), 4, (2, 1)),
        played(lower(4, 1), 3, (2, 1)),
        played(grand(1), 3, (3, 1)),
        played(grand(2), 3, (3, 0)),
    ];

    // Depois das duas primeiras rodadas de cima: a final de cima e a segunda
    // rodada de baixo estão prontas; os byes da primeira de baixo já avançaram.
    let midway = replay(format, 6, results(&reports[..4]));
    assert_eq!(ready_text(&midway), "W3.1, L2.1, L2.2");
    assert!(!midway.is_finished());

    // A grande final decidida sem o reset ainda não encerra a chave.
    let first_final = replay(format, 6, results(&reports[..10]));
    assert_eq!(ready_text(&first_final), "G2.1");
    assert!(!first_final.is_finished());

    let stage = replay(format, 6, results(&reports));
    assert_text(
        &snapshot(&stage),
        "\
W1.1 bo3 bye P1
W1.2 bo3 P4 v P5 -> P4 2-0
W1.3 bo3 bye P2
W1.4 bo3 P3 v P6 -> P3 2-1
W2.1 bo3 P1 v P4 -> P1 2-0
W2.2 bo3 P2 v P3 -> P3 2-1
W3.1 bo3 P1 v P3 -> P1 2-1
L1.1 bo3 bye P5
L1.2 bo3 bye P6
L2.1 bo3 P5 v P2 -> P2 2-0
L2.2 bo3 P6 v P4 -> P4 wo
L3.1 bo3 P2 v P4 -> P4 2-1
L4.1 bo3 P4 v P3 -> P3 2-1
G1.1 bo5 P1 v P3 -> P3 3-1
G2.1 bo5 P1 v P3 -> P3 3-0
finished: yes
#1 P3
#2 P1
#3 P4
#4 P2
#5-6 P5
#5-6 P6",
    );
}

/// Histórico de 4 participantes que termina na grande final, comum aos dois
/// testes abaixo (melhor de 1, grande final melhor de 3): o campeão da chave
/// de cima (P1) vence a primeira final.
fn four_players_upper_champion_wins_the_final() -> [engine::core::StageReport; 6] {
    [
        played(upper(1, 1), 1, (1, 0)),
        played(upper(1, 2), 3, (1, 0)),
        played(upper(2, 1), 1, (1, 0)),
        played(lower(1, 1), 2, (1, 0)),
        played(lower(2, 1), 3, (1, 0)),
        played(grand(1), 1, (2, 1)),
    ]
}

#[test]
fn double_elimination_without_reset_ends_at_the_grand_final() {
    let format = format(
        EliminationKind::Double {
            grand_final_reset: false,
        },
        1,
        3,
    );
    let reports = four_players_upper_champion_wins_the_final();

    let before_final = replay(format, 4, results(&reports[..5]));
    assert_eq!(ready_text(&before_final), "G1.1");
    assert!(!before_final.is_finished());

    let stage = replay(format, 4, results(&reports));
    assert_text(
        &snapshot(&stage),
        "\
W1.1 bo1 P1 v P4 -> P1 1-0
W1.2 bo1 P2 v P3 -> P3 1-0
W2.1 bo1 P1 v P3 -> P1 1-0
L1.1 bo1 P4 v P2 -> P2 1-0
L2.1 bo1 P2 v P3 -> P3 1-0
G1.1 bo3 P1 v P3 -> P1 2-1
finished: yes
#1 P1
#2 P3
#3 P2
#4 P4",
    );
}

/// Com reset, o mesmo histórico encerra a chave: o campeão de cima venceu a
/// primeira final e a segunda fica "não precisou".
#[test]
fn double_elimination_with_reset_not_needed_when_the_upper_champion_wins() {
    let format = format(
        EliminationKind::Double {
            grand_final_reset: true,
        },
        1,
        3,
    );
    let reports = four_players_upper_champion_wins_the_final();

    let stage = replay(format, 4, results(&reports));
    assert_text(
        &snapshot(&stage),
        "\
W1.1 bo1 P1 v P4 -> P1 1-0
W1.2 bo1 P2 v P3 -> P3 1-0
W2.1 bo1 P1 v P3 -> P1 1-0
L1.1 bo1 P4 v P2 -> P2 1-0
L2.1 bo1 P2 v P3 -> P3 1-0
G1.1 bo3 P1 v P3 -> P1 2-1
G2.1 bo3 not needed
finished: yes
#1 P1
#2 P3
#3 P2
#4 P4",
    );
}
