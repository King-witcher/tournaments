//! Histórico das partidas: o que cada evento grava (`match_events.kind`). O
//! evento de um mapa (`game`) grava o vencedor do mapa nas ações que o
//! definem; o da série, o vencedor e o placar (nada no W.O.).

use chrono::{DateTime, Utc};
use engine::core::{Outcome, ParticipantId};

use super::types::EventKind;
use crate::shared::ApiError;

impl EventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reported => "reported",
            Self::Confirmed => "confirmed",
            Self::AutoConfirmed => "auto_confirmed",
            Self::Contested => "contested",
            Self::Resolved => "resolved",
            Self::Entered => "entered",
            Self::Corrected => "corrected",
            Self::Walkover => "walkover",
            Self::AutoWalkover => "auto_walkover",
            Self::Voided => "voided",
            Self::Replay => "replay",
            Self::Decided => "decided",
            Self::Reopened => "reopened",
        }
    }

    /// Lê `match_events.kind` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        use EventKind::*;
        [
            Reported,
            Confirmed,
            AutoConfirmed,
            Contested,
            Resolved,
            Entered,
            Corrected,
            Walkover,
            AutoWalkover,
            Voided,
            Replay,
            Decided,
            Reopened,
        ]
        .into_iter()
        .find(|kind| kind.as_str() == value)
        .ok_or_else(|| {
            tracing::error!(kind = value, "evento de partida desconhecido no banco");
            ApiError::internal()
        })
    }
}

/// Um evento a gravar.
pub struct NewEvent<'a> {
    pub kind: EventKind,
    /// Quem agiu; `None` no que acontece sozinho (W.O. automático, série
    /// decidida).
    pub actor: Option<i64>,
    /// O mapa; `None`: a série inteira.
    pub game: Option<i16>,
    pub winner: Option<ParticipantId>,
    /// Mapas do vencedor e do perdedor da série.
    pub maps: Option<(u8, u8)>,
    pub comment: Option<&'a str>,
    /// Quando; `None`: agora.
    pub at: Option<DateTime<Utc>>,
}

impl<'a> NewEvent<'a> {
    /// Ação sobre o mapa `game`; `winner` nas que definem o vencedor dele.
    pub fn game(
        kind: EventKind,
        actor: Option<i64>,
        game: i16,
        winner: Option<ParticipantId>,
    ) -> Self {
        Self {
            kind,
            actor,
            game: Some(game),
            winner,
            maps: None,
            comment: None,
            at: None,
        }
    }

    /// Ação sobre a série inteira, com o resultado que ela deixou.
    pub fn series(kind: EventKind, actor: Option<i64>, outcome: Option<Outcome>) -> Self {
        let (winner, maps) = match outcome {
            None => (None, None),
            Some(Outcome::Walkover { winner }) => (Some(winner), None),
            Some(Outcome::Played { winner, score }) => {
                (Some(winner), Some((score.winner, score.loser)))
            }
        };
        Self {
            kind,
            actor,
            game: None,
            winner,
            maps,
            comment: None,
            at: None,
        }
    }

    pub fn with_comment(self, comment: Option<&'a str>) -> Self {
        Self { comment, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip() {
        for kind in [
            "reported",
            "auto_confirmed",
            "auto_walkover",
            "voided",
            "replay",
            "decided",
        ] {
            assert_eq!(EventKind::from_db(kind).unwrap().as_str(), kind);
        }
        assert!(EventKind::from_db("undone").is_err());
    }
}
