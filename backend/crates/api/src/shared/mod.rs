//! O que todos os domínios usam: erro da API (inclusive a conversão dos erros
//! do `engine`), extractors de corpo e caminho, o IP de quem pede
//! (`client_ip`), tokens e códigos, idiomas, formato de slug, validação de
//! texto, a conexão com o Redis, o bucket S3 (`storage`) e a recusa de
//! escrita vinda de outra origem (`origin`). Nada aqui conhece um domínio.

pub mod client_ip;
mod engine;
pub mod error;
pub mod extract;
pub mod locale;
pub mod origin;
pub mod redis;
pub mod slug;
pub mod storage;
pub mod text;
pub mod token;

pub use self::redis::Redis;
pub use client_ip::ClientIp;
pub use error::{ApiError, ErrorBody, ParamValue};
pub use extract::{JsonBody, PathParams, nullable};
pub use locale::Locale;
pub use storage::{Storage, StorageConfig};
