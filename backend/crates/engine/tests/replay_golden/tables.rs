//! Round robin e suíço: tabela, desempates e colocação final.

use engine::formats::round_robin::RoundRobinConfig;
use engine::formats::swiss::{SwissConfig, SwissRound};
use engine::formats::{StageFormat, StageHistory};

use crate::common::{
    assert_text, matches_text, p, placements_text, played, replay, results, robin, swiss,
    table_text, walkover, with_frags,
};

/// 5 participantes, melhor de 3: cada um folga uma vez (a folga não vale
/// ponto). P5 desiste depois da rodada 3 e perde por W.O. a única série que
/// falta. Três empatam com 2 pontos (P1, P3 e P5), e cada um venceu um dos
/// outros dois (ciclo), então o confronto direto não separa; o saldo de mapas
/// é 0 para os três; o saldo de frags separa P5 de P1 e P3, que seguem
/// empatados (+5) e voltam ao confronto direto, que P1 venceu.
#[test]
fn round_robin_with_odd_players_ties_and_walkover() {
    let format = StageFormat::RoundRobin(RoundRobinConfig { best_of: 3 });
    let reports = [
        with_frags(played(robin(1, 1), 2, (2, 1)), 100, 90),
        with_frags(played(robin(1, 2), 4, (2, 1)), 95, 98),
        with_frags(played(robin(2, 1), 1, (2, 0)), 60, 40),
        with_frags(played(robin(2, 2), 5, (2, 0)), 55, 45),
        with_frags(played(robin(3, 1), 5, (2, 1)), 80, 85),
        with_frags(played(robin(3, 2), 2, (2, 0)), 70, 30),
        with_frags(played(robin(4, 1), 2, (2, 0)), 66, 44),
        walkover(robin(4, 2), 3),
        with_frags(played(robin(5, 1), 1, (2, 1)), 90, 88),
        with_frags(played(robin(5, 2), 3, (2, 1)), 102, 80),
    ];

    // Em andamento: a tabela existe, mas ninguém tem colocação ainda.
    let midway = replay(format, 5, results(&reports[..5]));
    assert!(!midway.is_finished());
    assert_eq!(placements_text(&midway), "");
    assert_text(
        &table_text(&midway),
        "\
1 P5 pts2 2-1 byes0 maps 5-3 frags 225-230 bh - ahead points
2 P1 pts1 1-1 byes0 maps 3-2 frags 145-120 bh - ahead frag_diff
3 P2 pts1 1-0 byes0 maps 2-1 frags 100-90 bh - ahead map_diff
4 P4 pts1 1-1 byes0 maps 2-3 frags 140-153 bh - ahead points
5 P3 pts0 0-2 byes0 maps 1-4 frags 138-155 bh - ahead -",
    );

    let stage = replay(format, 5, results(&reports));
    assert_text(
        &matches_text(&stage),
        "\
RR1.1 bo3 P2 v P5 -> P2 2-1
RR1.2 bo3 P3 v P4 -> P4 2-1
RR1.3 bo3 bye P1
RR2.1 bo3 P1 v P3 -> P1 2-0
RR2.2 bo3 P4 v P5 -> P5 2-0
RR2.3 bo3 bye P2
RR3.1 bo3 P1 v P5 -> P5 2-1
RR3.2 bo3 P2 v P4 -> P2 2-0
RR3.3 bo3 bye P3
RR4.1 bo3 P1 v P2 -> P2 2-0
RR4.2 bo3 P3 v P5 -> P3 wo
RR4.3 bo3 bye P4
RR5.1 bo3 P1 v P4 -> P1 2-1
RR5.2 bo3 P2 v P3 -> P3 2-1
RR5.3 bo3 bye P5",
    );
    assert!(stage.is_finished());
    assert_eq!(stage.total_rounds(), Some(5));
    assert_text(
        &table_text(&stage),
        "\
1 P2 pts3 3-1 byes0 maps 7-3 frags 316-266 bh - ahead points
2 P1 pts2 2-2 byes0 maps 5-5 frags 279-274 bh - ahead head_to_head
3 P3 pts2 2-2 byes0 maps 5-5 frags 240-235 bh - ahead frag_diff
4 P5 pts2 2-2 byes0 maps 5-5 frags 225-230 bh - ahead points
5 P4 pts1 1-3 byes0 maps 3-7 frags 258-313 bh - ahead -",
    );
    assert_text(
        &placements_text(&stage),
        "#1 P2\n#2 P1\n#3 P3\n#4 P5\n#5 P4",
    );
}

/// Round robin: o confronto direto vem antes do saldo de mapas. P2 e P1 empatam
/// com 2 pontos; P1 tem o saldo melhor (+3 contra +1), mas P2 venceu o confronto.
#[test]
fn round_robin_head_to_head_comes_before_the_map_difference() {
    let format = StageFormat::RoundRobin(RoundRobinConfig { best_of: 3 });
    let reports = [
        played(robin(1, 1), 1, (2, 0)),
        played(robin(1, 2), 3, (2, 1)),
        played(robin(2, 1), 1, (2, 0)),
        played(robin(2, 2), 2, (2, 1)),
        played(robin(3, 1), 2, (2, 1)),
        played(robin(3, 2), 4, (2, 1)),
    ];
    let stage = replay(format, 4, results(&reports));
    assert!(stage.is_finished());
    assert_text(
        &table_text(&stage),
        "\
1 P2 pts2 2-1 byes0 maps 5-4 frags 0-0 bh - ahead head_to_head
2 P1 pts2 2-1 byes0 maps 5-2 frags 0-0 bh - ahead points
3 P4 pts1 1-2 byes0 maps 3-5 frags 0-0 bh - ahead head_to_head
4 P3 pts1 1-2 byes0 maps 3-5 frags 0-0 bh - ahead -",
    );
}

/// Suíço: o Buchholz vem antes do confronto direto. P1 e P6 empatam com 2
/// pontos; P6 venceu o confronto, mas P1 tem o Buchholz maior (10 contra 9).
#[test]
fn swiss_buchholz_comes_before_head_to_head() {
    let format = StageFormat::Swiss(SwissConfig {
        best_of: 3,
        rounds: Some(4),
    });
    let rounds = [
        round(&[[1, 4], [2, 5], [3, 6]], None),
        round(&[[1, 6], [2, 4], [3, 5]], None),
        round(&[[1, 2], [3, 4], [5, 6]], None),
        round(&[[1, 3], [2, 6], [4, 5]], None),
    ];
    let reports = [
        played(swiss(1, 1), 1, (2, 0)),
        played(swiss(1, 2), 2, (2, 0)),
        played(swiss(1, 3), 3, (2, 0)),
        played(swiss(2, 1), 6, (2, 0)),
        played(swiss(2, 2), 2, (2, 0)),
        played(swiss(2, 3), 3, (2, 0)),
        played(swiss(3, 1), 2, (2, 0)),
        played(swiss(3, 2), 3, (2, 0)),
        played(swiss(3, 3), 6, (2, 0)),
        played(swiss(4, 1), 1, (2, 0)),
        played(swiss(4, 2), 2, (2, 0)),
        played(swiss(4, 3), 4, (2, 0)),
    ];
    let history = StageHistory {
        rounds: rounds.to_vec(),
        ..results(&reports)
    };
    let stage = replay(format, 6, history);
    assert!(stage.is_finished());
    assert_text(
        &table_text(&stage),
        "\
1 P2 pts4 4-0 byes0 maps 8-0 frags 0-0 bh 5 ahead points
2 P3 pts3 3-1 byes0 maps 6-2 frags 0-0 bh 5 ahead points
3 P1 pts2 2-2 byes0 maps 4-4 frags 0-0 bh 10 ahead buchholz
4 P6 pts2 2-2 byes0 maps 4-4 frags 0-0 bh 9 ahead points
5 P4 pts1 1-3 byes0 maps 2-6 frags 0-0 bh 9 ahead points
6 P5 pts0 0-4 byes0 maps 0-8 frags 0-0 bh 10 ahead -",
    );
}

fn round(pairs: &[[i64; 2]], bye: Option<i64>) -> SwissRound {
    SwissRound {
        pairs: pairs.iter().map(|[a, b]| [p(*a), p(*b)]).collect(),
        bye: bye.map(p),
    }
}

/// 6 participantes, 3 rodadas, melhor de 3, com os pareamentos gravados como
/// a API os passa. P4 desiste na rodada 2 (W.O. para P2) e sai do pareamento:
/// a rodada 3 tem 5 ativos, e P6, o pior colocado, fica com o bye. Desempates
/// por pontos, Buchholz, saldo de mapas e saldo de frags.
#[test]
fn swiss_with_recorded_pairings_a_withdrawal_and_a_bye() {
    let format = StageFormat::Swiss(SwissConfig {
        best_of: 3,
        rounds: Some(3),
    });
    let rounds = [
        round(&[[1, 4], [2, 5], [3, 6]], None),
        round(&[[1, 3], [5, 6], [2, 4]], None),
        round(&[[5, 3], [1, 2]], Some(6)),
    ];
    let reports = [
        with_frags(played(swiss(1, 1), 1, (2, 0)), 55, 30),
        with_frags(played(swiss(1, 2), 5, (2, 1)), 80, 85),
        with_frags(played(swiss(1, 3), 3, (2, 0)), 60, 41),
        with_frags(played(swiss(2, 1), 3, (2, 1)), 100, 105),
        with_frags(played(swiss(2, 2), 5, (2, 0)), 58, 33),
        walkover(swiss(2, 3), 2),
        with_frags(played(swiss(3, 1), 5, (2, 1)), 88, 90),
        with_frags(played(swiss(3, 2), 2, (2, 0)), 62, 40),
    ];
    let history = |rounds_done: usize, reports_done: usize, out: &[i64]| StageHistory {
        reports: reports[..reports_done].to_vec(),
        rounds: rounds[..rounds_done].to_vec(),
        out: out.iter().map(|n| p(*n)).collect(),
    };

    // Rodada 1 decidida: a hora de parear a 2 (o pareamento é o gravado).
    let after_first = replay(format, 6, history(1, 3, &[]));
    assert!(!after_first.is_finished());
    assert_eq!(after_first.next_round(), Some(rounds[1].clone()));

    // Rodada 2 decidida, P4 fora: a 3 tem bye.
    let after_second = replay(format, 6, history(2, 6, &[4]));
    assert_eq!(after_second.next_round(), Some(rounds[2].clone()));
    assert_text(
        &table_text(&after_second),
        "\
1 P5 pts2 2-0 byes0 maps 4-1 frags 138-118 bh 1 ahead frag_diff
2 P3 pts2 2-0 byes0 maps 4-1 frags 160-146 bh 1 ahead points
3 P1 pts1 1-1 byes0 maps 3-2 frags 160-130 bh 2 ahead frag_diff
4 P2 pts1 1-1 byes0 maps 3-2 frags 85-80 bh 2 ahead points
5 P6 pts0 0-2 byes0 maps 0-4 frags 74-118 bh 4 ahead buchholz
6 P4 pts0 0-2 byes0 maps 0-4 frags 30-55 bh 2 ahead -",
    );

    // Terminado: sem próxima rodada, com a classificação final.
    let stage = replay(format, 6, history(3, 8, &[4]));
    assert_eq!(stage.next_round(), None);
    assert_eq!(stage.total_rounds(), Some(3));
    assert!(stage.is_finished());
    assert_text(
        &matches_text(&stage),
        "\
SW1.1 bo3 P1 v P4 -> P1 2-0
SW1.2 bo3 P2 v P5 -> P5 2-1
SW1.3 bo3 P3 v P6 -> P3 2-0
SW2.1 bo3 P1 v P3 -> P3 2-1
SW2.2 bo3 P5 v P6 -> P5 2-0
SW2.3 bo3 P2 v P4 -> P2 wo
SW3.1 bo3 P5 v P3 -> P5 2-1
SW3.2 bo3 P1 v P2 -> P2 2-0
SW3.3 bo3 bye P6",
    );
    assert_text(
        &table_text(&stage),
        "\
1 P5 pts3 3-0 byes0 maps 6-2 frags 226-208 bh 5 ahead points
2 P3 pts2 2-1 byes0 maps 5-3 frags 250-234 bh 5 ahead buchholz
3 P2 pts2 2-1 byes0 maps 5-2 frags 147-120 bh 4 ahead points
4 P6 pts1 0-2 byes1 maps 0-4 frags 74-118 bh 5 ahead buchholz
5 P1 pts1 1-2 byes0 maps 3-4 frags 200-192 bh 4 ahead points
6 P4 pts0 0-2 byes0 maps 0-4 frags 30-55 bh 3 ahead -",
    );
    assert_text(
        &placements_text(&stage),
        "#1 P5\n#2 P3\n#3 P2\n#4 P6\n#5 P1\n#6 P4",
    );
}
