//! Modalidade, vagas, regra de campeões e fases: o que define como o torneio
//! é jogado, só muda antes de ele começar e passa pela checagem do `engine`.

use std::ops::RangeInclusive;

use engine::{champions as ec, stages::StageSetup, tournament::TournamentSetup};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    team_rules::TeamRules,
    types::{ChampionRules, StageFormat},
    veto::VetoSetup,
};
use crate::shared::ApiError;

/// Vagas aceitas (no TDM, o mínimo é o de dois times).
const PARTICIPANTS: RangeInclusive<i32> = 2..=1024;
/// Bans de campeão de cada lado na série.
const MAX_BANS_PER_SERIES: u32 = engine::champions::MAX_BANS_PER_SERIES;
/// Fases por torneio (o check `tournament_stages_position_max` do banco).
pub const MAX_STAGES: usize = 4;
/// Jogadores por time no TDM.
const TDM_TEAM_SIZES: RangeInclusive<i32> = 2..=4;
/// Times na chave, no mínimo.
const MIN_SIDES: i32 = 2;

/// As fases do corpo nos tipos do `engine`. Erro: 422
/// `format.invalid_tiebreakers` `{stage}` com desempate repetido.
pub fn stages_from_dto(stages: Vec<StageFormat>) -> Result<Vec<StageSetup>, ApiError> {
    (1..)
        .zip(stages)
        .map(|(number, s)| StageSetup::try_from(s).map_err(|e| e.in_stage(number).into()))
        .collect()
}

/// A modalidade: duelo (um contra um) ou TDM (times de 2 a 4, `team_size`).
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TournamentMode {
    Duel,
    Tdm,
}

impl TournamentMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Duel => "duel",
            Self::Tdm => "tdm",
        }
    }

    /// Lê `tournaments.mode` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "duel" => Ok(Self::Duel),
            "tdm" => Ok(Self::Tdm),
            other => {
                tracing::error!(mode = other, "modalidade desconhecida no banco");
                Err(ApiError::internal())
            }
        }
    }

    /// Jogadores por time aceitos.
    pub const fn team_sizes(self) -> RangeInclusive<i32> {
        match self {
            Self::Duel => 1..=1,
            Self::Tdm => TDM_TEAM_SIZES,
        }
    }

    /// O tamanho quando a criação (ou a troca de modo) não diz: 1 no duelo,
    /// 2 no TDM.
    pub const fn default_team_size(self) -> i32 {
        match self {
            Self::Duel => 1,
            Self::Tdm => 2,
        }
    }
}

/// Modalidade, vagas, regra de campeões, fases e veto, nos tipos do `engine`
/// (os que ficam gravados).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setup {
    pub mode: TournamentMode,
    /// Jogadores por time: 1 no duelo, de 2 a 4 no TDM.
    pub team_size: i32,
    /// Vagas, em jogadores (a inscrição é individual).
    pub max_participants: i32,
    pub champion_rules: ec::ChampionRules,
    /// As fases, na ordem: o formato, o corte e o check-in de cada uma.
    pub stages: Vec<StageSetup>,
    pub veto: VetoSetup,
    /// Quem age pelo time.
    pub team_rules: TeamRules,
}

impl Setup {
    /// Erro: 422 `format.invalid_tiebreakers` com desempate repetido; o resto
    /// fica para `check`.
    pub fn from_dto(
        mode: TournamentMode,
        team_size: Option<i32>,
        max_participants: i32,
        rules: ChampionRules,
        stages: Vec<StageFormat>,
        veto: VetoSetup,
        team_rules: TeamRules,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            mode,
            team_size: team_size.unwrap_or(mode.default_team_size()),
            max_participants,
            champion_rules: rules.into(),
            stages: stages_from_dto(stages)?,
            veto,
            team_rules,
        })
    }

    /// Jogadores por time, para o `engine` (validado em `check`).
    pub fn team_size(&self) -> u32 {
        u32::try_from(self.team_size).unwrap_or(1)
    }

    /// É TDM (times de mais de um).
    pub fn teams(&self) -> bool {
        self.mode == TournamentMode::Tdm
    }

    /// A série mais longa das fases (para o pool de mapas).
    pub fn longest_series(&self) -> u8 {
        self.stages
            .iter()
            .map(|s| s.format.longest_series())
            .max()
            .unwrap_or(1)
    }

    /// As regras do veto das séries da fase `number`: um lugar só, para as
    /// regras de mapas e campeões por fase entrarem aqui. Hoje, as do torneio
    /// em todas as fases.
    pub fn stage_veto_rules(&self, _number: i16) -> engine::veto::VetoRules {
        self.veto_rules()
    }

    /// As regras do veto das séries do torneio.
    pub fn veto_rules(&self) -> engine::veto::VetoRules {
        self.veto.rules(self.champion_rules, self.team_size())
    }

    /// Limites do site (tamanho do time, vagas, bans, número de fases,
    /// veto), vagas que caibam os `registered` já inscritos e depois a
    /// checagem do `engine`: formato válido e pool de campeões suficiente no
    /// pior caso, com os times que as vagas comportam.
    pub fn check(&self, champion_pool: u32, registered: i64) -> Result<(), ApiError> {
        let sizes = self.mode.team_sizes();
        if !sizes.contains(&self.team_size) {
            return Err(
                ApiError::invalid("tournament.team_size_invalid").with_params([
                    ("min", i64::from(*sizes.start())),
                    ("max", i64::from(*sizes.end())),
                ]),
            );
        }
        let min = (MIN_SIDES * self.team_size).max(*PARTICIPANTS.start());
        if self.max_participants < min || self.max_participants > *PARTICIPANTS.end() {
            return Err(
                ApiError::invalid("tournament.max_participants_invalid").with_params([
                    ("min", i64::from(min)),
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
        if self.champion_rules.bans_per_series > MAX_BANS_PER_SERIES {
            return Err(ApiError::invalid("tournament.bans_invalid")
                .with_params([("max", i64::from(MAX_BANS_PER_SERIES))]));
        }
        if self.stages.is_empty() || self.stages.len() > MAX_STAGES {
            return Err(ApiError::invalid("tournament.stage_count_invalid")
                .with_params([("max", MAX_STAGES as i64)]));
        }
        self.veto.check(self.longest_series())?;
        let slots = usize::try_from(self.max_participants).unwrap_or(0);
        self.check_entrants(champion_pool, slots)
            .map_err(|e| self.in_players(e))
    }

    /// O teto do formato vem do `engine` em lados (times); nas vagas, que
    /// contam jogadores, ele vale vezes o tamanho do time.
    fn in_players(&self, mut err: ApiError) -> ApiError {
        if err.code == "format.too_many_participants" {
            let size = i64::from(self.team_size.max(1));
            err.params = err
                .params
                .into_iter()
                .map(|(name, value)| match name {
                    "max" => (name, value * size),
                    "got" => (name, i64::from(self.max_participants)),
                    _ => (name, value),
                })
                .collect();
        }
        err
    }

    /// A checagem do `engine` com `players` jogadores: as vagas na criação e
    /// na edição; ao iniciar, os dos times que entram na chave, todos
    /// completos (com o pool de campeões de agora, que pode ter mudado desde
    /// a criação). Os lados são os times que eles formam, `⌊players/tamanho⌋`.
    /// Às cegas exige espelho: 422 `tournament.blind_needs_mirror`.
    pub fn check_entrants(&self, champion_pool: u32, players: usize) -> Result<(), ApiError> {
        engine::veto::PickOrder::from(self.veto.pick_order).check(self.champion_rules.mirror)?;
        self.engine(champion_pool, players).validate()?;
        Ok(())
    }

    /// Ao iniciar a fase `number` (2 em diante) com `sides` times de verdade:
    /// o formato com esse número e o pool de campeões de agora, só dessa fase
    /// (as fases são isoladas na regra de não repetição). Erros: os de
    /// `check_entrants` para a fase (com `params.stage`).
    pub fn check_stage(
        &self,
        champion_pool: u32,
        number: i16,
        sides: usize,
    ) -> Result<(), ApiError> {
        engine::veto::PickOrder::from(self.veto.pick_order).check(self.champion_rules.mirror)?;
        let index = usize::try_from(number - 1).map_err(|_| ApiError::internal())?;
        let players = sides * self.team_size() as usize;
        Ok(self
            .engine(champion_pool, players)
            .validate_stage(index, sides)?)
    }

    fn engine(&self, champion_pool: u32, players: usize) -> TournamentSetup<'_> {
        TournamentSetup {
            team_size: self.team_size(),
            slots: players,
            champions: self.champion_rules,
            champion_pool,
            stages: &self.stages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tournaments::types::{Bracket, FinalTie, NoRepeatRule, NoRepeatScope, NoRepeatWho};

    fn double(best_of: u8) -> StageFormat {
        StageFormat::Elimination {
            bracket: Bracket::Double,
            grand_final_reset: true,
            best_of,
            finals_best_of: 5,
            advance: None,
            check_in: false,
        }
    }

    fn placement(best_of: u8) -> StageFormat {
        StageFormat::Placement {
            best_of,
            advance: None,
            check_in: false,
        }
    }

    fn setup(
        slots: i32,
        bans: u8,
        scope: Option<NoRepeatScope>,
        stages: Vec<StageFormat>,
    ) -> Setup {
        let rules = ChampionRules {
            bans_per_series: bans,
            mirror: true,
            same_team: false,
            no_repeat: scope.map(|scope| NoRepeatRule {
                who: NoRepeatWho::Player,
                scope,
            }),
        };
        Setup::from_dto(
            TournamentMode::Duel,
            None,
            slots,
            rules,
            stages,
            VetoSetup::off(),
            TeamRules::default(),
        )
        .unwrap()
    }

    fn tdm(size: i32, slots: i32) -> Setup {
        Setup {
            mode: TournamentMode::Tdm,
            team_size: size,
            ..setup(slots, 1, None, vec![double(3)])
        }
    }

    type Params = Vec<(&'static str, i64)>;

    fn check(s: Setup) -> Result<(), (&'static str, Params)> {
        s.check(17, 0).map_err(|e| (e.code, e.params))
    }

    #[test]
    fn slots_fit_who_is_registered() {
        let s = || setup(8, 1, None, vec![double(3)]);
        assert!(s().check(17, 8).is_ok());
        let err = s().check(17, 9).unwrap_err();
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
            err("tournament.stage_count_invalid", &[("max", 4)])
        );
        // Até 4 fases (a eliminação só na última).
        let placements = |n| (0..n).map(|_| placement(3)).collect::<Vec<_>>();
        assert_eq!(check(setup(8, 1, None, placements(4))), Ok(()));
        assert_eq!(
            check(setup(8, 1, None, placements(5))),
            err("tournament.stage_count_invalid", &[("max", 4)])
        );
    }

    #[test]
    fn stage_rules_carry_the_stage() {
        let err = |code, params: &[(&'static str, i64)]| Err((code, params.to_vec()));
        assert_eq!(
            check(setup(8, 1, None, vec![double(3), placement(3)])),
            err("stage.elimination_not_last", &[("stage", 1)])
        );
        assert_eq!(
            check(setup(8, 1, None, vec![placement(3), double(4)])),
            err("format.invalid_best_of", &[("best_of", 4), ("stage", 2)])
        );
        let cut = |advance| StageFormat::Placement {
            best_of: 3,
            advance,
            check_in: false,
        };
        assert_eq!(
            check(setup(8, 1, None, vec![placement(3), cut(Some(1))])),
            err(
                "stage.invalid_advance",
                &[("stage", 2), ("min", 2), ("max", 128)]
            )
        );
        assert_eq!(
            check(setup(8, 1, None, vec![cut(Some(4)), placement(3)])),
            err("stage.entry_rules_first", &[("stage", 1)])
        );
    }

    #[test]
    fn team_size_and_slots_follow_the_mode() {
        let err = |code, params: &[(&'static str, i64)]| Err((code, params.to_vec()));
        assert_eq!(check(tdm(2, 4)), Ok(()));
        assert_eq!(
            check(tdm(5, 16)),
            err("tournament.team_size_invalid", &[("min", 2), ("max", 4)])
        );
        assert_eq!(
            check(tdm(1, 16)),
            err("tournament.team_size_invalid", &[("min", 2), ("max", 4)])
        );
        let duel = Setup {
            team_size: 2,
            ..setup(8, 1, None, vec![double(3)])
        };
        assert_eq!(
            check(duel),
            err("tournament.team_size_invalid", &[("min", 1), ("max", 1)])
        );
        // Vagas para dois times, no mínimo.
        assert_eq!(
            check(tdm(3, 5)),
            err(
                "tournament.max_participants_invalid",
                &[("min", 6), ("max", 1024)]
            )
        );
    }

    #[test]
    fn the_format_cap_counts_players_in_teams() {
        let rr = |slots| Setup {
            stages: stages_from_dto(vec![StageFormat::RoundRobin {
                best_of: 3,
                tiebreakers: vec![],
                final_tie: FinalTie::Seed,
                advance: None,
                check_in: false,
            }])
            .unwrap(),
            ..tdm(2, slots)
        };
        // 32 times de 2 no round robin.
        assert_eq!(check(rr(64)), Ok(()));
        assert_eq!(
            check(rr(66)),
            Err((
                "format.too_many_participants",
                vec![("max", 64), ("got", 66), ("stage", 1)]
            ))
        );
    }

    #[test]
    fn the_engine_counts_players_below_two_teams() {
        // A checagem do site recusa antes (`max_participants_invalid`); a do
        // `engine`, chamada com os jogadores, fala em jogadores, não em lados.
        let err = tdm(4, 16).check_entrants(17, 7).unwrap_err();
        assert_eq!(
            (err.code, err.params),
            ("format.not_enough_players", vec![("min", 8), ("got", 7)])
        );
    }

    #[test]
    fn blind_needs_a_mirror_at_every_check() {
        use crate::tournaments::veto::PickOrder;
        let blind = |mirror: bool| {
            let mut s = setup(8, 1, None, vec![double(3)]);
            s.veto.pick_order = PickOrder::Blind;
            s.champion_rules.mirror = mirror;
            s
        };
        assert_eq!(check(blind(true)), Ok(()));
        assert_eq!(
            check(blind(false)),
            Err(("tournament.blind_needs_mirror", vec![]))
        );
        // Ao iniciar, é a checagem dos que entram que roda.
        let err = blind(false).check_entrants(17, 8).unwrap_err();
        assert_eq!(err.code, "tournament.blind_needs_mirror");
        // Em qualquer tamanho, e sem espelho as outras ordens continuam.
        let mut team = blind(false);
        team.mode = TournamentMode::Tdm;
        team.team_size = 2;
        assert_eq!(
            check(team.clone()),
            Err(("tournament.blind_needs_mirror", vec![]))
        );
        team.veto.pick_order = PickOrder::Alternating;
        assert_eq!(check(team), Ok(()));
    }

    #[test]
    fn engine_errors_keep_their_numbers() {
        assert_eq!(
            check(setup(8, 1, None, vec![double(4)])),
            Err(("format.invalid_best_of", vec![("best_of", 4), ("stage", 1)]))
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
            check(setup(16, 1, Some(NoRepeatScope::Series), vec![double(3)])),
            Ok(())
        );
    }
}
