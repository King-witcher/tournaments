//! Montagem do histórico e o estado em texto compacto, só pela API pública.

use engine::core::{
    BracketSide, MatchId, MatchReport, MatchState, Outcome, ParticipantId, SeriesFrags,
    SeriesScore, StageReport,
};
use engine::formats::table::Criterion;
use engine::formats::{StageBracket, StageFormat, StageHistory};

pub fn p(n: i64) -> ParticipantId {
    ParticipantId(n)
}

/// Os participantes de 1 a `n`, na ordem dos seeds (o 1 é o seed 1).
pub fn seeds(n: i64) -> Vec<ParticipantId> {
    (1..=n).map(ParticipantId).collect()
}

fn id(side: BracketSide, round: u16, index: u16) -> MatchId {
    MatchId { side, round, index }
}

/// Chave de cima (`W2.1`).
pub fn upper(round: u16, index: u16) -> MatchId {
    id(BracketSide::Winners, round, index)
}

/// Chave de baixo (`L2.1`).
pub fn lower(round: u16, index: u16) -> MatchId {
    id(BracketSide::Losers, round, index)
}

/// Grande final: rodada 1 é a final, 2 é o reset (`G1.1`, `G2.1`).
pub fn grand(round: u16) -> MatchId {
    id(BracketSide::GrandFinal, round, 1)
}

/// Round robin (`RR3.2`).
pub fn robin(round: u16, index: u16) -> MatchId {
    id(BracketSide::RoundRobin, round, index)
}

/// Suíço (`SW3.2`).
pub fn swiss(round: u16, index: u16) -> MatchId {
    id(BracketSide::Swiss, round, index)
}

/// Série jogada: o vencedor e o placar em mapas (do vencedor, do perdedor).
pub fn played(match_id: MatchId, winner: i64, score: (u8, u8)) -> StageReport {
    StageReport::from(MatchReport {
        match_id,
        outcome: Outcome::Played {
            winner: p(winner),
            score: SeriesScore {
                winner: score.0,
                loser: score.1,
            },
        },
    })
}

/// W.O.: o vencedor leva sem jogar, e a série não tem frags.
pub fn walkover(match_id: MatchId, winner: i64) -> StageReport {
    StageReport::from(MatchReport {
        match_id,
        outcome: Outcome::Walkover { winner: p(winner) },
    })
}

/// Empate na série par: `maps` mapas para cada lado.
pub fn drawn(match_id: MatchId, maps: u8) -> StageReport {
    StageReport::from(MatchReport {
        match_id,
        outcome: Outcome::Drawn { maps },
    })
}

/// A série com os frags somados dos mapas (do vencedor da série, do perdedor;
/// no empate, dos lados `a` e `b`). Ficam nessa ordem até `replay`, que sabe de
/// que lado cada um joga.
pub fn with_frags(mut report: StageReport, winner: i32, loser: i32) -> StageReport {
    report.frags = Some(SeriesFrags {
        a: winner,
        b: loser,
    });
    report
}

/// Refaz a fase como a API: configuração, participantes de 1 a `n` em ordem
/// de seed e o histórico gravado. Um histórico que não fecha derruba o teste.
/// Os frags de `with_frags` vão para os lados da partida: os lados saem de um
/// replay sem frags, que não mudam quem joga com quem.
pub fn replay(format: StageFormat, n: i64, mut history: StageHistory) -> StageBracket {
    let bare = StageHistory {
        reports: history
            .reports
            .iter()
            .map(|r| StageReport::from(r.report))
            .collect(),
        ..history.clone()
    };
    let sides = redo(format, n, bare);
    for r in &mut history.reports {
        let id = r.report.match_id;
        let state = sides.matches().iter().find(|m| m.id == id).map(|m| m.state);
        if let (Some(MatchState::Completed { b, .. }), Some(frags)) = (state, r.frags.as_mut())
            && r.report.outcome.winner() == Some(b)
        {
            *frags = SeriesFrags {
                a: frags.b,
                b: frags.a,
            };
        }
    }
    redo(format, n, history)
}

fn redo(format: StageFormat, n: i64, history: StageHistory) -> StageBracket {
    StageBracket::replay(format, seeds(n), history)
        .unwrap_or_else(|e| panic!("o histórico gravado deixou de refazer: {e:?}"))
}

/// Só resultados, sem pareamentos nem desistentes (todos menos o suíço).
pub fn results(reports: &[StageReport]) -> StageHistory {
    StageHistory {
        reports: reports.to_vec(),
        ..StageHistory::default()
    }
}

#[track_caller]
pub fn assert_text(actual: &str, expected: &str) {
    assert!(
        actual == expected,
        "\n--- obtido ---\n{actual}\n--- esperado ---\n{expected}\n"
    );
}

fn who(p: ParticipantId) -> String {
    format!("P{}", p.0)
}

fn outcome_text(outcome: Outcome) -> String {
    match outcome {
        Outcome::Played { winner, score } => {
            format!("{} {}-{}", who(winner), score.winner, score.loser)
        }
        Outcome::Walkover { winner } => format!("{} wo", who(winner)),
        Outcome::Drawn { maps } => format!("draw {maps}-{maps}"),
    }
}

fn state_text(state: MatchState) -> String {
    match state {
        MatchState::Pending => "pending".into(),
        MatchState::Ready { a, b } => format!("{} v {}", who(a), who(b)),
        MatchState::Bye { advanced: Some(p) } => format!("bye {}", who(p)),
        MatchState::Bye { advanced: None } => "bye -".into(),
        MatchState::Completed { a, b, outcome } => {
            format!("{} v {} -> {}", who(a), who(b), outcome_text(outcome))
        }
        MatchState::NotNeeded => "not needed".into(),
    }
}

/// `W1.2`, `L3.1`, `G2.1`, `RR1.1`, `SW2.3`.
pub fn label(id: MatchId) -> String {
    let side = match id.side {
        BracketSide::Winners => "W",
        BracketSide::Losers => "L",
        BracketSide::GrandFinal => "G",
        BracketSide::RoundRobin => "RR",
        BracketSide::Swiss => "SW",
    };
    format!("{side}{}.{}", id.round, id.index)
}

/// Uma linha por partida, em ordem de id: `rótulo boN estado`, com os lados
/// e o resultado nas decididas.
pub fn matches_text(stage: &StageBracket) -> String {
    stage
        .matches()
        .iter()
        .map(|m| format!("{} bo{} {}", label(m.id), m.best_of, state_text(m.state)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// As partidas prontas agora, em ordem de id (`W3.1, L2.1`).
pub fn ready_text(stage: &StageBracket) -> String {
    stage
        .ready_matches()
        .map(|m| label(m.id))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Uma linha por colocação: `#1 P3`, e `#5-6 P5` na faixa empatada.
pub fn placements_text(stage: &StageBracket) -> String {
    let lines: Vec<String> = stage
        .placements()
        .iter()
        .map(|p| {
            let rank = if p.tied > 1 {
                format!("#{}-{}", p.rank, p.rank + p.tied - 1)
            } else {
                format!("#{}", p.rank)
            };
            format!("{rank} {}", who(p.participant))
        })
        .collect();
    lines.join("\n")
}

fn criterion_text(criterion: Option<Criterion>) -> &'static str {
    match criterion {
        None => "-",
        Some(Criterion::Points) => "points",
        Some(Criterion::SeriesDiff) => "series_diff",
        Some(Criterion::Buchholz) => "buchholz",
        Some(Criterion::HeadToHead) => "head_to_head",
        Some(Criterion::MapDiff) => "map_diff",
        Some(Criterion::FragDiff) => "frag_diff",
        Some(Criterion::Seed) => "seed",
    }
}

/// Uma linha por posição da tabela: pontos, vitórias-derrotas (com empates,
/// vitórias-empates-derrotas), byes, mapas, frags, Buchholz e o critério que pôs
/// a linha à frente da seguinte.
pub fn table_text(stage: &StageBracket) -> String {
    let rows = stage.table().expect("fase de tabela");
    rows.iter()
        .map(|r| {
            let buchholz = r.buchholz.map_or("-".to_string(), |b| b.to_string());
            let record = if r.draws > 0 {
                format!("{}-{}-{}", r.wins, r.draws, r.losses)
            } else {
                format!("{}-{}", r.wins, r.losses)
            };
            format!(
                "{} {} pts{} {} byes{} maps {}-{} frags {}-{} bh {} ahead {}",
                r.position,
                who(r.participant),
                r.points,
                record,
                r.byes,
                r.maps_won,
                r.maps_lost,
                r.frags_for,
                r.frags_against,
                buchholz,
                criterion_text(r.ahead_by),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// O estado de ponta a ponta: as partidas, se terminou e as colocações.
pub fn snapshot(stage: &StageBracket) -> String {
    format!(
        "{}\nfinished: {}\n{}",
        matches_text(stage),
        if stage.is_finished() { "yes" } else { "no" },
        placements_text(stage)
    )
}
