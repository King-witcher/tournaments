use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::shared::Locale;

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

/// O que o próprio usuário muda na conta.
#[derive(Deserialize, ToSchema)]
pub struct UpdateMe {
    /// Nome exibido: 1 a 32 caracteres, uma linha; espaços nas pontas são
    /// removidos. Não muda os apelidos nas comunidades.
    #[schema(example = "Rapha")]
    pub display_name: String,
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

/// Cadastro com senha: a conta só é criada quando o link do email é aberto.
#[derive(Deserialize, ToSchema)]
pub struct Signup {
    /// 3 a 20 letras ou números, `_`, `.` ou `-`; espaços nas pontas são
    /// removidos. Único sem diferenciar maiúsculas.
    #[schema(example = "Rapha")]
    pub username: String,
    /// 8 a 128 caracteres.
    pub password: String,
    /// Gravado na forma canônica: minúsculo, sem `+etiqueta` e, no Gmail, sem
    /// pontos. Domínio descartável ou de apelidos é recusado.
    #[schema(example = "rapha@gmail.com")]
    pub email: String,
    /// Idioma do email e da conta; sem ele, `pt`.
    pub locale: Option<Locale>,
}

/// Para onde o link foi (ou iria, se a conta já existe).
#[derive(Serialize, ToSchema)]
pub struct EmailSent {
    /// O endereço na forma canônica, que é para onde o link vai.
    #[schema(example = "rapha@gmail.com")]
    pub email: String,
}

/// Token de um link mandado por email (cadastro, troca de email).
#[derive(Deserialize, ToSchema)]
pub struct LinkToken {
    pub token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordResetRequest {
    /// Username ou email da conta.
    #[schema(example = "Rapha")]
    pub login: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordReset {
    /// Token do link do email.
    pub token: String,
    /// Senha nova, de 8 a 128 caracteres.
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangeEmail {
    #[schema(example = "rapha@gmail.com")]
    pub email: String,
    /// Senha atual: obrigatória se a conta tem senha.
    pub current_password: Option<String>,
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

/// Uma sessão aberta da conta.
#[derive(Serialize, ToSchema)]
pub struct SessionInfo {
    /// Identifica a sessão para encerrá-la; não serve para entrar.
    #[schema(example = "3f9a0c1b2d4e5f60")]
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// `User-Agent` do navegador que entrou, se veio.
    pub user_agent: Option<String>,
    /// É a sessão desta requisição.
    pub current: bool,
}

/// `{id}` de `/auth/sessions/{id}`.
#[derive(Deserialize)]
pub struct SessionPath {
    pub id: String,
}

/// Formas de login da conta, e o email que recupera a senha. Sempre sobra
/// pelo menos um login.
#[derive(Serialize, ToSchema)]
pub struct LoginMethods {
    pub discord: bool,
    /// Username do login com senha, se a conta tem senha.
    pub username: Option<String>,
    /// Email confirmado, na forma canônica. Sem ele, a senha esquecida não
    /// tem recuperação e o Discord não pode sair.
    pub email: Option<String>,
}
