//! Testes do suíço: helpers; casos por assunto nos submódulos.

mod history;
mod props;
mod rounds;

use std::collections::HashSet;

use super::*;
use crate::core::{MatchReport, MatchState, Outcome, SeriesFrags, SeriesScore, Slot};
use crate::series::wins_needed;

fn p(n: i64) -> ParticipantId {
    ParticipantId(n)
}

fn seeds(n: i64) -> Vec<ParticipantId> {
    (1..=n).map(p).collect()
}

fn id(round: u16, index: u16) -> MatchId {
    MatchId {
        side: BracketSide::Swiss,
        round,
        index,
    }
}

fn config(rounds: Option<u16>) -> SwissConfig {
    SwissConfig { best_of: 3, rounds }
}

/// O pareamento em números: os pares e o bye.
fn plain(round: &SwissRound) -> (Vec<[i64; 2]>, Option<i64>) {
    (
        round.pairs.iter().map(|[a, b]| [a.0, b.0]).collect(),
        round.bye.map(|b| b.0),
    )
}

fn round(pairs: &[[i64; 2]], bye: Option<i64>) -> SwissRound {
    SwissRound {
        pairs: pairs.iter().map(|[a, b]| [p(*a), p(*b)]).collect(),
        bye: bye.map(p),
    }
}

fn played(m: &Match, winner: ParticipantId, loser_maps: u8) -> StageReport {
    StageReport {
        report: MatchReport {
            match_id: m.id,
            outcome: Outcome::Played {
                winner,
                score: SeriesScore {
                    winner: wins_needed(m.best_of),
                    loser: loser_maps,
                },
            },
        },
        frags: Some(SeriesFrags::default()),
    }
}

fn walkover(m: &Match, winner: ParticipantId) -> StageReport {
    StageReport::from(MatchReport {
        match_id: m.id,
        outcome: Outcome::Walkover { winner },
    })
}

/// O histórico como a API guarda: pareamentos, resultados e quem saiu.
#[derive(Clone)]
struct Run {
    config: SwissConfig,
    seeded: Vec<ParticipantId>,
    rounds: Vec<SwissRound>,
    reports: Vec<StageReport>,
    out: Vec<ParticipantId>,
}

impl Run {
    fn new(n: i64, rounds: Option<u16>) -> Self {
        Self {
            config: config(rounds),
            seeded: seeds(n),
            rounds: Vec::new(),
            reports: Vec::new(),
            out: Vec::new(),
        }
    }

    fn bracket(&self) -> SwissBracket {
        SwissBracket::replay(
            self.config,
            self.seeded.clone(),
            self.rounds.clone(),
            self.reports.clone(),
            &self.out,
        )
        .unwrap()
    }

    /// Pareia a próxima rodada, se é hora, e grava.
    fn pair(&mut self) -> Option<SwissRound> {
        let r = self.bracket().next_round()?;
        self.rounds.push(r.clone());
        Some(r)
    }

    /// Decide toda partida pronta, como a API: quem está fora perde por W.O.
    /// (os dois fora, vence o seed melhor); senão vence `winner` por 2 x 0.
    fn decide(&mut self, winner: impl Fn(ParticipantId, ParticipantId) -> ParticipantId) {
        let b = self.bracket();
        for m in b.matches() {
            let MatchState::Ready { a, b: other } = m.state else {
                continue;
            };
            let report = match (self.out.contains(&a), self.out.contains(&other)) {
                (false, false) => played(m, winner(a, other), 0),
                (true, false) => walkover(m, other),
                (false, true) => walkover(m, a),
                (true, true) => walkover(m, a.min(other)),
            };
            self.reports.push(report);
        }
    }

    /// Pareia e decide até o fim.
    fn play(&mut self, winner: impl Fn(ParticipantId, ParticipantId) -> ParticipantId) {
        while self.pair().is_some() {
            self.decide(&winner);
        }
    }
}

fn favorite(a: ParticipantId, b: ParticipantId) -> ParticipantId {
    a.min(b)
}
