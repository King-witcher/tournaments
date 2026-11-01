//! O nome do objeto de um logo ou de uma capa no bucket público:
//! `logos/<comunidade>/<aleatório>.<ext>` e
//! `covers/<comunidade>/<torneio>/<aleatório>.<ext>`. Como nos screenshots, as
//! pastas são os slugs (que não mudam) e os 128 bits aleatórios fazem o nome
//! impossível de adivinhar e único, então o objeto nunca muda de conteúdo e o
//! CDN o guarda para sempre. A tabela confere o formato.

use crate::shared::{ImageType, token::random_token};

/// De quem é a imagem: o logo da comunidade ou a capa de um torneio dela.
#[derive(Debug, Clone, Copy)]
pub enum Place<'a> {
    Logo {
        tenant: &'a str,
    },
    Cover {
        tenant: &'a str,
        tournament: &'a str,
    },
}

/// Nome novo de objeto para a imagem de `place`.
pub fn new_key(place: Place<'_>, kind: ImageType) -> String {
    let token = random_token();
    let ext = kind.extension();
    match place {
        Place::Logo { tenant } => format!("logos/{tenant}/{token}.{ext}"),
        Place::Cover { tenant, tournament } => {
            format!("covers/{tenant}/{tournament}/{token}.{ext}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_read_like_the_site() {
        let logo = new_key(Place::Logo { tenant: "quake-br" }, ImageType::Webp);
        let token = logo
            .strip_prefix("logos/quake-br/")
            .and_then(|rest| rest.strip_suffix(".webp"))
            .unwrap_or_else(|| panic!("{logo}"));
        assert_eq!(token.len(), 22);

        let place = Place::Cover {
            tenant: "quake-br",
            tournament: "copa",
        };
        let cover = new_key(place, ImageType::Jpeg);
        assert!(cover.starts_with("covers/quake-br/copa/"), "{cover}");
        assert!(cover.ends_with(".jpg"), "{cover}");
        assert_ne!(new_key(place, ImageType::Jpeg), cover);
    }
}
