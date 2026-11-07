//! O que o aviso traz como código e a mensagem mostra como texto: o código
//! da série (`wb-1-2` vira `C1.2`, como na chave do site) e o marcador de
//! conta excluída (o nome vazio vira "Conta excluída").

use serde_json::Value;

use crate::shared::Locale;

/// Os parâmetros que trazem o nome de alguém (apelido ou nome do time).
const NAMES: [&str; 6] = ["opponent", "player", "winner", "loser", "side_a", "side_b"];

/// "Conta excluída" no idioma da configuração.
pub const fn deleted_account(locale: Locale) -> &'static str {
    match locale {
        Locale::Pt => "Conta excluída",
        Locale::Es => "Cuenta eliminada",
        Locale::En => "Deleted account",
    }
}

/// `params` com os nomes vazios (conta excluída) trocados pelo texto, também
/// no pódio.
pub fn with_names(params: &Value, locale: Locale) -> Value {
    let mut params = params.clone();
    let Value::Object(map) = &mut params else {
        return params;
    };
    for (key, value) in map.iter_mut() {
        match value {
            Value::String(name) if NAMES.contains(&key.as_str()) => fill(name, locale),
            Value::Array(items) if key == "podium" => {
                for item in items {
                    if let Some(Value::String(name)) = item.get_mut("player") {
                        fill(name, locale);
                    }
                }
            }
            _ => {}
        }
    }
    params
}

fn fill(name: &mut String, locale: Locale) {
    if name.is_empty() {
        deleted_account(locale).clone_into(name);
    }
}

/// O código curto da série `key`, como a chave do site mostra: `C1.2` na
/// chave de cima (`L` em espanhol, `W` em inglês), `R2.1` na repescagem
/// (`L2.1` em inglês), `GF1` e `GF2` na grande final, `3.2` no round robin,
/// no suíço e na eliminação simples (`single`). Na disputa de posições, `1.2`:
/// o site mostra o que está em jogo ("Disputa do 3º lugar"), que não cabe
/// numa frase como "o veto da série {match}". Da fase 2 em diante (`s2-...`),
/// com a fase depois: `1.2 (fase 2)`. Fora da forma `{lado}-{rodada}-{partida}`,
/// como veio.
pub fn match_code(key: &str, single: bool, locale: Locale) -> String {
    let parsed = crate::matches::MatchKey::from(key.to_owned());
    if parsed.stage > 1 {
        let code = local_code(&parsed.local, single, locale);
        let word = match locale {
            Locale::Pt | Locale::Es => "fase",
            Locale::En => "stage",
        };
        return format!("{code} ({word} {})", parsed.stage);
    }
    local_code(key, single, locale)
}

fn local_code(key: &str, single: bool, locale: Locale) -> String {
    let mut parts = key.split('-');
    let (Some(side), Some(round), Some(index), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return key.to_owned();
    };
    match side {
        "wb" if single => format!("{round}.{index}"),
        "wb" => {
            let letter = match locale {
                Locale::Pt => "C",
                Locale::Es => "L",
                Locale::En => "W",
            };
            format!("{letter}{round}.{index}")
        }
        "lb" => {
            let letter = match locale {
                Locale::Pt | Locale::Es => "R",
                Locale::En => "L",
            };
            format!("{letter}{round}.{index}")
        }
        "gf" => format!("GF{round}"),
        "rr" | "sw" | "pl" => format!("{round}.{index}"),
        _ => key.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_code_is_the_one_on_the_bracket() {
        let pt = |key, single| match_code(key, single, Locale::Pt);
        assert_eq!(pt("wb-1-2", false), "C1.2");
        assert_eq!(pt("wb-1-2", true), "1.2");
        assert_eq!(pt("lb-3-1", false), "R3.1");
        assert_eq!(pt("gf-2-1", false), "GF2");
        assert_eq!(pt("rr-2-1", false), "2.1");
        assert_eq!(pt("sw-4-3", false), "4.3");
        assert_eq!(pt("pl-1-2", false), "1.2");
        assert_eq!(match_code("wb-1-2", false, Locale::Es), "L1.2");
        assert_eq!(match_code("lb-3-1", false, Locale::Es), "R3.1");
        let en = |key, single| match_code(key, single, Locale::En);
        assert_eq!(en("wb-1-2", false), "W1.2");
        assert_eq!(en("wb-1-2", true), "1.2");
        assert_eq!(en("lb-3-1", false), "L3.1");
        assert_eq!(en("gf-2-1", false), "GF2");
        assert_eq!(en("rr-2-1", false), "2.1");
        assert_eq!(pt("xx-1-1", false), "xx-1-1");
        assert_eq!(pt("wb-1", false), "wb-1");
        // Da fase 2 em diante, com a fase.
        assert_eq!(pt("s2-pl-1-2", false), "1.2 (fase 2)");
        assert_eq!(en("s3-wb-1-2", true), "1.2 (stage 3)");
        assert_eq!(pt("s2-wb-1-2", false), "C1.2 (fase 2)");
    }

    #[test]
    fn a_deleted_account_gets_a_name() {
        let params = json!({
            "opponent": "", "winner": "Ana", "tournament": "",
            "podium": [{ "rank": 1, "player": "" }, { "rank": 2, "player": "Bia" }],
        });
        let named = with_names(&params, Locale::Pt);
        assert_eq!(named["opponent"], "Conta excluída");
        assert_eq!(named["winner"], "Ana");
        assert_eq!(named["tournament"], "");
        assert_eq!(named["podium"][0]["player"], "Conta excluída");
        assert_eq!(named["podium"][1]["player"], "Bia");
        let es = with_names(&json!({ "loser": "" }), Locale::Es);
        assert_eq!(es["loser"], "Cuenta eliminada");
        let en = with_names(&json!({ "loser": "" }), Locale::En);
        assert_eq!(en["loser"], "Deleted account");
        let drawn = with_names(&json!({ "side_a": "", "side_b": "Ana" }), Locale::Pt);
        assert_eq!(drawn["side_a"], "Conta excluída");
        assert_eq!(drawn["side_b"], "Ana");
    }
}
