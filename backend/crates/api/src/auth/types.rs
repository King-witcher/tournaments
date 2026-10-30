use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{auth::CurrentUser, shared::Locale};

/// Abrir o OAuth de um provedor para ligá-lo à conta logada, com o que o
/// navegador gerou e guardou na aba. Para entrar, o front monta a URL de
/// autorização sozinho, com o client id público, e não passa por aqui.
#[derive(Deserialize, ToSchema)]
pub struct OAuthStart {
    /// O que o provedor devolve ao callback, para o front conferir que a volta
    /// é do fluxo desta aba: 22 a 128 caracteres de `A-Z a-z 0-9 - . _ ~`.
    pub state: String,
    /// `BASE64URL(SHA-256(code_verifier))`, sem `=` (43 caracteres): o
    /// método é sempre `S256`.
    pub code_challenge: String,
    /// Senha atual: obrigatória se a conta tem senha.
    pub current_password: Option<String>,
}

/// A autorização no provedor, para onde o navegador vai.
#[derive(Serialize, ToSchema)]
pub struct OAuthRedirect {
    pub url: String,
}

/// O code que o provedor mandou ao callback do front, com o verifier da aba.
#[derive(Deserialize, ToSchema)]
pub struct OAuthCode {
    pub code: String,
    /// O verifier do challenge que foi na autorização: 43 a 128 caracteres de
    /// `A-Z a-z 0-9 - . _ ~`.
    pub code_verifier: String,
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

/// Desligar um provedor (Discord ou Google) da conta.
#[derive(Deserialize, ToSchema)]
pub struct UnlinkLogin {
    /// Senha atual: obrigatória se a conta tem senha.
    pub current_password: Option<String>,
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

/// Pedido de exclusão da conta logada.
#[derive(Deserialize, ToSchema)]
pub struct ScheduleAccountDeletion {
    /// Senha atual: obrigatória se a conta tem senha.
    pub current_password: Option<String>,
}

/// A exclusão agendada da conta.
#[derive(Serialize, ToSchema)]
pub struct AccountDeletion {
    /// Quando a conta é anonimizada, se ninguém entrar nela antes.
    pub scheduled_for: DateTime<Utc>,
}

/// Quem acabou de entrar (login, cadastro confirmado, senha nova pelo link).
#[derive(Serialize, ToSchema)]
pub struct SignedIn {
    #[serde(flatten)]
    pub user: CurrentUser,
    /// Entrar cancelou a exclusão da conta, que estava agendada.
    pub deletion_cancelled: bool,
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
    pub google: bool,
    /// Username do login com senha, se a conta tem senha.
    pub username: Option<String>,
    /// Email confirmado, na forma canônica. Toda conta com senha tem (é por
    /// ele que a senha esquecida volta); só a conta sem senha fica sem.
    pub email: Option<String>,
}
