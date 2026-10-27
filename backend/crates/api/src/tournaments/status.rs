//! Ciclo de vida do torneio e o que cada fase dele ainda deixa mudar.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::ApiError;

/// rascunho → inscrição → check-in → em andamento → encerrado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TournamentStatus {
    /// Só organizer e owner veem. Único status em que dá para excluir.
    Draft,
    Registration,
    CheckIn,
    InProgress,
    Finished,
}

impl TournamentStatus {
    /// Lê `tournaments.status` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "draft" => Ok(Self::Draft),
            "registration" => Ok(Self::Registration),
            "check_in" => Ok(Self::CheckIn),
            "in_progress" => Ok(Self::InProgress),
            "finished" => Ok(Self::Finished),
            other => {
                tracing::error!(status = other, "status de torneio desconhecido no banco");
                Err(ApiError::internal())
            }
        }
    }

    /// Vagas, regra de campeões e fases só mudam antes de o torneio começar.
    pub const fn setup_editable(self) -> bool {
        matches!(self, Self::Draft | Self::Registration | Self::CheckIn)
    }

    /// Textos, traduções e visibilidade mudam até o torneio encerrar.
    pub const fn texts_editable(self) -> bool {
        !matches!(self, Self::Finished)
    }

    /// O próprio membro se inscreve, sai ou entra por convite só aqui.
    pub const fn registration_open(self) -> bool {
        matches!(self, Self::Registration)
    }

    /// O organizador inscreve e tira participantes até o torneio começar.
    pub const fn participants_editable(self) -> bool {
        matches!(self, Self::Draft | Self::Registration | Self::CheckIn)
    }

    /// Gerar link de inscrição: já no rascunho (para mandar antes de
    /// publicar) e com a inscrição aberta.
    pub const fn invites_allowed(self) -> bool {
        matches!(self, Self::Draft | Self::Registration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_each_status_allows() {
        use TournamentStatus::*;
        let all = [Draft, Registration, CheckIn, InProgress, Finished];
        let allows = |rule: fn(TournamentStatus) -> bool| all.map(rule);
        assert_eq!(
            allows(TournamentStatus::setup_editable),
            [true, true, true, false, false]
        );
        assert_eq!(
            allows(TournamentStatus::texts_editable),
            [true, true, true, true, false]
        );
        assert_eq!(
            allows(TournamentStatus::registration_open),
            [false, true, false, false, false]
        );
        assert_eq!(
            allows(TournamentStatus::participants_editable),
            [true, true, true, false, false]
        );
        assert_eq!(
            allows(TournamentStatus::invites_allowed),
            [true, true, false, false, false]
        );
    }
}
