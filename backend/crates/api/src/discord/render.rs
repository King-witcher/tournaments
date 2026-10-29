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
        db::people(&state.db, job.tenant, &job.users, tournament).await?
    };
    let to = addressed(spec.to, &people, job.mention_players, job.locale);

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
/// só por menção. No TDM, os jogadores de cada time vêm juntos, depois do
/// nome do time (sem nome, "Time de {capitão}" no idioma da configuração);
/// os times, separados por " vs ".
fn addressed(to: To, people: &[Person], mention: bool, locale: Locale) -> Addressed {
    let mut mentioned = Vec::new();
    // Cada grupo: o time (no TDM) e as partes de quem é dele.
    let mut groups: Vec<(Option<&db::PersonTeam>, Vec<String>)> = Vec::new();
    for person in people {
        let id = person
            .discord_id
            .as_deref()
            .filter(|id| mention && !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()));
        let part = if let Some(id) = id {
            mentioned.push(id.to_owned());
            format!("<@{id}>")
        } else if to == To::Players {
            escape(&person.nickname)
        } else {
            continue;
        };
        let team = person.team.as_ref().filter(|_| to == To::Players);
        match groups
            .iter_mut()
            .find(|(t, _)| team.is_some() && t.map(|t| t.id) == team.map(|t| t.id))
        {
            Some((_, parts)) => parts.push(part),
            None => groups.push((team, vec![part])),
        }
    }
    mentioned.truncate(100);
    let text = if to == To::Players {
        groups
            .iter()
            .map(|(team, parts)| match team {
                Some(team) => format!("**{}**: {}", team_name(team, locale), parts.join(" ")),
                None => parts.join(" "),
            })
            .collect::<Vec<_>>()
            .join(" vs ")
    } else {
        groups
            .into_iter()
            .flat_map(|(_, parts)| parts)
            .collect::<Vec<_>>()
            .join(" ")
    };
    Addressed { text, mentioned }
}

/// O nome de um time no texto: o dele ou "Time de {capitão}".
fn team_name(team: &db::PersonTeam, locale: Locale) -> String {
    match &team.name {
        Some(name) => escape(name),
        None => {
            let of = match locale {
                Locale::Pt => "Time de",
                Locale::Es => "Equipo de",
            };
            format!("{of} {}", escape(&team.captain))
        }
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
            team: None,
        }
    }

    const PT: Locale = Locale::Pt;

    #[test]
    fn players_are_mentioned_or_named() {
        let people = [person("Ana", Some("111")), person("Bia", None)];
        let on = addressed(To::Players, &people, true, PT);
        assert_eq!(on.text, "<@111> vs Bia");
        assert_eq!(on.mentioned, ["111"]);
        let off = addressed(To::Players, &people, false, PT);
        assert_eq!(off.text, "Ana vs Bia");
        assert!(off.mentioned.is_empty());
    }

    #[test]
    fn teams_come_together_under_their_names() {
        let team = |id, name: Option<&str>, captain: &str| {
            Some(db::PersonTeam {
                id,
                name: name.map(Into::into),
                captain: captain.into(),
            })
        };
        let in_team = |nickname, discord, t: &Option<db::PersonTeam>| Person {
            team: t.clone(),
            ..person(nickname, discord)
        };
        let (a, b) = (team(1, Some("Os Bons"), "Ana"), team(2, None, "Caio"));
        let people = [
            in_team("Ana", Some("111"), &a),
            in_team("Caio", None, &b),
            in_team("Bia", None, &a),
            in_team("Duda", Some("444"), &b),
        ];
        let got = addressed(To::Players, &people, true, PT);
        assert_eq!(
            got.text,
            "**Os Bons**: <@111> Bia vs **Time de Caio**: Caio <@444>"
        );
        assert_eq!(got.mentioned, ["111", "444"]);
        let es = addressed(To::Players, &people, false, Locale::Es).text;
        assert!(es.ends_with("**Equipo de Caio**: Caio Duda"), "{es}");
    }

    #[test]
    fn staff_is_only_mentioned() {
        let people = [person("Ana", Some("111")), person("Bia", None)];
        assert_eq!(addressed(To::Staff, &people, true, PT).text, "<@111>");
        assert_eq!(addressed(To::Staff, &people, false, PT).text, "");
    }

    #[test]
    fn a_discord_id_that_is_not_a_number_is_never_mentioned() {
        let people = [person("Ana", Some("1> @everyone <@2"))];
        let got = addressed(To::Players, &people, true, PT);
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
