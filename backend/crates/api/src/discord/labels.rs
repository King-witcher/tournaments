//! O que o aviso traz como código e a mensagem mostra como texto: o código
//! da série (`wb-1-2` vira `C1.2`, como na chave do site) e o marcador de
//! conta excluída (o nome vazio vira "Conta excluída").

use serde_json::Value;

use crate::shared::Locale;

/// Os parâmetros que trazem o nome de alguém (apelido ou nome do time).
const NAMES: [&str; 4] = ["opponent", "player", "winner", "loser"];

/// "Conta excluída" no idioma da configuração.
pub const fn deleted_account(locale: Locale) -> &'static str {
    match locale {
        Locale::Pt => "Conta excluída",
        Locale::Es => "Cuenta eliminada",
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
/// chave de cima (`L` em espanhol), `R2.1` na repescagem, `GF1` e `GF2` na
/// grande final, `3.2` no round robin, no suíço e na eliminação simples
/// (`single`). Fora da forma `{lado}-{rodada}-{partida}`, como veio.
pub fn match_code(key: &str, single: bool, locale: Locale) -> String {
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
            };
            format!("{letter}{round}.{index}")
        }
        "lb" => format!("R{round}.{index}"),
        "gf" => format!("GF{round}"),
        "rr" | "sw" => format!("{round}.{index}"),
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
        assert_eq!(match_code("wb-1-2", false, Locale::Es), "L1.2");
        assert_eq!(pt("xx-1-1", false), "xx-1-1");
        assert_eq!(pt("wb-1", false), "wb-1");
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
    }
}
