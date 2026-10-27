use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
pub struct LoginQuery {
    /// Caminho interno para voltar depois do login (ex.: `/pt/torneios`).
    pub return_to: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct Credentials {
    /// 3 a 20 letras ou números, `_`, `.` ou `-`; espaços nas pontas são
    /// removidos. Único sem diferenciar maiúsculas.
    #[schema(example = "Rapha")]
    pub username: String,
    /// 8 a 128 caracteres.
    pub password: String,
}

/// Campos ausentes não mudam.
#[derive(Deserialize, ToSchema)]
pub struct ChangePassword {
    pub current_password: String,
    pub username: Option<String>,
    /// Trocar a senha encerra as outras sessões da conta.
    pub password: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct CurrentPassword {
    pub current_password: String,
}

/// Formas de login da conta. Sempre sobra pelo menos uma.
#[derive(Serialize, ToSchema)]
pub struct LoginMethods {
    pub discord: bool,
    /// Username do login com senha, se a conta tem senha.
    pub username: Option<String>,
}
