//! O resultado vigente de uma partida como fica gravado (`match_results`) e o
//! que dele entra no replay. Só o confirmado vale na chave.

use chrono::{DateTime, Utc};
use engine::core::{MatchId, MatchReport, Outcome, ParticipantId, SeriesScore};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::ApiError;

/// pendente (esperando o adversário) → confirmado, ou → contestado
/// (esperando o organizador) → confirmado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    Pending,
    Contested,
    Confirmed,
}

impl ResultStatus {
    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "pending" => Ok(Self::Pending),
            "contested" => Ok(Self::Contested),
            "confirmed" => Ok(Self::Confirmed),
            other => Err(unknown("status", other)),
        }
    }
}

/// Um dos dois lados de uma partida.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    A,
    B,
}

impl Side {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
        }
    }

    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            other => Err(unknown("side", other)),
        }
    }
}

/// Como o resultado foi confirmado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confirmation {
    /// O adversário de quem enviou.
    Opponent,
    /// A janela de contestação acabou sem resposta.
    Deadline,
    /// Organizer ou owner (confirmou, resolveu, lançou ou corrigiu).
    Organizer,
    /// W.O. automático: o perdedor desistiu ou saiu da comunidade.
    Withdrawal,
}

impl Confirmation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opponent => "opponent",
            Self::Deadline => "deadline",
            Self::Organizer => "organizer",
            Self::Withdrawal => "withdrawal",
        }
    }

    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "opponent" => Ok(Self::Opponent),
            "deadline" => Ok(Self::Deadline),
            "organizer" => Ok(Self::Organizer),
            "withdrawal" => Ok(Self::Withdrawal),
            other => Err(unknown("confirmed_via", other)),
        }
    }
}

fn unknown(column: &'static str, value: &str) -> ApiError {
    tracing::error!(column, value, "valor desconhecido em match_results");
    ApiError::internal()
}

/// Uma linha de `match_results` (`db::results`), com os enums já lidos.
#[derive(Clone)]
pub struct StoredResult {
    pub id: i64,
    pub match_id: MatchId,
    pub side_a: ParticipantId,
    pub side_b: ParticipantId,
    pub status: ResultStatus,
    pub winner: ParticipantId,
    /// Mapas vencidos por `a` e por `b`; `None` no W.O.
    pub score: Option<(u8, u8)>,
    /// Lado de quem enviou; `None` quando o organizador lançou.
    pub reported_side: Option<Side>,
    pub reported_at: DateTime<Utc>,
    pub confirm_by: Option<DateTime<Utc>>,
    pub contested_at: Option<DateTime<Utc>>,
    pub contest_comment: Option<String>,
    pub confirmed_via: Option<Confirmation>,
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Ordem no replay; só no confirmado.
    pub position: Option<i32>,
}

/// O que `db::results` lê, antes de converter.
pub(super) struct ResultRow {
    pub id: i64,
    pub match_key: String,
    pub side_a: i64,
    pub side_b: i64,
    pub status: String,
    pub winner: i64,
    pub a_wins: Option<i16>,
    pub b_wins: Option<i16>,
    pub reported_side: Option<String>,
    pub reported_at: DateTime<Utc>,
    pub confirm_by: Option<DateTime<Utc>>,
    pub contested_at: Option<DateTime<Utc>>,
    pub contest_comment: Option<String>,
    pub confirmed_via: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub position: Option<i32>,
}

impl TryFrom<ResultRow> for StoredResult {
    type Error = ApiError;

    fn try_from(r: ResultRow) -> Result<Self, ApiError> {
        let wins = |w: i16| u8::try_from(w).map_err(|_| ApiError::internal());
        let score = match (r.a_wins, r.b_wins) {
            (Some(a), Some(b)) => Some((wins(a)?, wins(b)?)),
            _ => None,
        };
        Ok(Self {
            id: r.id,
            match_id: super::key::parse(&r.match_key)
                .ok_or_else(|| unknown("match_key", &r.match_key))?,
            side_a: ParticipantId(r.side_a),
            side_b: ParticipantId(r.side_b),
            status: ResultStatus::from_db(&r.status)?,
            winner: ParticipantId(r.winner),
            score,
            reported_side: r.reported_side.as_deref().map(Side::from_db).transpose()?,
            reported_at: r.reported_at,
            confirm_by: r.confirm_by,
            contested_at: r.contested_at,
            contest_comment: r.contest_comment,
            confirmed_via: r
                .confirmed_via
                .as_deref()
                .map(Confirmation::from_db)
                .transpose()?,
            confirmed_at: r.confirmed_at,
            position: r.position,
        })
    }
}

impl StoredResult {
    pub fn side_of(&self, p: ParticipantId) -> Option<Side> {
        if p == self.side_a {
            Some(Side::A)
        } else if p == self.side_b {
            Some(Side::B)
        } else {
            None
        }
    }

    pub fn winner_side(&self) -> Side {
        if self.winner == self.side_a {
            Side::A
        } else {
            Side::B
        }
    }

    pub fn outcome(&self) -> Outcome {
        let winner = self.winner;
        match self.score {
            None => Outcome::Walkover { winner },
            Some((a, b)) => Outcome::Played {
                winner,
                score: SeriesScore {
                    winner: a.max(b),
                    loser: a.min(b),
                },
            },
        }
    }

    pub fn report(&self) -> MatchReport {
        MatchReport {
            match_id: self.match_id,
            outcome: self.outcome(),
        }
    }
}
