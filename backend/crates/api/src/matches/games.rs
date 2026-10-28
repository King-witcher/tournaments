//! Os mapas de uma série como ficam gravados (`match_games`) e o que deles
//! vai para o `engine` (`series::games`), que decide o próximo mapa, onde a
//! série fecha e quais mapas ficam depois da decisão.

use chrono::{DateTime, Utc};
use engine::{
    core::{MapId, MatchId, ParticipantId},
    series::{self, SeriesGame},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::result::{Confirmation, Side, unknown};
use crate::shared::ApiError;

/// pendente (esperando o adversário) → confirmado, ou → contestado
/// (esperando o organizador) → confirmado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    Pending,
    Contested,
    Confirmed,
}

impl GameStatus {
    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "pending" => Ok(Self::Pending),
            "contested" => Ok(Self::Contested),
            "confirmed" => Ok(Self::Confirmed),
            other => Err(unknown("status", other)),
        }
    }
}

impl From<GameStatus> for series::GameStatus {
    fn from(status: GameStatus) -> Self {
        match status {
            GameStatus::Pending => Self::Pending,
            GameStatus::Contested => Self::Contested,
            GameStatus::Confirmed => Self::Confirmed,
        }
    }
}

/// Um mapa gravado, com os enums já lidos.
#[derive(Clone)]
pub struct StoredGame {
    pub id: i64,
    pub match_id: MatchId,
    pub number: i16,
    pub side_a: ParticipantId,
    pub side_b: ParticipantId,
    pub winner: ParticipantId,
    /// `None` só em série antiga, sem detalhe por mapa.
    pub map: Option<GameMap>,
    /// Frags de `a` e de `b`, se vieram.
    pub frags: Option<(i16, i16)>,
    pub status: GameStatus,
    /// Lado de quem enviou; `None` quando o organizador lançou.
    pub reported_side: Option<Side>,
    pub reported_at: DateTime<Utc>,
    pub confirm_by: Option<DateTime<Utc>>,
    pub contested_at: Option<DateTime<Utc>>,
    pub contest_comment: Option<String>,
    pub confirmed_via: Option<Confirmation>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct GameMap {
    pub id: MapId,
    pub slug: String,
    pub name: String,
}

/// O que `db::games` lê, antes de converter.
pub(super) struct GameRow {
    pub id: i64,
    pub match_key: String,
    pub number: i16,
    pub side_a: i64,
    pub side_b: i64,
    pub winner: i64,
    pub map_id: Option<i64>,
    pub map_slug: Option<String>,
    pub map_name: Option<String>,
    pub a_frags: Option<i16>,
    pub b_frags: Option<i16>,
    pub status: String,
    pub reported_side: Option<String>,
    pub reported_at: DateTime<Utc>,
    pub confirm_by: Option<DateTime<Utc>>,
    pub contested_at: Option<DateTime<Utc>>,
    pub contest_comment: Option<String>,
    pub confirmed_via: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

impl TryFrom<GameRow> for StoredGame {
    type Error = ApiError;

    fn try_from(r: GameRow) -> Result<Self, ApiError> {
        let map = match (r.map_id, r.map_slug, r.map_name) {
            (Some(id), Some(slug), Some(name)) => Some(GameMap {
                id: MapId(id),
                slug,
                name,
            }),
            _ => None,
        };
        Ok(Self {
            id: r.id,
            match_id: super::key::parse(&r.match_key)
                .ok_or_else(|| unknown("match_key", &r.match_key))?,
            number: r.number,
            side_a: ParticipantId(r.side_a),
            side_b: ParticipantId(r.side_b),
            winner: ParticipantId(r.winner),
            map,
            frags: r.a_frags.zip(r.b_frags),
            status: GameStatus::from_db(&r.status)?,
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
        })
    }
}

impl StoredGame {
    pub fn sides(&self) -> (ParticipantId, ParticipantId) {
        (self.side_a, self.side_b)
    }

    pub fn winner_side(&self) -> Side {
        if self.winner == self.side_a {
            Side::A
        } else {
            Side::B
        }
    }

    pub fn side_of(&self, p: ParticipantId) -> Option<Side> {
        if p == self.side_a {
            Some(Side::A)
        } else if p == self.side_b {
            Some(Side::B)
        } else {
            None
        }
    }

    pub fn series_game(&self) -> SeriesGame {
        SeriesGame {
            winner: self.winner_side().into(),
            status: self.status.into(),
            map: self.map.as_ref().map(|m| m.id),
        }
    }
}

/// Os mapas da série como o `engine` os vê, na ordem.
pub fn series_games(games: &[&StoredGame]) -> Vec<SeriesGame> {
    games.iter().map(|g| g.series_game()).collect()
}

/// Mapas confirmados vencidos por `a` e por `b`.
pub fn confirmed_wins(games: &[&StoredGame]) -> (u8, u8) {
    let won = |side: Side| {
        games
            .iter()
            .filter(|g| g.status == GameStatus::Confirmed && g.winner_side() == side)
            .count() as u8
    };
    (won(Side::A), won(Side::B))
}
