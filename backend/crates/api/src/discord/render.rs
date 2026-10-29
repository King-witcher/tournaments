//! Monta o corpo da requisição ao Discord a partir de uma mensagem da fila: o
//! texto do catálogo no idioma da configuração, o link do site, o nome do
//! torneio traduzido e as menções. `allowed_mentions` lista exatamente as
//! pessoas que a API mencionou (nunca `@everyone`, `@here` nem cargos).

use serde_json::{Value, json};

use super::{
    catalog::{self, To, escape},
    db::{self, Job, Person},
};
use crate::{AppState, shared::Locale};

/// Limite de caracteres de uma mensagem do Discord.
const MAX_CHARS: usize = 2000;

/// O corpo de `job`, ou `None` se o código já não está no catálogo.
pub async fn message(state: &AppState, job: &Job) -> Result<Option<Value>, sqlx::Error> {
    let Some(spec) = catalog::find(&job.code) else {
        return Ok(None);
    };
    let tournament = job.path.trim_start_matches('/').split('/').nth(1);
    let name = match tournament {
        Some(slug) => db::translated_name(&state.db, job.tenant, slug, job.locale).await?,
        None => None,
    };
    let people = if spec.to == To::Nobody || job.users.is_empty() {
        Vec::new()
    } else {
        db::people(&state.db, job.tenant, &job.users).await?
    };
    let to = addressed(spec.to, &people, job.mention_players);

    let mut extra = vec![
        ("link", link(&state.auth.web_origin, job.locale, &job.path)),
        ("to", to.text),
        ("score_part", score_part(&job.params)),
    ];
    if let Some(name) = name {
        extra.push(("tournament", escape(&name)));
    }
    let content = catalog::render(spec.text(job.locale), &job.params, &extra);
    Ok(Some(body(&content, &to.mentioned)))
}

/// A mensagem de teste.
pub fn test_message(origin: &str, slug: &str, community: &str, locale: Locale) -> Value {
    let extra = [
        ("community", escape(community)),
        ("link", link(origin, locale, &format!("/{slug}"))),
    ];
    let content = catalog::render(catalog::test_text(locale), &Value::Null, &extra);
    body(&content, &[])
}

fn body(content: &str, mentioned: &[String]) -> Value {
    let content: String = content.chars().take(MAX_CHARS).collect();
    json!({
        "content": content,
        "allowed_mentions": { "parse": [], "users": mentioned },
    })
}

fn link(origin: &str, locale: Locale, path: &str) -> String {
    format!("{origin}/{}{path}", locale.as_str())
}

/// ` (2-1)` quando o aviso traz o placar.
fn score_part(params: &Value) -> String {
    match params.get("score") {
        Some(Value::String(score)) => format!(" ({score})"),
        _ => String::new(),
    }
}

/// O `{to}` de uma mensagem e os ids do Discord que ele menciona.
#[derive(Debug, PartialEq, Eq)]
struct Addressed {
    text: String,
    mentioned: Vec<String>,
}

/// Quem a mensagem procura. Menciona quem vinculou o Discord, se
/// `mention`; os jogadores sem menção entram pelo apelido, a organização
/// só por menção.
fn addressed(to: To, people: &[Person], mention: bool) -> Addressed {
    let mut mentioned = Vec::new();
    let mut parts = Vec::new();
    for person in people {
        let id = person
            .discord_id
            .as_deref()
            .filter(|id| mention && !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));
        if let Some(id) = id {
            parts.push(format!("<@{id}>"));
            mentioned.push(id.to_owned());
        } else if to == To::Players {
            parts.push(escape(&person.nickname));
        }
    }
    mentioned.truncate(100);
    let separator = if to == To::Players { " vs " } else { " " };
    Addressed {
        text: parts.join(separator),
        mentioned,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn person(nickname: &str, discord_id: Option<&str>) -> Person {
        Person {
            nickname: nickname.into(),
            discord_id: discord_id.map(Into::into),
        }
    }

    #[test]
    fn players_are_mentioned_or_named() {
        let people = [person("Ana", Some("111")), person("Bia", None)];
        let on = addressed(To::Players, &people, true);
        assert_eq!(on.text, "<@111> vs Bia");
        assert_eq!(on.mentioned, ["111"]);
        let off = addressed(To::Players, &people, false);
        assert_eq!(off.text, "Ana vs Bia");
        assert!(off.mentioned.is_empty());
    }

    #[test]
    fn staff_is_only_mentioned() {
        let people = [person("Ana", Some("111")), person("Bia", None)];
        assert_eq!(addressed(To::Staff, &people, true).text, "<@111>");
        assert_eq!(addressed(To::Staff, &people, false).text, "");
    }

    #[test]
    fn a_discord_id_that_is_not_a_number_is_never_mentioned() {
        let people = [person("Ana", Some("1> @everyone <@2"))];
        let got = addressed(To::Players, &people, true);
        assert_eq!(got.text, "Ana");
        assert!(got.mentioned.is_empty());
    }

    #[test]
    fn the_body_limits_mentions_and_length() {
        let value = body(&"a".repeat(3000), &["1".into()]);
        assert_eq!(value["content"].as_str().unwrap().chars().count(), 2000);
        assert_eq!(
            value["allowed_mentions"],
            json!({ "parse": [], "users": ["1"] })
        );
    }

    #[test]
    fn the_score_is_optional() {
        assert_eq!(score_part(&json!({ "score": "2-1" })), " (2-1)");
        assert_eq!(score_part(&json!({ "score": null })), "");
        assert_eq!(score_part(&json!({})), "");
    }
}
