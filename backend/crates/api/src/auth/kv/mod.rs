//! O que o domínio guarda no Redis, dividido por conceito: sessões, erros de
//! senha recentes, cadastros esperando a confirmação do email, links de uso
//! único (recuperação de senha, troca de email) e vínculos de provedor
//! autorizados. Dado que vale por um tempo e some sozinho (TTL).

mod attempts;
mod links;
mod oauth_links;
mod sessions;
mod signups;

pub use attempts::*;
pub use links::*;
pub use oauth_links::*;
pub use sessions::*;
pub use signups::*;
