//! O que todos os domínios usam: erro da API, extractors de corpo e caminho,
//! tokens e validação de texto. Nada aqui conhece um domínio.

pub mod error;
pub mod extract;
pub mod text;
pub mod token;

pub use error::{ApiError, ErrorBody};
pub use extract::{JsonBody, PathParams};
