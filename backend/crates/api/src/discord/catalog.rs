//! Os avisos que o Discord recebe e o texto de cada um, em português e
//! espanhol. É a exceção à regra de a API devolver só códigos (como o email,
//! em `auth/emails.rs`): a mensagem sai da API direto para outro serviço, sem
//! passar pelo front, que só traduz o que a pessoa lê no site.
//!
//! Um aviso novo (como os do veto) entra aqui com uma linha em
//! `CATALOG`: o código é o do `Notice` ou do `Announce` do outbox, e o texto
//! usa os parâmetros dele entre chaves. O resto (fila, ritmo, interruptor da
//! comunidade, menções) vale para todos. O texto também ganha, sem constar
//! dos parâmetros: `{link}` (o endereço no site, no idioma da configuração),
//! `{to}` (quem o aviso procura, ver `To`), `{score_part}` (` (2-1)` se
//! houver placar), e o `tournament` no idioma da configuração quando o torneio
//! o traduz. Lista de `{rank, player}` (o pódio) vira uma linha por item.

use serde_json::Value;

use crate::shared::Locale;

/// A quem o aviso se dirige (`{to}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum To {
    /// A ninguém: o aviso é para o canal.
    Nobody,
    /// Os jogadores da partida: mencionados (se a comunidade quer e eles
    /// vincularam o Discord) ou, sem isso, pelo apelido.
    Players,
    /// A organização: só quem vinculou o Discord é mencionado.
    Staff,
}

pub struct Spec {
    /// Código do `Notice`/`Announce` (`match.ready`).
    pub code: &'static str,
    /// Ligado numa configuração nova.
    pub default_on: bool,
    pub to: To,
    /// Avisos iguais ainda na fila viram um só, juntando quem se dirige
    /// (a partida pronta chega uma vez por lado).
    pub merge: bool,
    pt: &'static str,
    es: &'static str,
}

impl Spec {
    pub const fn text(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Pt => self.pt,
            Locale::Es => self.es,
        }
    }
}

const fn spec(
    code: &'static str,
    default_on: bool,
    to: To,
    pt: &'static str,
    es: &'static str,
) -> Spec {
    Spec {
        code,
        default_on,
        to,
        merge: false,
        pt,
        es,
    }
}

pub const CATALOG: &[Spec] = &[
    spec(
        "check_in.opened",
        true,
        To::Nobody,
        "O check-in de **{tournament}** abriu. Faça o seu para jogar.\n{link}",
        "El check-in de **{tournament}** está abierto. Haz el tuyo para jugar.\n{link}",
    ),
    spec(
        "tournament.started",
        true,
        To::Nobody,
        "**{tournament}** começou. A chave está aqui: {link}",
        "**{tournament}** empezó. La llave está aquí: {link}",
    ),
    Spec {
        merge: true,
        ..spec(
            "match.ready",
            true,
            To::Players,
            "SÃ©rie pronta em **{tournament}** ({match}): {to}\n{link}",
            "Serie lista en **{tournament}** ({match}): {to}\n{link}",
        )
    },
    spec(
        "match.opponent_ready",
        true,
        To::Players,
        "{to}: {opponent} está pronto para o veto da sÃ©rie {match} de **{tournament}** e espera você.\n{link}",
        "{to}: {opponent} está listo para el veto de la serie {match} de **{tournament}** y te espera.\n{link}",
    ),
    // Desligado numa configuração nova: quem clicou em "Pronto" já está na
    // página; serve mais para o começo forçado pela organização.
    Spec {
        merge: true,
        ..spec(
            "match.veto_started",
            false,
            To::Players,
            "O veto da sÃ©rie {match} de **{tournament}** começou: {to}\n{link}",
            "Empezó el veto de la partida {match} de **{tournament}**: {to}\n{link}",
        )
    },
    spec(
        "tournament.start_due",
        true,
        To::Staff,
        "{to} **{tournament}** chegou ao horário marcado e ainda não foi iniciado.\n{link}",
        "{to} **{tournament}** llegó a la hora marcada y aún no se inició.\n{link}",
    ),
    spec(
        "match.game_contested",
        true,
        To::Staff,
        "{to} {player} contestou a partida {game} da sÃ©rie {match} de **{tournament}**. A organização precisa resolver.\n{link}",
        "{to} {player} impugnó el mapa {game} de la partida {match} de **{tournament}**. La organización debe resolverlo.\n{link}",
    ),
    spec(
        "tournament.finished",
        true,
        To::Nobody,
        "**{tournament}** terminou. Pódio:\n{podium}\n{link}",
        "**{tournament}** terminó. Podio:\n{podium}\n{link}",
    ),
    spec(
        "match.series_decided",
        false,
        To::Nobody,
        "**{tournament}**, sÃ©rie {match}: {winner} venceu {loser}{score_part}.\n{link}",
        "**{tournament}**, partida {match}: {winner} venció a {loser}{score_part}.\n{link}",
    ),
];

pub fn find(code: &str) -> Option<&'static Spec> {
    CATALOG.iter().find(|s| s.code == code)
}

/// A mensagem do botão "Enviar teste".
pub const fn test_text(locale: Locale) -> &'static str {
    match locale {
        Locale::Pt => {
            "Teste de **{community}**. Os avisos da comunidade vão chegar neste canal.\n{link}"
        }
        Locale::Es => {
            "Prueba de **{community}**. Los avisos de la comunidad llegarán a este canal.\n{link}"
        }
    }
}

/// Monta o texto de `template`: `{nome}` vira o valor de `extra` ou o
/// parâmetro de `params` (texto ou número); o que não existe fica vazio.
pub fn render(template: &str, params: &Value, extra: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(template.len() + 64);
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('}') {
            Some(end) => {
                out.push_str(&value(&after[..end], params, extra));
                rest = &after[end + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.trim().to_owned()
}

fn value(name: &str, params: &Value, extra: &[(&str, String)]) -> String {
    if let Some((_, v)) = extra.iter().find(|(k, _)| *k == name) {
        return v.clone();
    }
    match params.get(name) {
        Some(Value::String(s)) => escape(s),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Array(items)) => items.iter().map(line).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

/// Um item de lista: `{rank, player}` vira `1. Ana`.
fn line(item: &Value) -> String {
    match (item.get("rank"), item.get("player")) {
        (Some(Value::Number(rank)), Some(Value::String(player))) => {
            format!("{rank}. {}", escape(player))
        }
        _ => String::new(),
    }
}

/// Texto que alguém escolheu (apelido, nome de torneio) não vira formatação
/// nem menção na mensagem: os caracteres do Markdown do Discord e o `@`
/// ganham uma barra. (O `allowed_mentions` já impede qualquer menção que não
/// seja a que a API pôs; isto evita que o texto tente.)
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(
            c,
            '*' | '_' | '`' | '~' | '|' | '>' | '\\' | '@' | '<' | '#' | '['
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn fills_params_and_extras() {
        let params = json!({ "tournament": "Copa", "game": 2, "ignored": null });
        let extra = [("link", "L".to_owned())];
        let text = render("{tournament} g{game} {nope}|{link}", &params, &extra);
        assert_eq!(text, "Copa g2 |L");
        assert_eq!(render("sem {chave", &params, &[]), "sem {chave");
        assert_eq!(render("  {to} x", &params, &[("to", String::new())]), "x");
    }

    #[test]
    fn extras_win_over_params() {
        let params = json!({ "tournament": "Copa" });
        let extra = [("tournament", "Taza".to_owned())];
        assert_eq!(render("{tournament}", &params, &extra), "Taza");
    }

    #[test]
    fn a_podium_is_one_line_per_place() {
        let params = json!({ "podium": [
            { "rank": 1, "player": "Ana" }, { "rank": 2, "player": "Bia" }
        ] });
        assert_eq!(render("{podium}", &params, &[]), "1. Ana\n2. Bia");
    }

    #[test]
    fn user_text_cannot_format_or_mention() {
        assert_eq!(
            escape("@everyone **x** <@1>"),
            "\\@everyone \\*\\*x\\*\\* \\<\\@1\\>"
        );
        assert_eq!(escape("Ana"), "Ana");
        let params = json!({ "player": "@here" });
        assert_eq!(render("{player}", &params, &[]), "\\@here");
    }

    #[test]
    fn every_code_is_listed_once_with_both_languages() {
        let mut codes: Vec<_> = CATALOG.iter().map(|s| s.code).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), CATALOG.len());
        for s in CATALOG {
            assert!(!s.pt.is_empty() && !s.es.is_empty(), "{}", s.code);
        }
    }
}
