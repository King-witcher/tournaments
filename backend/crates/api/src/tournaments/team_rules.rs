//! Quem age pelo time nas séries de um torneio em time: quatro regras, guardadas
//! no torneio, que mudam até ele começar. Só as regras ficam gravadas, nunca o
//! preset, que o front deriva delas: um preset novo não pede migration. No duelo
//! o capitão é o único jogador, e as regras não mudam nada. Quem confere a regra
//! em cada ação é `matches::acting`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::ApiError;

/// Quem faz o que é do lado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TeamActor {
    Captain,
    /// Qualquer jogador do time, valendo o primeiro clique.
    Anyone,
}

impl TeamActor {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Captain => "captain",
            Self::Anyone => "anyone",
        }
    }

    /// Lê as colunas `veto_by`, `ban_by` e `report_by` (garantidas pelo check da
    /// tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "captain" => Ok(Self::Captain),
            "anyone" => Ok(Self::Anyone),
            other => {
                tracing::error!(value = other, "regra de quem age desconhecida no banco");
                Err(ApiError::internal())
            }
        }
    }
}

/// Quem escolhe os campeões.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TeamPicker {
    /// O capitão escolhe por todos do time: ele diz para qual colega é o campeão.
    Captain,
    /// Cada jogador escolhe o próprio.
    EachPlayer,
}

impl TeamPicker {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Captain => "captain",
            Self::EachPlayer => "each_player",
        }
    }

    /// Lê `tournaments.pick_by` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "captain" => Ok(Self::Captain),
            "each_player" => Ok(Self::EachPlayer),
            other => {
                tracing::error!(value = other, "regra de quem escolhe desconhecida no banco");
                Err(ApiError::internal())
            }
        }
    }
}

/// O que uma regra cobre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamAction {
    /// Vetar mapas e, na votação, votar.
    Veto,
    Ban,
    Pick,
    /// Enviar, confirmar e contestar um resultado.
    Report,
}

/// As quatro regras de um torneio em time.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamRules {
    /// Vetar mapas e, no modo votação, votar.
    pub veto: TeamActor,
    /// Banir campeões.
    pub ban: TeamActor,
    /// Escolher campeões.
    pub pick: TeamPicker,
    /// Enviar, confirmar e contestar o resultado.
    pub report: TeamActor,
}

impl TeamRules {
    /// Todos os jogadores: o comportamento de quem já existia, e o padrão das
    /// colunas.
    pub const EVERYONE: Self = Self {
        veto: TeamActor::Anyone,
        ban: TeamActor::Anyone,
        pick: TeamPicker::EachPlayer,
        report: TeamActor::Anyone,
    };

    /// Tudo no capitão: o preset e a sala centrada nele.
    pub const CAPTAIN: Self = Self {
        veto: TeamActor::Captain,
        ban: TeamActor::Captain,
        pick: TeamPicker::Captain,
        report: TeamActor::Captain,
    };

    /// O capitão veta e bane, cada jogador escolhe o próprio campeão e
    /// qualquer um envia: o preset Balanceado e o padrão de um torneio novo.
    pub const BALANCED: Self = Self {
        veto: TeamActor::Captain,
        ban: TeamActor::Captain,
        pick: TeamPicker::EachPlayer,
        report: TeamActor::Anyone,
    };

    /// `action` é de qualquer um do time, não só do capitão.
    pub const fn open_to_all(self, action: TeamAction) -> bool {
        match action {
            TeamAction::Veto => matches!(self.veto, TeamActor::Anyone),
            TeamAction::Ban => matches!(self.ban, TeamActor::Anyone),
            TeamAction::Pick => matches!(self.pick, TeamPicker::EachPlayer),
            TeamAction::Report => matches!(self.report, TeamActor::Anyone),
        }
    }

    /// Um jogador do time pode fazer `action`, sendo o capitão ou não.
    pub const fn allows(self, action: TeamAction, captain: bool) -> bool {
        captain || self.open_to_all(action)
    }
}

impl Default for TeamRules {
    fn default() -> Self {
        Self::EVERYONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_captain_can_do_everything_and_the_others_what_is_open() {
        let rules = TeamRules {
            ban: TeamActor::Captain,
            ..TeamRules::EVERYONE
        };
        for action in [
            TeamAction::Veto,
            TeamAction::Ban,
            TeamAction::Pick,
            TeamAction::Report,
        ] {
            assert!(rules.allows(action, true));
            assert_eq!(rules.allows(action, false), action != TeamAction::Ban);
        }
    }
}
