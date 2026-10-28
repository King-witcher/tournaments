//! A série decidida de uma partida como fica gravada (`match_results`) e o
//! que dela entra no replay. Só a série decidida (pelos mapas, pelo
//! organizador ou pelo W.O. automático) tem linha aqui; a série em andamento
//! fica nos mapas (`games`).

use chrono::{DateTime, Utc};
use engine::{
    core::{MatchId, MatchReport, Outcome, ParticipantId, SeriesScore},
    series,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::ApiError;

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

    pub(super) fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            other => Err(unknown("side", other)),
        }
    }

    /// O participante deste lado entre `(a, b)`.
    pub fn of(self, (a, b): (ParticipantId, ParticipantId)) -> ParticipantId {
        match self {
            Self::A => a,
            Self::B => b,
        }
    }
}

impl From<Side> for series::Side {
    fn from(side: Side) -> Self {
        match side {
            Side::A => Self::A,
            Side::B => Self::B,
        }
    }
}

impl From<series::Side> for Side {
    fn from(side: series::Side) -> Self {
        match side {
            series::Side::A => Self::A,
            series::Side::B => Self::B,
        }
    }
}

/// Como um mapa ou a série foi confirmado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confirmation {
    /// O adversário de quem enviou.
    Opponent,
    /// A janela de contestação acabou sem resposta.
    Deadline,
    /// Organizer ou owner (confirmou, resolveu, lançou, corrigiu ou deu W.O.).
    Organizer,
    /// W.O. automático: o perdedor desistiu ou saiu da comunidade.
    Withdrawal,
    /// A série: os mapas confirmados deram a maioria a um lado.
    Maps,
}

impl Confirmation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opponent => "opponent",
            Self::Deadline => "deadline",
            Self::Organizer => "organizer",
            Self::Withdrawal => "withdrawal",
            Self::Maps => "maps",
        }
    }

    pub(super) fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "opponent" => Ok(Self::Opponent),
            "deadline" => Ok(Self::Deadline),
            "organizer" => Ok(Self::Organizer),
            "withdrawal" => Ok(Self::Withdrawal),
            "maps" => Ok(Self::Maps),
            other => Err(unknown("confirmed_via", other)),
        }
    }
}

pub(super) fn unknown(column: &'static str, value: &str) -> ApiError {
    tracing::error!(column, value, "valor desconhecido em resultados de partida");
    ApiError::internal()
}

/// Uma linha de `match_results` (`db::results`), com os enums já lidos.
#[derive(Clone)]
pub struct StoredResult {
    pub id: i64,
    pub match_id: MatchId,
    pub side_a: ParticipantId,
    pub side_b: ParticipantId,
    pub winner: ParticipantId,
    /// Mapas vencidos por `a` e por `b`; `None` no W.O.
    pub score: Option<(u8, u8)>,
    pub confirmed_via: Confirmation,
    pub confirmed_at: DateTime<Utc>,
    /// Ordem no replay.
    pub position: i32,
}

/// O que `db::results` lê, antes de converter.
pub(super) struct ResultRow {
    pub id: i64,
    pub match_key: String,
    pub side_a: i64,
    pub side_b: i64,
    pub winner: i64,
    pub a_wins: Option<i16>,
    pub b_wins: Option<i16>,
    pub confirmed_via: String,
    pub confirmed_at: DateTime<Utc>,
    pub position: i32,
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
            winner: ParticipantId(r.winner),
            score,
            confirmed_via: Confirmation::from_db(&r.confirmed_via)?,
            confirmed_at: r.confirmed_at,
            position: r.position,
        })
    }
}

impl StoredResult {
    pub fn winner_side(&self) -> Side {
        if self.winner == self.side_a {
            Side::A
        } else {
            Side::B
        }
    }

    pub fn outcome(&self) -> Outcome {
        played_or_walkover(self.winner, self.score)
    }

    pub fn report(&self) -> MatchReport {
        MatchReport {
            match_id: self.match_id,
            outcome: self.outcome(),
        }
    }
}

/// O resultado com placar `score` (mapas de `a` e de `b`), ou W.O. sem ele.
pub fn played_or_walkover(winner: ParticipantId, score: Option<(u8, u8)>) -> Outcome {
    match score {
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
