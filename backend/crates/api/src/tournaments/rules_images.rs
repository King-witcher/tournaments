//! Imagens nas regras (Markdown): só as enviadas pelo upload de `images` e
//! registradas na comunidade, no CDN do bucket público; nada de endereço de
//! fora (a imagem de outro site rastrearia quem lê e dispararia GETs lá). Ao
//! gravar as regras de um texto (o original ou uma tradução), `save` lê as
//! imagens do Markdown, recusa a que não é imagem registrada desta comunidade
//! e a que passa do limite do torneio, e regrava quais imagens o texto usa;
//! a que nenhum texto usa mais sai para a fila de exclusão pelo banco.

use pulldown_cmark::{Event, Options, Parser, Tag};
use sqlx::PgConnection;

use super::db::{self, TournamentId};
use crate::{
    AppState,
    shared::{ApiError, Locale, public_base},
    tenants::TenantId,
};

/// Imagens distintas por torneio, somando o original e as traduções (que
/// costumam repetir as mesmas). Um diagrama ou dois é o normal; com dez, a
/// página das regras pesa no máximo 10 MB (1 MB cada).
pub const MAX_PER_TOURNAMENT: i64 = 10;

const INVALID: ApiError = ApiError::invalid("tournament.rules_image_invalid");

/// O endereço do CDN para as imagens das regras; `None` sem o bucket público
/// (aí nenhuma imagem vale).
pub fn cdn(state: &AppState) -> Option<&'static str> {
    state.public_storage.as_ref().and(public_base())
}

/// Os endereços das imagens do Markdown, sem repetir, na ordem. A tag `<img`
/// em HTML cru é recusada (`tournament.rules_image_invalid`): o front não
/// desenha HTML, e assim o texto não guarda endereço nenhum de fora.
fn image_urls(markdown: &str) -> Result<Vec<String>, ApiError> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_GFM;
    let mut urls: Vec<String> = Vec::new();
    for event in Parser::new_ext(markdown, options) {
        match event {
            Event::Start(Tag::Image { dest_url, .. }) => {
                if !urls.iter().any(|u| *u == *dest_url) {
                    urls.push(dest_url.into_string());
                }
            }
            Event::Html(html) | Event::InlineHtml(html)
                if html.to_ascii_lowercase().contains("<img") =>
            {
                return Err(INVALID);
            }
            _ => {}
        }
    }
    Ok(urls)
}

/// O nome do objeto de cada endereço, que tem de estar no CDN (`cdn`, sem
/// barra no fim) e na pasta das imagens das regras.
fn keys(urls: Vec<String>, cdn: Option<&str>) -> Result<Vec<String>, ApiError> {
    if urls.is_empty() {
        return Ok(urls);
    }
    let prefix = format!("{}/rules/", cdn.ok_or(INVALID)?);
    urls.into_iter()
        .map(|url| {
            url.strip_prefix(&prefix)
                .map(|rest| format!("rules/{rest}"))
                .ok_or(INVALID)
        })
        .collect()
}

/// Grava as imagens que as regras `rules` (já limpas) do texto `locale` do
/// torneio (`None`: o original; a tradução já gravada) usam. Erros: 422
/// `tournament.rules_image_invalid` (endereço que não é uma imagem das regras
/// desta comunidade, ou `<img` em HTML cru) e `tournament.rules_images_too_many`
/// `{max}` (imagens distintas no torneio, somando os textos).
pub async fn save(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    locale: Option<Locale>,
    rules: &str,
    cdn: Option<&str>,
) -> Result<(), ApiError> {
    let keys = keys(image_urls(rules)?, cdn)?;
    let too_many = || {
        ApiError::invalid("tournament.rules_images_too_many")
            .with_params([("max", MAX_PER_TOURNAMENT)])
    };
    if i64::try_from(keys.len()).unwrap_or(i64::MAX) > MAX_PER_TOURNAMENT {
        return Err(too_many());
    }
    let ids = if keys.is_empty() {
        Vec::new()
    } else {
        db::lock_rules_images(conn, tenant, &keys).await?
    };
    if ids.len() != keys.len() {
        return Err(INVALID);
    }
    if !ids.is_empty()
        && db::rules_images_with(conn, tournament, locale, &ids).await? > MAX_PER_TOURNAMENT
    {
        return Err(too_many());
    }
    db::set_rules_images(conn, tenant, tournament, locale, &ids).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CDN: &str = "https://img.test";

    fn read(markdown: &str) -> Result<Vec<String>, &'static str> {
        keys(image_urls(markdown).map_err(|e| e.code)?, Some(CDN)).map_err(|e| e.code)
    }

    #[test]
    fn plain_text_and_links_have_no_images() {
        let text = "# Regras\n\n- md3\n- [site](https://example.com/a.png)\n\n<b>negrito</b>";
        assert_eq!(read(text), Ok(vec![]));
        assert_eq!(keys(vec![], None).map_err(|e| e.code), Ok(vec![]));
    }

    #[test]
    fn images_from_the_cdn_become_object_names_once() {
        let a = format!("{CDN}/rules/quake-br/abc.jpg");
        let b = format!("{CDN}/rules/outra/def.jpg");
        let text = format!(
            "![chave]({a})\n\n| x |\n|---|\n| ![]({b} \"título\") |\n\n![de novo]({a})\n\n![ref][r]\n\n[r]: <{b}>"
        );
        assert_eq!(
            read(&text),
            Ok(vec![
                "rules/quake-br/abc.jpg".into(),
                "rules/outra/def.jpg".into()
            ])
        );
    }

    #[test]
    fn anything_else_is_refused() {
        for text in [
            "![x](https://example.com/rules/a.jpg)".to_owned(),
            format!("![x]({CDN}/logos/quake-br/abc.jpg)"),
            format!("![x]({CDN}.evil.com/rules/abc.jpg)"),
            "![x](//img.test/rules/abc.jpg)".to_owned(),
            format!("<img src=\"{CDN}/rules/quake-br/abc.jpg\">"),
            "texto <IMG SRC=x> no meio".to_owned(),
        ] {
            assert_eq!(read(&text), Err("tournament.rules_image_invalid"), "{text}");
        }
        // Sem o bucket público, nenhuma imagem vale.
        let url = format!("{CDN}/rules/quake-br/abc.jpg");
        assert_eq!(
            keys(vec![url], None).map_err(|e| e.code),
            Err("tournament.rules_image_invalid")
        );
    }
}
