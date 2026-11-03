//! Ciclo de vida do torneio e o que cada fase dele ainda deixa mudar.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::ApiError;

/// rascunho → inscrição → check-in → em andamento → encerrado.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TournamentStatus {
    /// Só organizer e owner veem. Único status em que o endereço muda e em
    /// que excluir não pede o endereço digitado.
    Draft,
    Registration,
    CheckIn,
    InProgress,
    Finished,
}

impl TournamentStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Registration => "registration",
            Self::CheckIn => "check_in",
            Self::InProgress => "in_progress",
            Self::Finished => "finished",
        }
    }

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

    /// Vagas, regra de campeões, fases, data de início e janela de check-in
    /// só mudam antes de o torneio começar.
    pub const fn setup_editable(self) -> bool {
        matches!(self, Self::Draft | Self::Registration | Self::CheckIn)
    }

    /// Check-in (o próprio inscrito ou o organizador) só se faz e desfaz aqui.
    pub const fn check_in_open(self) -> bool {
        matches!(self, Self::CheckIn)
    }

    /// Já começou: a chave existe. Daqui em diante, só o owner exclui.
    pub const fn has_started(self) -> bool {
        matches!(self, Self::InProgress | Self::Finished)
    }

    /// Encerrado: só o nome, a descrição (com as traduções deles) e a capa
    /// ainda mudam.
    pub const fn is_finished(self) -> bool {
        matches!(self, Self::Finished)
    }

    /// O próprio membro se inscreve, sai ou entra por convite aqui: a
    /// inscrição fica aberta até o início, também durante o check-in.
    pub const fn registration_open(self) -> bool {
        matches!(self, Self::Registration | Self::CheckIn)
    }

    /// O organizador inscreve e tira participantes até o torneio começar.
    pub const fn participants_editable(self) -> bool {
        matches!(self, Self::Draft | Self::Registration | Self::CheckIn)
    }

    /// O link de inscrição aparece e se troca: já no rascunho (para mandar
    /// antes de publicar) e até o início.
    pub const fn invites_allowed(self) -> bool {
        matches!(self, Self::Draft | Self::Registration | Self::CheckIn)
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
            allows(TournamentStatus::has_started),
            [false, false, false, true, true]
        );
        assert_eq!(
            allows(TournamentStatus::is_finished),
            [false, false, false, false, true]
        );
        assert_eq!(
            allows(TournamentStatus::registration_open),
            [false, true, true, false, false]
        );
        assert_eq!(
            allows(TournamentStatus::participants_editable),
            [true, true, true, false, false]
        );
        assert_eq!(
            allows(TournamentStatus::invites_allowed),
            [true, true, true, false, false]
        );
        assert_eq!(
            allows(TournamentStatus::check_in_open),
            [false, false, true, false, false]
        );
    }
}
