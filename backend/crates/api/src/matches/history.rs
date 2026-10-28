//! Histórico das partidas: o que cada evento grava (`match_events.kind`) e
//! como um resultado vira as colunas do evento.

use engine::core::Outcome;

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
            Self::Voided => "voided",
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
            Voided,
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
    /// Quem agiu (`None` só na confirmação por tempo, que o SQL grava sozinho).
    pub actor: i64,
    /// O resultado que a ação deixou, quando ela define um.
    pub outcome: Option<Outcome>,
    pub comment: Option<&'a str>,
}

/// `(vencedor, mapas do vencedor, mapas do perdedor)` de um resultado, como
/// o evento grava.
pub fn outcome_columns(outcome: Option<Outcome>) -> (Option<i64>, Option<i16>, Option<i16>) {
    match outcome {
        None => (None, None, None),
        Some(Outcome::Walkover { winner }) => (Some(winner.0), None, None),
        Some(Outcome::Played { winner, score }) => (
            Some(winner.0),
            Some(i16::from(score.winner)),
            Some(i16::from(score.loser)),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip() {
        for kind in ["reported", "auto_confirmed", "voided"] {
            assert_eq!(EventKind::from_db(kind).unwrap().as_str(), kind);
        }
        assert!(EventKind::from_db("undone").is_err());
    }
}
