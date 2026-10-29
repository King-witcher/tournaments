//! O arquivo de um print: o tipo de imagem aceito e o nome do objeto no
//! bucket, `prints/<comunidade>/<torneio>/<partida>/mapa-<n>-<aleatório>.<ext>`
//! (`prints/quake-br/copa/wb-1-3/mapa-2-….webp`): no console da AWS, as pastas
//! são os slugs (que não mudam), a partida e o mapa. Os 128 bits aleatórios
//! fazem o nome impossível de adivinhar e únicos mesmo se uma comunidade
//! apagada tiver o slug reaproveitado (a fila de exclusão apaga pelo nome
//! exato, nunca pela pasta). A tabela confere o formato.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::token::random_token;

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

    const fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Jpeg => "jpg",
        }
    }

    pub fn from_mime(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.mime() == value)
    }
}

/// Onde vai um print: os slugs da comunidade e do torneio, a partida e o
/// número do mapa.
pub struct Place<'a> {
    pub tenant: &'a str,
    pub tournament: &'a str,
    pub key: &'a str,
    pub number: i16,
}

/// Nome novo de objeto para um print do mapa.
pub fn new_key(at: &Place<'_>, kind: ImageType) -> String {
    let token = random_token();
    let (tenant, tournament, key, n) = (at.tenant, at.tournament, at.key, at.number);
    format!(
        "prints/{tenant}/{tournament}/{key}/mapa-{n}-{token}.{}",
        kind.extension()
    )
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

    #[test]
    fn the_name_reads_like_the_site() {
        let at = Place {
            tenant: "quake-br",
            tournament: "copa",
            key: "wb-1-3",
            number: 2,
        };
        let name = new_key(&at, ImageType::Jpeg);
        let token = name
            .strip_prefix("prints/quake-br/copa/wb-1-3/mapa-2-")
            .and_then(|rest| rest.strip_suffix(".jpg"))
            .unwrap_or_else(|| panic!("{name}"));
        assert_eq!(token.len(), 22);
        assert_ne!(new_key(&at, ImageType::Jpeg), name);
    }
}
