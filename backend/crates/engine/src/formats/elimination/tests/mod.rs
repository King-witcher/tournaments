//! Testes da eliminação: helpers compartilhados; casos por assunto nos submódulos.

mod double;
mod props;
mod report;
mod single;
mod validation;

use super::*;
use crate::core::SeriesScore;
use crate::series::wins_needed;
use proptest::prelude::*;

const W: BracketSide = BracketSide::Winners;
const L: BracketSide = BracketSide::Losers;
const GF: BracketSide = BracketSide::GrandFinal;

fn p(n: i64) -> ParticipantId {
    ParticipantId(n)
}

fn seeds(n: i64) -> Vec<ParticipantId> {
    (1..=n).map(p).collect()
}

fn id(side: BracketSide, round: u16, index: u16) -> MatchId {
    MatchId { side, round, index }
}

fn single(best_of: u8, finals: u8) -> EliminationConfig {
    EliminationConfig {
        kind: EliminationKind::Single,
        series: SeriesLength {
            default: best_of,
            finals,
        },
    }
}

fn double(reset: bool) -> EliminationConfig {
    EliminationConfig {
        kind: EliminationKind::Double {
            grand_final_reset: reset,
        },
        series: SeriesLength {
            default: 3,
            finals: 5,
        },
    }
}

fn played(m: &Match, winner: ParticipantId) -> MatchReport {
    MatchReport {
        match_id: m.id,
        outcome: Outcome::Played {
            winner,
            score: SeriesScore {
                winner: wins_needed(m.best_of),
                loser: 0,
            },
        },
    }
}

fn ready_pair(b: &EliminationBracket, mid: MatchId) -> (ParticipantId, ParticipantId) {
    match b.get(mid).unwrap().state {
        MatchState::Ready { a, b } => (a, b),
        other => panic!("{mid:?} não está pronta: {other:?}"),
    }
}

/// Vence quem tem o menor id (o seed menor). Para antes da grande final
/// se `stop_at_gf`. Retorna a ordem dos resultados.
fn play_favorites(b: &mut EliminationBracket, stop_at_gf: bool) -> Vec<MatchReport> {
    let mut reports = Vec::new();
    loop {
        let next = b
            .ready_matches()
            .find(|m| !(stop_at_gf && m.id.side == GF))
            .copied();
        let Some(m) = next else { break };
        let MatchState::Ready { a, b: other } = m.state else {
            unreachable!()
        };
        let r = played(&m, a.min(other));
        b.report(r).unwrap();
        reports.push(r);
    }
    reports
}

fn count(b: &EliminationBracket, side: BracketSide) -> usize {
    b.matches().iter().filter(|m| m.id.side == side).count()
}
