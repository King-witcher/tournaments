//! Tipo das imagens que o navegador manda aos buckets (screenshots, logos,
//! capas).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Tipo da imagem. O front converte para WebP; o Safari, que não gera WebP
/// no canvas, manda JPEG.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    #[serde(rename = "image/webp")]
    Webp,
    #[serde(rename = "image/jpeg")]
    Jpeg,
}

impl ImageType {
    pub const ALL: [Self; 2] = [Self::Webp, Self::Jpeg];

    pub const fn mime(self) -> &'static str {
        match self {
            Self::Webp => "image/webp",
            Self::Jpeg => "image/jpeg",
        }
    }

    /// A extensão do nome do objeto.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Jpeg => "jpg",
        }
    }

    pub fn from_mime(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.mime() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_round_trips() {
        for t in ImageType::ALL {
            assert_eq!(ImageType::from_mime(t.mime()), Some(t));
        }
        assert_eq!(ImageType::from_mime("image/png"), None);
        assert_eq!(
            serde_json::to_value(ImageType::Webp).unwrap(),
            serde_json::json!("image/webp")
        );
    }
}
