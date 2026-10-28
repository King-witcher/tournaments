//! Identificador de partida nas rotas e no banco: o `MatchId` do `engine`
//! legível, `{lado}-{rodada}-{partida}` (`wb-1-3` na chave de cima, `lb-2-1`
//! na de baixo, `gf-1-1` e `gf-2-1` na grande final e no reset). Na
//! eliminação simples tudo é `wb`. Com mais de uma fase, a fase entra na
//! rota antes da partida.

use engine::core::{BracketSide, MatchId};

pub fn format(id: MatchId) -> String {
    let side = match id.side {
        BracketSide::Winners => "wb",
        BracketSide::Losers => "lb",
        BracketSide::GrandFinal => "gf",
    };
    format!("{side}-{}-{}", id.round, id.index)
}

/// `None` se não está na forma que `format` gera (lado conhecido, rodada e
/// partida a partir de 1, sem zero à esquerda nem sinal).
pub fn parse(key: &str) -> Option<MatchId> {
    let mut parts = key.split('-');
    let side = match parts.next()? {
        "wb" => BracketSide::Winners,
        "lb" => BracketSide::Losers,
        "gf" => BracketSide::GrandFinal,
        _ => return None,
    };
    let round = parts.next()?.parse().ok()?;
    let index = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let id = MatchId { side, round, index };
    (round > 0 && index > 0 && format(id) == key).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for (side, key) in [
            (BracketSide::Winners, "wb-1-3"),
            (BracketSide::Losers, "lb-12-1"),
            (BracketSide::GrandFinal, "gf-2-1"),
        ] {
            let id = parse(key).unwrap();
            assert_eq!(id.side, side);
            assert_eq!(format(id), key);
        }
    }

    #[test]
    fn only_the_canonical_form() {
        for bad in [
            "",
            "wb",
            "wb-1",
            "wb-1-",
            "xb-1-1",
            "wb-0-1",
            "wb-1-0",
            "wb-01-1",
            "wb-+1-1",
            "wb-1-1-1",
            "WB-1-1",
            "wb-70000-1",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }
}
