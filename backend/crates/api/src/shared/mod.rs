//! O que todos os domínios usam: erro da API (inclusive a conversão dos erros
//! do `engine`), extractors de corpo e caminho, tokens, idiomas, formato de
//! slug, validação de texto e a conexão com o Redis. Nada aqui conhece um
//! domínio.

mod engine;
pub mod error;
pub mod extract;
pub mod locale;
pub mod redis;
pub mod slug;
pub mod text;
pub mod token;

pub use self::redis::Redis;
pub use error::{ApiError, ErrorBody, ParamValue};
pub use extract::{JsonBody, PathParams, nullable};
pub use locale::Locale;
