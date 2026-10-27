//! Idiomas do site. Conteúdo do organizador tem um idioma original e
//! traduções opcionais nos outros.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiError;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    Pt,
    Es,
}

impl Locale {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::Es => "es",
        }
    }

    /// Lê uma coluna `locale` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "pt" => Ok(Self::Pt),
            "es" => Ok(Self::Es),
            other => {
                tracing::error!(locale = other, "idioma desconhecido no banco");
                Err(ApiError::internal())
            }
        }
    }
}
