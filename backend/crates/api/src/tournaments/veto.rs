//! A configuração do veto das partidas: como sai o mapa de cada jogo (veto,
//! votação, sorteio ou desligado), o pool de mapas do torneio, o ban/pick de
//! campeões e o tempo de cada passo. Faz parte da configuração (`Setup`):
//! muda até o torneio começar. O veto de cada partida fica em `matches`.

use std::{collections::HashMap, ops::RangeInclusive};

use engine::{core::MapId, veto as ev};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;

use crate::{maps, shared::ApiError};

/// Tempo de cada passo quando a criação não diz.
pub const DEFAULT_STEP_SECONDS: i32 = 30;
/// Tempo aceito, em segundos (o mesmo check está na tabela).
const STEP_SECONDS: RangeInclusive<i32> = 10..=300;
/// Janela da escolha simultânea de campeões do TDM quando a criação não diz.
pub const DEFAULT_TEAM_PICK_SECONDS: i32 = 60;
/// Janela aceita, em segundos (o mesmo check está na tabela).
const TEAM_PICK_SECONDS: RangeInclusive<i32> = 15..=300;
/// Teto do pool (a tabela aceita até 64 posições).
const MAX_POOL: usize = 32;

/// Como sai o mapa de cada jogo da série.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MapMode {
    /// Ban/pick alternado da série inteira antes do primeiro jogo, com o
    /// decisivo.
    Veto,
    /// A cada jogo, 3 mapas sorteados do pool (sem os já jogados); cada um
    /// vota; votos diferentes, sorteio entre os dois.
    Vote,
    /// O mapa de cada jogo é sorteado quando ele começa.
    Random,
    /// Os jogadores combinam fora e enviam o mapa.
    Off,
}

impl MapMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Veto => "veto",
            Self::Vote => "vote",
            Self::Random => "random",
            Self::Off => "off",
        }
    }

    /// Lê `tournaments.map_mode` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        [Self::Veto, Self::Vote, Self::Random, Self::Off]
            .into_iter()
            .find(|m| m.as_str() == value)
            .ok_or_else(|| {
                tracing::error!(map_mode = value, "modo de mapa desconhecido no banco");
                ApiError::internal()
            })
    }
}

impl From<MapMode> for ev::MapMode {
    fn from(mode: MapMode) -> Self {
        match mode {
            MapMode::Veto => Self::Veto,
            MapMode::Vote => Self::Vote,
            MapMode::Random => Self::Random,
            MapMode::Off => Self::Off,
        }
    }
}

/// A configuração do veto que chega: o que vier no lugar do atual (na
/// criação, do padrão).
#[derive(Deserialize, ToSchema, Default)]
pub struct VetoInput {
    /// Padrão `veto`.
    #[serde(default)]
    pub map_mode: Option<MapMode>,
    /// Slugs dos mapas (`GET /maps`, ativos), na ordem, sem repetir; fora do
    /// modo `off`, pelo menos um por jogo da série mais longa. Padrão: os 7
    /// primeiros do pool de duelo. Vale para o veto, a votação e o sorteio.
    #[serde(default)]
    pub map_pool: Option<Vec<String>>,
    /// Ban/pick de campeões a cada mapa, com os bans de `champion_rules`.
    /// Padrão ligado; vale com qualquer modo de mapa.
    #[serde(default)]
    pub champion_phase: Option<bool>,
    /// Segundos de cada passo, de 10 a 300 (padrão 30); no fim, a escolha é
    /// sorteada.
    #[serde(default)]
    pub step_seconds: Option<i32>,
    /// TDM: segundos da escolha simultânea de campeões (todos do mapa numa
    /// janela só, depois dos bans), de 15 a 300 (padrão 60); no fim, quem
    /// não escolheu recebe um sorteado.
    #[serde(default)]
    pub team_pick_seconds: Option<i32>,
}

/// A configuração do veto como fica gravada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VetoSetup {
    pub map_mode: MapMode,
    /// O pool, na ordem.
    pub pool: Vec<MapId>,
    pub champion_phase: bool,
    pub step_seconds: i32,
    /// TDM: segundos da escolha simultânea de campeões.
    pub team_pick_seconds: i32,
}

/// A configuração do veto na resposta.
#[derive(Serialize, ToSchema)]
pub struct VetoSettings {
    pub map_mode: MapMode,
    /// Slugs do pool, na ordem.
    pub map_pool: Vec<String>,
    pub champion_phase: bool,
    pub step_seconds: i32,
    /// TDM: segundos da escolha simultânea de campeões.
    pub team_pick_seconds: i32,
}

/// Um mapa do pool do torneio.
#[derive(Debug, Clone)]
pub struct PoolMap {
    pub id: MapId,
    pub slug: String,
    pub name: String,
}

impl VetoSetup {
    /// Sem veto: mapa combinado fora e sem fase de campeões.
    #[cfg(test)]
    pub fn off() -> Self {
        Self {
            map_mode: MapMode::Off,
            pool: Vec::new(),
            champion_phase: false,
            step_seconds: DEFAULT_STEP_SECONDS,
            team_pick_seconds: DEFAULT_TEAM_PICK_SECONDS,
        }
    }

    /// As regras do `engine` para as séries do torneio (`Setup::veto_rules`).
    pub fn rules(
        &self,
        bans_per_game: u32,
        team_size: u32,
        team_duplicates: bool,
    ) -> ev::VetoRules {
        ev::VetoRules {
            map_mode: self.map_mode.into(),
            pool: self.pool.clone(),
            champion_phase: self.champion_phase,
            champion_bans: u8::try_from(bans_per_game).unwrap_or(u8::MAX),
            team_size: u8::try_from(team_size).unwrap_or(1),
            team_duplicates,
        }
    }

    /// Tempo do passo e da escolha simultânea, pool sem mapa repetido e com
    /// mapas para a série mais longa (`longest`). Erros: 422
    /// `tournament.veto_seconds_invalid` `{min, max}`,
    /// `tournament.team_pick_seconds_invalid` `{min, max}`,
    /// `tournament.map_pool_repeated`, `tournament.map_pool_too_small`
    /// `{min}`, `tournament.map_pool_too_large` `{max}`.
    pub fn check(&self, longest: u8) -> Result<(), ApiError> {
        if !STEP_SECONDS.contains(&self.step_seconds) {
            return Err(
                ApiError::invalid("tournament.veto_seconds_invalid").with_params([
                    ("min", i64::from(*STEP_SECONDS.start())),
                    ("max", i64::from(*STEP_SECONDS.end())),
                ]),
            );
        }
        if !TEAM_PICK_SECONDS.contains(&self.team_pick_seconds) {
            return Err(
                ApiError::invalid("tournament.team_pick_seconds_invalid").with_params([
                    ("min", i64::from(*TEAM_PICK_SECONDS.start())),
                    ("max", i64::from(*TEAM_PICK_SECONDS.end())),
                ]),
            );
        }
        if self.pool.len() > MAX_POOL {
            return Err(ApiError::invalid("tournament.map_pool_too_large")
                .with_params([("max", MAX_POOL as i64)]));
        }
        match self.rules(0, 1, false).validate(longest) {
            Ok(()) => Ok(()),
            Err(ev::VetoError::PoolRepeated(_)) => {
                Err(ApiError::invalid("tournament.map_pool_repeated"))
            }
            Err(_) => Err(ApiError::invalid("tournament.map_pool_too_small")
                .with_params([("min", i64::from(longest))])),
        }
    }
}

/// O que `input` deixa, sobre `current` (na criação, `None`: o padrão). Os
/// slugs viram ids dos mapas ativos. Erro: 422 `tournament.map_unknown`.
pub async fn resolve(
    conn: &mut PgConnection,
    input: VetoInput,
    current: Option<&VetoSetup>,
) -> Result<VetoSetup, ApiError> {
    let pool = match (input.map_pool, current) {
        (Some(slugs), _) => {
            let ids: HashMap<String, MapId> = maps::active_ids(conn, &slugs).await?;
            slugs
                .iter()
                .map(|s| ids.get(s).copied())
                .collect::<Option<Vec<MapId>>>()
                .ok_or(ApiError::invalid("tournament.map_unknown"))?
        }
        (None, Some(current)) => current.pool.clone(),
        (None, None) => maps::default_pool(conn).await?,
    };
    Ok(VetoSetup {
        map_mode: input
            .map_mode
            .or(current.map(|c| c.map_mode))
            .unwrap_or(MapMode::Veto),
        pool,
        champion_phase: input
            .champion_phase
            .or(current.map(|c| c.champion_phase))
            .unwrap_or(true),
        step_seconds: input
            .step_seconds
            .or(current.map(|c| c.step_seconds))
            .unwrap_or(DEFAULT_STEP_SECONDS),
        team_pick_seconds: input
            .team_pick_seconds
            .or(current.map(|c| c.team_pick_seconds))
            .unwrap_or(DEFAULT_TEAM_PICK_SECONDS),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(mode: MapMode, maps: i64, seconds: i32) -> VetoSetup {
        VetoSetup {
            map_mode: mode,
            pool: (1..=maps).map(MapId).collect(),
            champion_phase: true,
            step_seconds: seconds,
            team_pick_seconds: DEFAULT_TEAM_PICK_SECONDS,
        }
    }

    fn code(r: Result<(), ApiError>) -> (&'static str, Vec<(&'static str, i64)>) {
        r.map_or_else(|e| (e.code, e.params), |()| ("ok", vec![]))
    }

    #[test]
    fn limits() {
        assert_eq!(code(setup(MapMode::Veto, 7, 30).check(5)).0, "ok");
        assert_eq!(
            code(setup(MapMode::Veto, 7, 9).check(5)),
            (
                "tournament.veto_seconds_invalid",
                vec![("min", 10), ("max", 300)]
            )
        );
        let mut slow = setup(MapMode::Veto, 7, 30);
        slow.team_pick_seconds = 301;
        assert_eq!(
            code(slow.check(5)),
            (
                "tournament.team_pick_seconds_invalid",
                vec![("min", 15), ("max", 300)]
            )
        );
        assert_eq!(
            code(setup(MapMode::Vote, 4, 30).check(5)),
            ("tournament.map_pool_too_small", vec![("min", 5)])
        );
        // Desligado não precisa de pool.
        assert_eq!(code(setup(MapMode::Off, 0, 30).check(5)).0, "ok");
        let mut repeated = setup(MapMode::Veto, 7, 30);
        repeated.pool.push(MapId(1));
        assert_eq!(code(repeated.check(3)).0, "tournament.map_pool_repeated");
        assert_eq!(
            code(setup(MapMode::Veto, 33, 30).check(3)),
            ("tournament.map_pool_too_large", vec![("max", 32)])
        );
    }
}
