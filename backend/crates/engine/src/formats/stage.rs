//! A fase de qualquer formato, para quem não precisa saber qual é (a API):
//! partidas, estado de cada uma, fim, colocação, a tabela dos formatos de
//! tabela e o próximo pareamento do suíço, sempre refeitos por `replay`.

use super::StageFormat;
use super::elimination::EliminationBracket;
use super::round_robin::RoundRobinBracket;
use super::swiss::{SwissBracket, SwissRound};
use super::table::TableRow;
use crate::core::{
    EngineError, Match, MatchId, MatchState, Occupant, ParticipantId, Placement, StageReport,
};

/// O histórico de uma fase: tudo que a API grava dela, menos a configuração
/// e os participantes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StageHistory {
    /// As séries decididas, na ordem em que passaram a valer.
    pub reports: Vec<StageReport>,
    /// Suíço: os pareamentos gravados, da rodada 1 em diante. Nos outros
    /// formatos, vazio.
    pub rounds: Vec<SwissRound>,
    /// Quem está fora (desistiu ou saiu). Só o suíço usa: não pareia e, com
    /// menos de 2 ativos, termina. Nos outros, a API resolve as partidas de
    /// quem saiu por W.O.
    pub out: Vec<ParticipantId>,
}

/// O estado de uma fase, de qualquer formato.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StageBracket {
    Elimination(EliminationBracket),
    RoundRobin(RoundRobinBracket),
    Swiss(SwissBracket),
}

impl StageBracket {
    /// A fase sem histórico: no suíço, ainda sem rodada (a primeira sai de
    /// `next_round`). Erros: os de montar o formato.
    pub fn new(format: StageFormat, seeded: Vec<ParticipantId>) -> Result<Self, EngineError> {
        Self::replay(format, seeded, StageHistory::default())
    }

    /// Reconstrói a fase. `seeded` em ordem de seed. Erros: os do `replay`
    /// de cada formato; `InvalidPairing` com pareamento fora do suíço.
    pub fn replay(
        format: StageFormat,
        seeded: Vec<ParticipantId>,
        history: StageHistory,
    ) -> Result<Self, EngineError> {
        if !matches!(format, StageFormat::Swiss(_)) && !history.rounds.is_empty() {
            return Err(EngineError::InvalidPairing { round: 1 });
        }
        Ok(match format {
            StageFormat::Elimination(c) => Self::Elimination(EliminationBracket::replay(
                c,
                seeded,
                history.reports.into_iter().map(|r| r.report),
            )?),
            StageFormat::RoundRobin(c) => {
                Self::RoundRobin(RoundRobinBracket::replay(c, seeded, history.reports)?)
            }
            StageFormat::Swiss(c) => Self::Swiss(SwissBracket::replay(
                c,
                seeded,
                history.rounds,
                history.reports,
                &history.out,
            )?),
        })
    }

    pub fn format(&self) -> StageFormat {
        match self {
            Self::Elimination(b) => StageFormat::Elimination(b.config()),
            Self::RoundRobin(b) => StageFormat::RoundRobin(b.config()),
            Self::Swiss(b) => StageFormat::Swiss(b.config()),
        }
    }

    /// Na ordem dos seeds.
    pub fn participants(&self) -> &[ParticipantId] {
        match self {
            Self::Elimination(b) => b.participants(),
            Self::RoundRobin(b) => b.participants(),
            Self::Swiss(b) => b.participants(),
        }
    }

    /// Todas as partidas, em ordem de id. No suíço, só as das rodadas já
    /// pareadas.
    pub fn matches(&self) -> &[Match] {
        match self {
            Self::Elimination(b) => b.matches(),
            Self::RoundRobin(b) => b.matches(),
            Self::Swiss(b) => b.matches(),
        }
    }

    pub fn get(&self, id: MatchId) -> Option<&Match> {
        match self {
            Self::Elimination(b) => b.get(id),
            Self::RoundRobin(b) => b.get(id),
            Self::Swiss(b) => b.get(id),
        }
    }

    /// Quem ocupa cada lado (`a`, `b`) da partida agora; `None` se ela não
    /// existe. Nos formatos de tabela, sempre conhecido.
    pub fn occupants(&self, id: MatchId) -> Option<(Occupant, Occupant)> {
        match self {
            Self::Elimination(b) => b.occupants(id),
            Self::RoundRobin(b) => b.occupants(id),
            Self::Swiss(b) => b.occupants(id),
        }
    }

    /// Partidas que podem ser jogadas agora (`MatchState::Ready`).
    pub fn ready_matches(&self) -> impl Iterator<Item = &Match> {
        self.matches()
            .iter()
            .filter(|m| matches!(m.state, MatchState::Ready { .. }))
    }

    /// Registra um resultado. Os frags só contam nos formatos de tabela.
    pub fn report(&mut self, report: StageReport) -> Result<(), EngineError> {
        match self {
            Self::Elimination(b) => b.report(report.report),
            Self::RoundRobin(b) => b.report(report),
            Self::Swiss(b) => b.report(report),
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            Self::Elimination(b) => b.is_finished(),
            Self::RoundRobin(b) => b.is_finished(),
            Self::Swiss(b) => b.is_finished(),
        }
    }

    /// Colocações já definidas. Na eliminação, os eliminados (faixas com
    /// empate) e, no fim, campeão e vice; nos formatos de tabela, todos, só
    /// no fim, uma posição cada.
    pub fn placements(&self) -> Vec<Placement> {
        match self {
            Self::Elimination(b) => b.standings(),
            Self::RoundRobin(b) => b.placements(),
            Self::Swiss(b) => b.placements(),
        }
    }

    /// A tabela atual, em ordem de posição; `None` na eliminação. Em ordem, é
    /// a semente de uma fase seguinte.
    pub fn table(&self) -> Option<&[TableRow]> {
        match self {
            Self::Elimination(_) => None,
            Self::RoundRobin(b) => Some(b.table()),
            Self::Swiss(b) => Some(b.table()),
        }
    }

    /// Número de rodadas dos formatos de tabela (no suíço, as da fase, não só
    /// as pareadas); `None` na eliminação.
    pub fn total_rounds(&self) -> Option<u16> {
        match self {
            Self::Elimination(_) => None,
            Self::RoundRobin(b) => Some(b.rounds()),
            Self::Swiss(b) => Some(b.total_rounds()),
        }
    }

    /// Suíço: o pareamento da próxima rodada, quando é hora (ver
    /// `SwissBracket::next_round`). Nos outros formatos, `None`.
    pub fn next_round(&self) -> Option<SwissRound> {
        match self {
            Self::Swiss(b) => b.next_round(),
            _ => None,
        }
    }

    /// Máximo de mapas que um participante pode jogar na fase.
    pub fn worst_case_games(&self) -> u32 {
        match self {
            Self::Elimination(b) => b.worst_case_games(),
            Self::RoundRobin(b) => b.worst_case_games(),
            Self::Swiss(b) => b.worst_case_games(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{MatchReport, Outcome, SeriesScore};
    use crate::formats::elimination::{EliminationConfig, EliminationKind};
    use crate::formats::round_robin::RoundRobinConfig;
    use crate::formats::swiss::SwissConfig;
    use crate::series::{SeriesLength, wins_needed};

    fn seeds(n: i64) -> Vec<ParticipantId> {
        (1..=n).map(ParticipantId).collect()
    }

    fn favorite(m: &Match) -> StageReport {
        let MatchState::Ready { a, b } = m.state else {
            panic!("{:?} não está pronta", m.id)
        };
        StageReport::from(MatchReport {
            match_id: m.id,
            outcome: Outcome::Played {
                winner: a.min(b),
                score: SeriesScore {
                    winner: wins_needed(m.best_of),
                    loser: 0,
                },
            },
        })
    }

    /// Joga a fase até o fim pelos favoritos, gravando os pareamentos do
    /// suíço como a API faria, e confere o replay a cada passo.
    fn play(format: StageFormat, n: i64) -> (StageBracket, StageHistory) {
        let mut history = StageHistory::default();
        let mut stage = StageBracket::new(format, seeds(n)).unwrap();
        loop {
            if let Some(round) = stage.next_round() {
                history.rounds.push(round);
                stage = StageBracket::replay(format, seeds(n), history.clone()).unwrap();
                continue;
            }
            let Some(m) = stage.ready_matches().next().copied() else {
                break;
            };
            let r = favorite(&m);
            stage.report(r).unwrap();
            history.reports.push(r);
            assert_eq!(
                stage,
                StageBracket::replay(format, seeds(n), history.clone()).unwrap()
            );
        }
        (stage, history)
    }

    fn formats() -> [StageFormat; 3] {
        [
            StageFormat::Elimination(EliminationConfig {
                kind: EliminationKind::Double {
                    grand_final_reset: true,
                },
                series: SeriesLength {
                    default: 3,
                    finals: 5,
                },
            }),
            StageFormat::RoundRobin(RoundRobinConfig { best_of: 3 }),
            StageFormat::Swiss(SwissConfig {
                best_of: 1,
                rounds: None,
            }),
        ]
    }

    #[test]
    fn every_format_plays_to_the_end_with_the_favorite_first() {
        for format in formats() {
            for n in [2, 5, 8] {
                let (stage, _) = play(format, n);
                assert!(stage.is_finished(), "{format:?} {n}");
                assert_eq!(stage.format(), format);
                let placements = stage.placements();
                let first = placements.iter().find(|p| p.rank == 1).unwrap();
                assert_eq!(first.participant, ParticipantId(1), "{format:?} {n}");
                assert_eq!(
                    stage.table().is_some(),
                    !matches!(format, StageFormat::Elimination(_))
                );
            }
        }
    }

    #[test]
    fn elimination_is_the_same_bracket() {
        let format = formats()[0];
        let StageFormat::Elimination(config) = format else {
            unreachable!()
        };
        let (stage, history) = play(format, 6);
        let reports = history.reports.iter().map(|r| r.report);
        let direct = EliminationBracket::replay(config, seeds(6), reports).unwrap();
        assert_eq!(stage.placements(), direct.standings());
        assert_eq!(stage, StageBracket::Elimination(direct));
        assert_eq!(stage.total_rounds(), None);
        assert_eq!(stage.next_round(), None);
    }

    #[test]
    fn pairings_only_in_the_swiss() {
        let history = StageHistory {
            rounds: vec![SwissRound {
                pairs: vec![[ParticipantId(1), ParticipantId(2)]],
                bye: None,
            }],
            ..StageHistory::default()
        };
        for format in &formats()[..2] {
            assert_eq!(
                StageBracket::replay(*format, seeds(2), history.clone()),
                Err(EngineError::InvalidPairing { round: 1 })
            );
        }
        assert!(StageBracket::replay(formats()[2], seeds(2), history).is_ok());
    }

    #[test]
    fn worst_case_matches_the_format() {
        for format in formats() {
            let stage = StageBracket::new(format, seeds(8)).unwrap();
            assert_eq!(Ok(stage.worst_case_games()), format.worst_case_games(8));
        }
    }
}
