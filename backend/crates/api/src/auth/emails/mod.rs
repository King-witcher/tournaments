//! Os emails da conta, em português e espanhol: os links e códigos (`links.rs`:
//! cadastro, entrar por email, troca de email) e os avisos do que mudou
//! no acesso à conta e da exclusão agendada (`notices.rs`). São a exceção à
//! regra de a API só devolver códigos: o email sai daqui, sem passar pelo
//! front. O layout (o visual do site) é o de `mail::compose`. Cada link abre
//! uma página do front com o token depois do `#`, que não chega a servidor
//! nenhum (nem ao log da Vercel).

mod links;
mod notices;

use crate::{
    mail::{self, Content, Email},
    shared::Locale,
};

pub use links::{
    account_exists, email_change, email_in_use, email_login, email_proof, signup_confirmation,
};
pub use notices::{Notice, account_deletion_reminder, account_deletion_scheduled, notice};

/// Validade do link e do código do cadastro e da troca de email.
pub const LINK_TTL_SECS: u64 = 60 * 60;

/// `LINK_TTL_SECS` como o texto diz (igual em português e espanhol).
const VALID_FOR: &str = "1 hora";

/// Validade do link e do código de entrar por email: entram na conta, então
/// valem menos que os outros.
pub const EMAIL_LOGIN_TTL_SECS: u64 = 15 * 60;

/// `EMAIL_LOGIN_TTL_SECS` como o texto diz (igual em português e espanhol).
const LOGIN_VALID_FOR: &str = "15 minutos";

fn page(web: &str, locale: Locale, path: &str) -> String {
    format!("{web}/{}{path}", locale.as_str())
}

fn compose(web: &str, locale: Locale, to: &str, content: Content) -> Email {
    mail::compose(locale, &page(web, locale, ""), to, content)
}
