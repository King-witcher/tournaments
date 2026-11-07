//! Identificador de partida nas rotas e no banco: o `MatchId` do `engine`
//! legível, `{lado}-{rodada}-{partida}` (`wb-1-3` na chave de cima, `lb-2-1`
//! na de baixo, `gf-1-1` e `gf-2-1` na grande final e no reset, `rr-3-2` no
//! round robin, `sw-4-1` no suíço, `pl-1-2` no par 2 da disputa de
//! posições). Na eliminação simples tudo é `wb`.
//!
//! Da fase 2 em diante, o código público (rotas, respostas, avisos, tempo
//! real, Discord) ganha o prefixo da fase, `s<N>-` (`s2-pl-1-1`); a fase 1
//! continua sem prefixo, e os links antigos valem. No banco, `match_key` fica
//! sem prefixo, ao lado da coluna `stage`.

use engine::core::{BracketSide, MatchId};
use serde::Deserialize;

/// O identificador de uma partida no banco (sem a fase).
pub fn format(id: MatchId) -> String {
    let side = match id.side {
        BracketSide::Winners => "wb",
        BracketSide::Losers => "lb",
        BracketSide::GrandFinal => "gf",
        BracketSide::RoundRobin => "rr",
        BracketSide::Swiss => "sw",
        BracketSide::Placement => "pl",
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
        "rr" => BracketSide::RoundRobin,
        "sw" => BracketSide::Swiss,
        "pl" => BracketSide::Placement,
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

/// O código público da partida `id` da fase `stage`.
pub fn public(stage: i16, id: MatchId) -> String {
    prefixed(stage, &format(id))
}

/// O código público do identificador do banco `key` na fase `stage`.
pub fn prefixed(stage: i16, key: &str) -> String {
    if stage > 1 {
        format!("s{stage}-{key}")
    } else {
        key.to_owned()
    }
}

/// O código público de uma partida, como chega na rota: a fase e o
/// identificador do banco. Fora da forma (`s1-`, `s02-`, prefixo sem
/// partida), fica na fase 1 com o texto inteiro, que não acha partida
/// nenhuma (404 `match.not_found`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "String")]
pub struct MatchKey {
    pub stage: i16,
    /// O identificador no banco, sem a fase.
    pub local: String,
}

impl From<String> for MatchKey {
    fn from(public: String) -> Self {
        let split = public.strip_prefix('s').and_then(|rest| {
            let (number, local) = rest.split_once('-')?;
            let stage: i16 = number.parse().ok()?;
            (stage > 1 && number == stage.to_string()).then(|| (stage, local.to_owned()))
        });
        match split {
            Some((stage, local)) => Self { stage, local },
            None => Self {
                stage: 1,
                local: public,
            },
        }
    }
}

impl MatchKey {
    /// O código público, como entrou.
    pub fn public(&self) -> String {
        prefixed(self.stage, &self.local)
    }
}

impl std::fmt::Display for MatchKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.public())
    }
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
            (BracketSide::RoundRobin, "rr-31-16"),
            (BracketSide::Swiss, "sw-4-1"),
            (BracketSide::Placement, "pl-1-2"),
        ] {
            let id = parse(key).unwrap();
            assert_eq!(id.side, side);
            assert_eq!(format(id), key);
        }
    }

    #[test]
    fn the_stage_prefix_from_the_second_stage_on() {
        let id = parse("pl-1-2").unwrap();
        assert_eq!(public(1, id), "pl-1-2");
        assert_eq!(public(2, id), "s2-pl-1-2");
        let key = |s: &str| MatchKey::from(s.to_owned());
        assert_eq!(
            key("s2-pl-1-2"),
            MatchKey {
                stage: 2,
                local: "pl-1-2".into()
            }
        );
        assert_eq!(key("s2-pl-1-2").public(), "s2-pl-1-2");
        assert_eq!(key("wb-1-1").stage, 1);
        // Fora da forma, fica na fase 1 e não acha partida.
        for bad in ["s1-wb-1-1", "s02-wb-1-1", "s2", "sx-wb-1-1", "s-wb-1-1"] {
            let k = key(bad);
            assert_eq!((k.stage, k.local.as_str()), (1, bad), "{bad}");
            assert_eq!(parse(&k.local), None, "{bad}");
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
            "sw-0-1",
            "RR-1-1",
            "pl-01-1",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }
}
