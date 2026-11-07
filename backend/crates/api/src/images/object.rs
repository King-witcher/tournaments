//! O nome do objeto de um logo, de uma capa ou de uma foto de perfil no
//! bucket público: `logos/<comunidade>/<aleatório>.<ext>`,
//! `covers/<comunidade>/<torneio>/<aleatório>.<ext>` e
//! `avatars/<aleatório>.<ext>` (a pessoa não tem slug, e o id interno não vai
//! para a URL); a imagem das regras, `rules/<comunidade>/<aleatório>.<ext>`
//! (da comunidade, não do torneio: a cópia de um torneio leva o texto das
//! regras com as URLs). Como nos screenshots, as
//! pastas são os slugs de quando o objeto foi criado (ele não se move se o
//! endereço mudar: o banco guarda o nome inteiro, e nada o compara com o slug
//! de agora) e os 128 bits aleatórios fazem o nome impossível de adivinhar e
//! único, então o objeto nunca muda de conteúdo e o CDN o guarda por um ano
//! (a imagem das regras, por uma hora: `cache_control`). A tabela confere o
//! formato.

use crate::shared::{
    ImageType,
    storage::{PUBLIC_CACHE, RULES_CACHE},
    token::random_token,
};

/// De quem é a imagem: o logo da comunidade, a capa de um torneio dela, a foto
/// de uma pessoa ou uma imagem das regras da comunidade.
#[derive(Debug, Clone, Copy)]
pub enum Place<'a> {
    Avatar,
    Logo {
        tenant: &'a str,
    },
    Cover {
        tenant: &'a str,
        tournament: &'a str,
    },
    Rules {
        tenant: &'a str,
    },
}

/// Nome novo de objeto para a imagem de `place`.
pub fn new_key(place: Place<'_>, kind: ImageType) -> String {
    let token = random_token();
    let ext = kind.extension();
    match place {
        Place::Avatar => format!("avatars/{token}.{ext}"),
        Place::Logo { tenant } => format!("logos/{tenant}/{token}.{ext}"),
        Place::Cover { tenant, tournament } => {
            format!("covers/{tenant}/{tournament}/{token}.{ext}")
        }
        Place::Rules { tenant } => format!("rules/{tenant}/{token}.{ext}"),
    }
}

impl Place<'_> {
    /// O `Cache-Control` que o envio grava no objeto: um ano, sem invalidar
    /// nada; a imagem das regras, uma hora, para sumir do CDN logo depois de
    /// sair do bucket.
    pub fn cache_control(self) -> &'static str {
        match self {
            Self::Rules { .. } => RULES_CACHE,
            Self::Avatar | Self::Logo { .. } | Self::Cover { .. } => PUBLIC_CACHE,
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

        let rules = new_key(Place::Rules { tenant: "quake-br" }, ImageType::Jpeg);
        assert!(rules.starts_with("rules/quake-br/"), "{rules}");

        let avatar = new_key(Place::Avatar, ImageType::Jpeg);
        let token = avatar
            .strip_prefix("avatars/")
            .and_then(|rest| rest.strip_suffix(".jpg"))
            .unwrap_or_else(|| panic!("{avatar}"));
        assert_eq!(token.len(), 22);
    }
}
