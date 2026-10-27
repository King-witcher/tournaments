//! O que todos os domínios usam: erro da API (inclusive a conversão dos erros
//! do `engine`), extractors de corpo e caminho, tokens, idiomas, formato de
//! slug e validação de texto. Nada aqui conhece um domínio.

mod engine;
pub mod error;
pub mod extract;
pub mod locale;
pub mod slug;
pub mod text;
pub mod token;

pub use error::{ApiError, ErrorBody};
pub use extract::{JsonBody, PathParams};
pub use locale::Locale;
