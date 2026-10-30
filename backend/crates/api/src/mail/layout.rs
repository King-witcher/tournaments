//! O layout dos emails, no visual do site (Carbon): o cabeçalho com o nome do
//! site, um bloco branco com título (`heading-04`), parágrafos (`body-02`), o
//! código para digitar no site (grande, em fonte de largura fixa), o
//! aviso em destaque (notificação de alerta, de contraste baixo), o botão
//! primário e o rodapé (`helper-text-01`). Cores do tema `white` e, onde o
//! leitor segue o modo escuro (Apple Mail, Outlook), do `g100`. Tabelas e
//! estilo inline, que é o que os leitores de email respeitam; a fonte é a
//! IBM Plex Sans de quem a tem instalada, sem buscar fora (a Helvetica ou a
//! Arial no lugar). O texto vem de quem manda, no idioma de quem recebe; aqui
//! só ficam o nome do site e o rodapé, nos dois idiomas.

use super::Email;
use crate::shared::Locale;

/// O que um email diz, já no idioma de quem recebe.
pub struct Content {
    /// Título do email e começo do assunto.
    pub title: String,
    pub paragraphs: Vec<String>,
    /// Código para a pessoa digitar no site, em destaque depois dos
    /// parágrafos (que dizem para que serve).
    pub code: Option<String>,
    /// Aviso em destaque antes do botão ("Não foi você?").
    pub alert: Option<Alert>,
    pub action: Action,
    /// Linha secundária depois do botão: validade do link, "se não foi você".
    pub note: Option<String>,
}

pub struct Alert {
    pub title: String,
    pub text: String,
}

/// O botão e o endereço dele.
pub struct Action {
    pub label: String,
    pub url: String,
}

const FONT: &str = "'IBM Plex Sans','Helvetica Neue',Arial,sans-serif";
const MONO: &str = "'IBM Plex Mono',Menlo,Consolas,monospace";

/// O modo escuro (`g100`) e as margens do celular. Os leitores que não
/// entendem `<style>` ficam com o claro, que está inline.
const STYLE: &str = "body{margin:0;padding:0}\
@media (max-width:600px){.page{padding:16px 0!important}.pad{padding-left:16px!important;padding-right:16px!important}}\
@media (prefers-color-scheme:dark){\
.bg{background:#161616!important}\
.card{background:#262626!important;border-color:#393939!important}\
.alert{background:#393939!important}\
.code{background:#393939!important}\
.fg{color:#f4f4f4!important}\
.fg2{color:#c6c6c6!important}\
.fg3{color:#a8a8a8!important}\
.link{color:#78a9ff!important}}";

struct Chrome {
    brand: &'static str,
    site: &'static str,
    fallback: &'static str,
    sent_to: &'static str,
}

fn chrome(locale: Locale) -> Chrome {
    match locale {
        Locale::Pt => Chrome {
            brand: "Torneios QC",
            site: "Torneios de Quake Champions",
            fallback: "Se o botão não abrir, copie este endereço no navegador:",
            sent_to: "Este email foi enviado para",
        },
        Locale::Es => Chrome {
            brand: "Torneos QC",
            site: "Torneos de Quake Champions",
            fallback: "Si el botón no se abre, copia esta dirección en el navegador:",
            sent_to: "Este email se envió a",
        },
    }
}

/// O email para `to`, em texto puro e em HTML com o mesmo conteúdo. `home` é
/// o início do site no idioma, linkado no rodapé.
pub fn compose(locale: Locale, home: &str, to: &str, content: Content) -> Email {
    let chrome = chrome(locale);
    Email {
        to: to.to_owned(),
        subject: format!("{} · {}", content.title, chrome.brand),
        text: text(&chrome, home, to, &content),
        html: html(locale, &chrome, home, to, &content),
    }
}

fn text(chrome: &Chrome, home: &str, to: &str, content: &Content) -> String {
    let mut out = format!("{}\n\n", content.title);
    for p in &content.paragraphs {
        out += &format!("{p}\n\n");
    }
    if let Some(code) = &content.code {
        out += &format!("{code}\n\n");
    }
    if let Some(alert) = &content.alert {
        out += &format!("{} {}\n\n", alert.title, alert.text);
    }
    out += &format!("{}: {}\n\n", content.action.label, content.action.url);
    if let Some(note) = &content.note {
        out += &format!("{note}\n\n");
    }
    out += &format!("— {}: {home}\n{} {to}.\n", chrome.site, chrome.sent_to);
    out
}

fn html(locale: Locale, chrome: &Chrome, home: &str, to: &str, content: &Content) -> String {
    let title = escape(&content.title);
    let url = escape(&content.action.url);
    let card = "class=\"card pad fg\" style=\"background:#ffffff;color:#161616;";

    let mut body = String::new();
    // Cabeçalho: 48px, como o do UI Shell.
    body += &format!(
        "<tr><td {card}padding:14px 32px;border-bottom:1px solid #e0e0e0;\
         font:600 14px/20px {FONT};letter-spacing:0.16px\">{}</td></tr>",
        escape(chrome.brand)
    );
    body += &format!(
        "<tr><td {card}padding:32px 32px 0\">\
         <h1 class=\"fg\" style=\"margin:0 0 24px;font:400 28px/36px {FONT};color:#161616\">{title}</h1>"
    );
    for p in &content.paragraphs {
        body += &format!(
            "<p class=\"fg\" style=\"margin:0 0 16px;font:16px/24px {FONT};color:#161616\">{}</p>",
            escape(p)
        );
    }
    body += "</td></tr>";
    // O código: grande, em fonte de largura fixa e espaçado, para ler e
    // digitar sem confundir; um toque seleciona ele inteiro.
    if let Some(code) = &content.code {
        body += &format!(
            "<tr><td {card}padding:0 32px\">\
             <table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\"><tr>\
             <td class=\"code fg\" style=\"background:#f4f4f4;padding:12px 16px;font:600 32px/40px {MONO};\
             letter-spacing:8px;color:#161616;-webkit-user-select:all;user-select:all\">{}</td></tr></table></td></tr>",
            escape(code)
        );
    }
    if let Some(alert) = &content.alert {
        body += &format!(
            "<tr><td {card}padding:8px 32px 0\">\
             <table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\"><tr>\
             <td class=\"alert fg\" style=\"background:#fcf4d6;border-left:3px solid #f1c21b;padding:15px 16px 15px 13px;\
             font:14px/20px {FONT};letter-spacing:0.16px;color:#161616\">\
             <strong style=\"font-weight:600\">{}</strong> {}</td></tr></table></td></tr>",
            escape(&alert.title),
            escape(&alert.text)
        );
    }
    // Botão primário, tamanho lg (48px): texto à esquerda e folga à direita.
    // 32px acima dele, contando a margem do último parágrafo quando é ele que
    // vem antes.
    let above = if content.alert.is_some() || content.code.is_some() {
        32
    } else {
        16
    };
    body += &format!(
        "<tr><td {card}padding:{above}px 32px 0\">\
         <table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\"><tr>\
         <td style=\"background:#0f62fe;mso-padding-alt:15px 63px 15px 15px\">\
         <a href=\"{url}\" style=\"display:inline-block;padding:15px 63px 15px 15px;font:14px/18px {FONT};\
         letter-spacing:0.16px;color:#ffffff;text-decoration:none\">{}</a></td></tr></table></td></tr>",
        escape(&content.action.label)
    );
    body += &format!(
        "<tr><td {card}padding:16px 32px 0\">\
         <p class=\"fg3\" style=\"margin:0;font:12px/16px {FONT};letter-spacing:0.32px;color:#6f6f6f\">{}<br>\
         <a class=\"link\" href=\"{url}\" style=\"color:#0f62fe;word-break:break-all\">{url}</a></p></td></tr>",
        escape(chrome.fallback)
    );
    if let Some(note) = &content.note {
        body += &format!(
            "<tr><td {card}padding:24px 32px 0\">\
             <p class=\"fg2\" style=\"margin:0;font:14px/20px {FONT};letter-spacing:0.16px;color:#525252\">{}</p></td></tr>",
            escape(note)
        );
    }
    body += &format!("<tr><td {card}padding:0;height:40px\"></td></tr>");
    body += &format!(
        "<tr><td class=\"pad fg3\" style=\"padding:24px 32px 0;font:12px/16px {FONT};letter-spacing:0.32px;color:#6f6f6f\">\
         <a class=\"link\" href=\"{}\" style=\"color:#0f62fe\">{}</a><br>{} {}.</td></tr>",
        escape(home),
        escape(chrome.site),
        escape(chrome.sent_to),
        escape(to)
    );

    // A prévia que a caixa de entrada mostra: o primeiro parágrafo.
    let preheader = content.paragraphs.first().map_or("", String::as_str);
    format!(
        "<!doctype html><html lang=\"{lang}\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <meta name=\"color-scheme\" content=\"light dark\"><meta name=\"supported-color-schemes\" content=\"light dark\">\
         <title>{title}</title><style>{STYLE}</style></head>\
         <body class=\"bg\" style=\"margin:0;padding:0;background:#f4f4f4\">\
         <div style=\"display:none;max-height:0;overflow:hidden\">{preheader}</div>\
         <table role=\"presentation\" class=\"bg\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"background:#f4f4f4\">\
         <tr><td class=\"page\" align=\"center\" style=\"padding:32px 16px\">\
         <table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"max-width:600px\">\
         {body}</table></td></tr></table></body></html>",
        lang = locale.as_str(),
        preheader = escape(preheader),
    )
}

/// Texto pronto para entrar no HTML, em conteúdo ou atributo.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content(alert: bool) -> Content {
        Content {
            title: "Título <b>".into(),
            paragraphs: vec!["Primeiro & único.".into()],
            code: alert.then(|| "042917".into()),
            alert: alert.then(|| Alert {
                title: "Não foi você?".into(),
                text: "Faça \"isto\".".into(),
            }),
            action: Action {
                label: "Abrir".into(),
                url: "https://site.test/pt/x#abc".into(),
            },
            note: Some("Vale 1 hora.".into()),
        }
    }

    #[test]
    fn html_escapes_every_text() {
        let email = compose(Locale::Pt, "https://site.test/pt", "a@b.com", content(true));
        assert!(email.html.contains("Título &lt;b&gt;"));
        assert!(email.html.contains("Primeiro &amp; único."));
        assert!(email.html.contains("Faça &quot;isto&quot;."));
        assert!(!email.html.contains("<b>"));
        assert!(email.html.starts_with("<!doctype html><html lang=\"pt\">"));
    }

    #[test]
    fn text_and_html_carry_the_same_content() {
        let email = compose(Locale::Es, "https://site.test/es", "a@b.com", content(true));
        assert_eq!(email.subject, "Título <b> · Torneos QC");
        for part in [
            "Primeiro & único.",
            "\n042917\n",
            "Não foi você?",
            "Vale 1 hora.",
            "a@b.com",
        ] {
            assert!(email.text.contains(part), "{part}");
        }
        assert!(email.text.contains("Abrir: https://site.test/pt/x#abc\n"));
        assert_eq!(
            email
                .html
                .matches("href=\"https://site.test/pt/x#abc\"")
                .count(),
            2
        );
        assert!(email.html.contains("Este email se envió a a@b.com."));
        assert!(email.html.contains(">042917</td>"));
    }

    #[test]
    fn alert_only_when_given() {
        let email = compose(
            Locale::Pt,
            "https://site.test/pt",
            "a@b.com",
            content(false),
        );
        assert!(!email.html.contains("#fcf4d6"));
        assert!(!email.text.contains("Não foi você?"));
        assert!(!email.html.contains("042917"));
    }
}
