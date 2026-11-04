//! O que o domínio guarda no Redis, dividido por conceito: sessões, erros de
//! senha recentes, cadastros esperando a confirmação do email, link e código
//! que vão por email (entrar, trocar o email, provar o email de agora) e vínculos
//! de provedor autorizados. Dado que vale por um tempo e some sozinho (TTL).

mod attempts;
mod email_secrets;
mod links;
mod oauth_links;
mod sessions;
mod signups;

pub use attempts::*;
pub use links::*;
pub use oauth_links::*;
pub use sessions::*;
pub use signups::*;
