//! Quem pode se inscrever pelo botão. Todo torneio publicado aparece para
//! todos os membros; a opção só controla a entrada.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::ApiError;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Admission {
    /// Qualquer membro se inscreve pelo botão.
    #[default]
    Open,
    /// Sem botão: entra-se pelo link de inscrição ou pelas mãos da
    /// organização.
    InviteOnly,
}

impl Admission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InviteOnly => "invite_only",
        }
    }

    /// Lê `tournaments.admission` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "open" => Ok(Self::Open),
            "invite_only" => Ok(Self::InviteOnly),
            other => {
                tracing::error!(admission = other, "inscrição desconhecida no banco");
                Err(ApiError::internal())
            }
        }
    }
}
