//! O que o domínio guarda no Redis, dividido por conceito: sessões (a cópia que
//! sobrevive a um boot), cadastros esperando a confirmação do email, link e código
//! que vão por email (entrar, trocar o email, provar o email de agora) e vínculos
//! de provedor autorizados. Dado que vale por um tempo e some sozinho (TTL).

mod email_secrets;
mod links;
mod oauth_links;
mod sessions;
mod signups;

pub use links::*;
pub use oauth_links::*;
pub use sessions::*;
pub use signups::*;
