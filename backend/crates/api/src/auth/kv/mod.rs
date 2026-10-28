//! O que o domínio guarda no Redis, dividido por conceito: sessões, erros de
//! senha recentes, cadastros esperando a confirmação do email e links de uso
//! único (recuperação de senha, troca de email). Dado que vale por um tempo e
//! some sozinho (TTL).

mod attempts;
mod links;
mod sessions;
mod signups;

pub use attempts::*;
pub use links::*;
pub use sessions::*;
pub use signups::*;
