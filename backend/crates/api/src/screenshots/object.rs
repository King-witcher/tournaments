//! O arquivo de um screenshot: o nome do objeto no bucket,
//! `screenshots/<comunidade>/<torneio>/<partida>/mapa-<n>-<aleatório>.<ext>`
//! (`screenshots/quake-br/copa/wb-1-3/mapa-2-….webp`): no console da AWS, as
//! pastas são os slugs de quando foi enviado (o objeto não se move se o endereço
//! mudar: nada compara o nome com o slug de agora), a partida e o mapa. Os 128
//! bits aleatórios fazem o nome impossível de adivinhar e único mesmo se outra
//! comunidade pegar o slug (a fila de exclusão apaga pelo nome exato, nunca pela
//! pasta). A tabela confere o formato. O tipo de imagem
//! aceito é o `shared::ImageType`.

use crate::shared::token::random_token;

pub use crate::shared::ImageType;

/// Onde vai um screenshot: os slugs da comunidade e do torneio, a partida e o
/// número do mapa.
pub struct Place<'a> {
    pub tenant: &'a str,
    pub tournament: &'a str,
    pub key: &'a str,
    pub number: i16,
}

/// Nome novo de objeto para um screenshot do mapa.
pub fn new_key(at: &Place<'_>, kind: ImageType) -> String {
    let token = random_token();
    let (tenant, tournament, key, n) = (at.tenant, at.tournament, at.key, at.number);
    format!(
        "screenshots/{tenant}/{tournament}/{key}/mapa-{n}-{token}.{}",
        kind.extension()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .strip_prefix("screenshots/quake-br/copa/wb-1-3/mapa-2-")
            .and_then(|rest| rest.strip_suffix(".jpg"))
            .unwrap_or_else(|| panic!("{name}"));
        assert_eq!(token.len(), 22);
        assert_ne!(new_key(&at, ImageType::Jpeg), name);
    }
}
