//! O que o domínio guarda no Redis, dividido por conceito: sessões e erros de
//! senha recentes. Dado que vale por um tempo e some sozinho (TTL).

mod attempts;
mod sessions;

pub use attempts::*;
pub use sessions::*;
