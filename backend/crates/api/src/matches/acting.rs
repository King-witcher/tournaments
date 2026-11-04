//! Quem pode agir pelo time: a regra do torneio (`TeamRules`) aplicada a quem
//! clica, num lugar só, para os handlers não espalharem `if`. O duelo não muda: o
//! capitão é o único jogador.
//!
//! Daqui também sai o que depende disso sem ser uma ação: quem precisa marcar
//! "Pronto" (`Stage::readiness`) e quem recebe os avisos que pedem ação
//! (`audience`, `veto_audience`). O que não pede ação continua indo para o time.

use engine::{
    core::{MatchId, ParticipantId},
    veto::MapMode,
};

use super::{
    result::Side,
    stage::Stage,
    veto::{Readiness, turn_action},
};
use crate::{
    shared::ApiError,
    teams::StoredTeam,
    tournaments::{TeamAction, TeamPicker, TeamRules},
};

/// 403: esta ação é só do capitão do time.
pub const NOT_CAPTAIN: ApiError = ApiError::forbidden("match.not_captain");

impl Stage {
    /// `user_id` é o capitão do time dele.
    pub fn is_captain(&self, user_id: i64) -> bool {
        self.entrants
            .iter()
            .any(|t| t.has(user_id) && t.captain == user_id)
    }

    /// `user_id`, que joga por um time da chave, pode fazer `action` pelo lado.
    pub fn may(&self, user_id: i64, action: TeamAction) -> bool {
        self.team_rules.allows(action, self.is_captain(user_id))
    }

    /// Quem marca "Pronto" pelo time `id` (ver `required`) e quem do lado já
    /// marcou, na série `match_id` entre `sides`. Conta sobre o elenco de agora:
    /// quem saiu do time ou da comunidade não trava o lado.
    pub fn readiness(&self, match_id: MatchId, sides: (ParticipantId, ParticipantId)) -> Readiness {
        Readiness::of(
            self.veto_of(match_id).map(|v| &v.log),
            [self.required(sides.0), self.required(sides.1)],
        )
    }

    /// Com tudo no capitão, ele; senão, todos os jogadores que estão no time e
    /// na comunidade.
    fn required(&self, id: ParticipantId) -> Vec<i64> {
        let Some(team) = self.entrant(id) else {
            return Vec::new();
        };
        if self.team_rules.captain_only() {
            vec![team.captain]
        } else {
            team.members
                .iter()
                .filter(|m| !m.left)
                .map(|m| m.user_id)
                .collect()
        }
    }
}

/// 403 `match.not_captain` se a regra de `action` não deixa `user_id` agir.
pub fn require(stage: &Stage, user_id: i64, action: TeamAction) -> Result<(), ApiError> {
    if stage.may(user_id, action) {
        Ok(())
    } else {
        Err(NOT_CAPTAIN)
    }
}

/// 403 `match.not_captain`: com tudo no capitão, só ele marca "Pronto".
pub fn require_ready(stage: &Stage, user_id: i64) -> Result<(), ApiError> {
    if !stage.team_rules.captain_only() || stage.is_captain(user_id) {
        Ok(())
    } else {
        Err(NOT_CAPTAIN)
    }
}

/// 403 `match.not_captain` se `user_id` não pode fazer a vez de agora, se há uma.
pub fn require_turn(
    stage: &Stage,
    user_id: i64,
    turn: &engine::veto::Turn,
) -> Result<(), ApiError> {
    match turn_action(turn) {
        Some(action) => require(stage, user_id, action),
        None => Ok(()),
    }
}

/// Quem do time recebe o aviso que pede `action`: o capitão, ou todos.
pub fn audience(team: &StoredTeam, rules: TeamRules, action: TeamAction) -> Vec<i64> {
    if rules.open_to_all(action) {
        team.user_ids()
    } else {
        vec![team.captain]
    }
}

/// Quem do time recebe o aviso de que o veto começou: só o capitão quando ele é o
/// único com algo a fazer em todo passo que o torneio tem; senão, todos.
pub fn veto_audience(stage: &Stage, team: &StoredTeam) -> Vec<i64> {
    let rules = stage.team_rules;
    let veto = &stage.veto_rules;
    let map_steps = matches!(veto.map_mode, MapMode::Veto | MapMode::Vote);
    let phases = [
        (map_steps, rules.open_to_all(TeamAction::Veto)),
        (
            veto.champion_phase && veto.bans() > 0,
            rules.open_to_all(TeamAction::Ban),
        ),
        (veto.champion_phase, rules.pick == TeamPicker::EachPlayer),
    ];
    let has_steps = phases.iter().any(|(on, _)| *on);
    let only_captain = has_steps && !phases.iter().any(|(on, open)| *on && *open);
    if only_captain {
        vec![team.captain]
    } else {
        team.user_ids()
    }
}

/// O lado de `side` entre `sides`.
pub const fn of_side(
    side: Side,
    (a, b): (ParticipantId, ParticipantId),
) -> (ParticipantId, ParticipantId) {
    match side {
        Side::A => (a, b),
        Side::B => (b, a),
    }
}
