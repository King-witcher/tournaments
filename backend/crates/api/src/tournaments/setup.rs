//! Modalidade, vagas, regra de campeões e fases: o que define como o torneio
//! é jogado, só muda antes de ele começar e passa pela checagem do `engine`.

use std::ops::RangeInclusive;

use engine::{champions as ec, formats as ef, tournament::TournamentSetup};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    types::{ChampionRules, StageFormat},
    veto::VetoSetup,
};
use crate::shared::ApiError;

/// Vagas aceitas.
const PARTICIPANTS: RangeInclusive<i32> = 2..=1024;
/// Bans de campeão por mapa.
const MAX_BANS_PER_GAME: u32 = 3;
/// Fases por torneio. Ainda não há regra de classificação entre fases nem
/// interface para mais de uma.
const MAX_STAGES: usize = 1;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TournamentMode {
    Duel,
}

impl TournamentMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Duel => "duel",
        }
    }

    /// Lê `tournaments.mode` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "duel" => Ok(Self::Duel),
            other => {
                tracing::error!(mode = other, "modalidade desconhecida no banco");
                Err(ApiError::internal())
            }
        }
    }

    /// Jogadores por lado.
    pub const fn team_size(self) -> u32 {
        match self {
            Self::Duel => 1,
        }
    }
}

/// Vagas, regra de campeões e fases, nos tipos do `engine` (os que ficam
/// gravados).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setup {
    pub max_participants: i32,
    pub champion_rules: ec::ChampionRules,
    pub stages: Vec<ef::StageFormat>,
    pub veto: VetoSetup,
}

impl Setup {
    pub fn from_dto(
        max_participants: i32,
        rules: ChampionRules,
        stages: Vec<StageFormat>,
        veto: VetoSetup,
    ) -> Self {
        Self {
            max_participants,
            champion_rules: rules.into(),
            stages: stages.into_iter().map(Into::into).collect(),
            veto,
        }
    }

    /// A série mais longa das fases (para o pool de mapas).
    pub fn longest_series(&self) -> u8 {
        self.stages
            .iter()
            .map(ef::StageFormat::longest_series)
            .max()
            .unwrap_or(1)
    }

    /// Limites do site (vagas, bans, número de fases, veto), vagas que caibam
    /// os `registered` já inscritos e depois a checagem do `engine`: formato
    /// válido e pool de campeões suficiente no pior caso.
    pub fn check(
        &self,
        mode: TournamentMode,
        champion_pool: u32,
        registered: i64,
    ) -> Result<(), ApiError> {
        if !PARTICIPANTS.contains(&self.max_participants) {
            return Err(
                ApiError::invalid("tournament.max_participants_invalid").with_params([
                    ("min", i64::from(*PARTICIPANTS.start())),
                    ("max", i64::from(*PARTICIPANTS.end())),
                ]),
            );
        }
        if i64::from(self.max_participants) < registered {
            return Err(
                ApiError::conflict("tournament.max_participants_below_registered")
                    .with_params([("registered", registered)]),
            );
        }
        if self.champion_rules.bans_per_game > MAX_BANS_PER_GAME {
            return Err(ApiError::invalid("tournament.bans_invalid")
                .with_params([("max", i64::from(MAX_BANS_PER_GAME))]));
        }
        if self.stages.is_empty() || self.stages.len() > MAX_STAGES {
            return Err(ApiError::invalid("tournament.stage_count_invalid")
                .with_params([("max", MAX_STAGES as i64)]));
        }
        self.veto.check(self.longest_series())?;
        self.check_entrants(mode, champion_pool, self.max_participants as usize)
    }

    /// A checagem do `engine` com `entrants` participantes, que ao iniciar são
    /// os que entram na chave (com o pool de campeões de agora, que pode ter
    /// mudado desde a criação).
    pub fn check_entrants(
        &self,
        mode: TournamentMode,
        champion_pool: u32,
        entrants: usize,
    ) -> Result<(), ApiError> {
        TournamentSetup {
            team_size: mode.team_size(),
            max_participants: entrants,
            champions: self.champion_rules,
            champion_pool,
            stages: &self.stages,
        }
        .validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tournaments::types::{Bracket, NoRepeatRule, NoRepeatScope, NoRepeatWho};

    fn double(best_of: u8) -> StageFormat {
        StageFormat::Elimination {
            bracket: Bracket::Double,
            grand_final_reset: true,
            best_of,
            finals_best_of: 5,
        }
    }

    fn setup(
        slots: i32,
        bans: u8,
        scope: Option<NoRepeatScope>,
        stages: Vec<StageFormat>,
    ) -> Setup {
        let rules = ChampionRules {
            bans_per_game: bans,
            no_repeat: scope.map(|scope| NoRepeatRule {
                who: NoRepeatWho::Player,
                scope,
            }),
        };
        Setup::from_dto(slots, rules, stages, VetoSetup::off())
    }

    type Params = Vec<(&'static str, i64)>;

    fn check(s: Setup) -> Result<(), (&'static str, Params)> {
        s.check(TournamentMode::Duel, 17, 0)
            .map_err(|e| (e.code, e.params))
    }

    #[test]
    fn slots_fit_who_is_registered() {
        let s = || setup(8, 1, None, vec![double(3)]);
        assert!(s().check(TournamentMode::Duel, 17, 8).is_ok());
        let err = s().check(TournamentMode::Duel, 17, 9).unwrap_err();
        assert_eq!(
            (err.code, err.params),
            (
                "tournament.max_participants_below_registered",
                vec![("registered", 9)]
            )
        );
    }

    #[test]
    fn site_limits_come_first() {
        let err = |code, params: &[(&'static str, i64)]| Err((code, params.to_vec()));
        assert_eq!(
            check(setup(1, 1, None, vec![double(3)])),
            err(
                "tournament.max_participants_invalid",
                &[("min", 2), ("max", 1024)]
            )
        );
        assert_eq!(
            check(setup(1025, 1, None, vec![double(3)])),
            err(
                "tournament.max_participants_invalid",
                &[("min", 2), ("max", 1024)]
            )
        );
        assert_eq!(
            check(setup(8, 4, None, vec![double(3)])),
            err("tournament.bans_invalid", &[("max", 3)])
        );
        assert_eq!(
            check(setup(8, 1, None, vec![])),
            err("tournament.stage_count_invalid", &[("max", 1)])
        );
        assert_eq!(
            check(setup(8, 1, None, vec![double(3), double(3)])),
            err("tournament.stage_count_invalid", &[("max", 1)])
        );
    }

    #[test]
    fn engine_errors_keep_their_numbers() {
        assert_eq!(
            check(setup(8, 1, None, vec![double(4)])),
            Err(("format.invalid_best_of", vec![("best_of", 4)]))
        );
        assert_eq!(
            check(setup(
                16,
                1,
                Some(NoRepeatScope::Tournament),
                vec![double(3)]
            )),
            Err((
                "champions.pool_exhausted",
                vec![("required", 33), ("available", 17)]
            ))
        );
        assert_eq!(
            check(setup(16, 1, Some(NoRepeatScope::Match), vec![double(3)])),
            Ok(())
        );
    }
}
