//! Os emails da conta, em português e espanhol. São a exceção à regra de a
//! API só devolver códigos: o email sai daqui, sem passar pelo front. Cada
//! link abre uma página do front com o token depois do `#`, que não chega a
//! servidor nenhum (nem ao log da Vercel).

use crate::{mail::Email, shared::Locale};

/// Validade dos links mandados por email (cadastro, recuperação de senha,
/// troca de email).
pub const LINK_TTL_SECS: u64 = 60 * 60;

/// `LINK_TTL_SECS` como o texto diz (igual em português e espanhol).
const VALID_FOR: &str = "1 hora";

fn site(locale: Locale) -> &'static str {
    match locale {
        Locale::Pt => "Torneios de Quake Champions",
        Locale::Es => "Torneos de Quake Champions",
    }
}

fn page(web: &str, locale: Locale, path: &str) -> String {
    format!("{web}/{}{path}", locale.as_str())
}

/// Link de confirmação do cadastro com senha.
pub fn signup_confirmation(web: &str, locale: Locale, to: &str, token: &str) -> Email {
    let link = format!("{}#{token}", page(web, locale, "/signup/confirm"));
    let (subject, lines) = match locale {
        Locale::Pt => (
            "Confirme seu email",
            [
                "Para terminar de criar sua conta, abra o link abaixo.".to_owned(),
                format!(
                    "O link vale por {}. Se não foi você, ignore este email: a conta não é criada sem ele.",
                    VALID_FOR
                ),
            ],
        ),
        Locale::Es => (
            "Confirma tu email",
            [
                "Para terminar de crear tu cuenta, abre el enlace de abajo.".to_owned(),
                format!(
                    "El enlace vale por {}. Si no fuiste tú, ignora este email: la cuenta no se crea sin él.",
                    VALID_FOR
                ),
            ],
        ),
    };
    compose(locale, to, subject, &lines, &link)
}

/// Alguém tentou criar conta com um email que já é de uma conta: o dono fica
/// sabendo (e lembra do username), sem a resposta do cadastro revelar nada.
/// Conta com senha recebe o caminho da recuperação; só com Discord, o login.
pub fn account_exists(web: &str, locale: Locale, to: &str, username: Option<&str>) -> Email {
    let link = page(
        web,
        locale,
        if username.is_some() {
            "/login/forgot"
        } else {
            "/login"
        },
    );
    let (subject, lines) = match (locale, username) {
        (Locale::Pt, Some(name)) => (
            "Você já tem uma conta",
            [
                format!("Alguém tentou criar uma conta com este email, mas ele já é da sua conta, de nome de usuário {name}."),
                "Se foi você e esqueceu a senha, recupere pelo link abaixo. Se não foi, ignore este email.".to_owned(),
            ],
        ),
        (Locale::Pt, None) => (
            "Você já tem uma conta",
            [
                "Alguém tentou criar uma conta com este email, mas ele já é da sua conta, que entra pelo Discord.".to_owned(),
                "Se foi você, entre pelo Discord no link abaixo. Se não foi, ignore este email.".to_owned(),
            ],
        ),
        (Locale::Es, Some(name)) => (
            "Ya tienes una cuenta",
            [
                format!("Alguien intentó crear una cuenta con este email, pero ya es de tu cuenta, con nombre de usuario {name}."),
                "Si fuiste tú y olvidaste la contraseña, recupérala con el enlace de abajo. Si no, ignora este email.".to_owned(),
            ],
        ),
        (Locale::Es, None) => (
            "Ya tienes una cuenta",
            [
                "Alguien intentó crear una cuenta con este email, pero ya es de tu cuenta, que entra con Discord.".to_owned(),
                "Si fuiste tú, entra con Discord en el enlace de abajo. Si no, ignora este email.".to_owned(),
            ],
        ),
    };
    compose(locale, to, subject, &lines, &link)
}

/// Link para criar uma senha nova.
pub fn password_reset(web: &str, locale: Locale, to: &str, username: &str, token: &str) -> Email {
    let link = format!("{}#{token}", page(web, locale, "/login/reset"));
    let (subject, lines) = match locale {
        Locale::Pt => (
            "Recuperar a senha",
            [
                format!(
                    "Pediram uma senha nova para a sua conta, de nome de usuário {username}. Para criar a senha nova, abra o link abaixo."
                ),
                format!(
                    "O link vale por {} e só uma vez. Se não foi você, ignore este email: a senha continua a mesma.",
                    VALID_FOR
                ),
            ],
        ),
        Locale::Es => (
            "Recuperar la contraseña",
            [
                format!(
                    "Pidieron una contraseña nueva para tu cuenta, con nombre de usuario {username}. Para crear la contraseña nueva, abre el enlace de abajo."
                ),
                format!(
                    "El enlace vale por {VALID_FOR} y una sola vez. Si no fuiste tú, ignora este email: la contraseña sigue igual."
                ),
            ],
        ),
    };
    compose(locale, to, subject, &lines, &link)
}

/// Link que confirma o email novo da conta.
pub fn email_change(web: &str, locale: Locale, to: &str, token: &str) -> Email {
    let link = format!("{}#{token}", page(web, locale, "/email/confirm"));
    let (subject, lines) = match locale {
        Locale::Pt => (
            "Confirme seu email",
            [
                "Para usar este email na sua conta, abra o link abaixo.".to_owned(),
                format!(
                    "O link vale por {}. Se não foi você, ignore este email: nada muda na conta.",
                    VALID_FOR
                ),
            ],
        ),
        Locale::Es => (
            "Confirma tu email",
            [
                "Para usar este email en tu cuenta, abre el enlace de abajo.".to_owned(),
                format!(
                    "El enlace vale por {}. Si no fuiste tú, ignora este email: nada cambia en la cuenta.",
                    VALID_FOR
                ),
            ],
        ),
    };
    compose(locale, to, subject, &lines, &link)
}

/// Texto puro e HTML com os mesmos parágrafos e o link. Os parágrafos só têm
/// texto nosso e username (letras, números, `_`, `.` e `-`): nada a escapar.
fn compose(locale: Locale, to: &str, subject: &str, lines: &[String], link: &str) -> Email {
    let site = site(locale);
    let text = format!("{}\n\n{link}\n\n— {site}\n", lines.join("\n\n"));
    let paragraphs: String = lines.iter().map(|l| format!("<p>{l}</p>")).collect();
    let html = format!(
        "<!doctype html><html lang=\"{}\"><body style=\"font-family:sans-serif;line-height:1.5\">\
         {paragraphs}<p><a href=\"{link}\">{link}</a></p><p>— {site}</p></body></html>",
        locale.as_str()
    );
    Email {
        to: to.to_owned(),
        subject: format!("{subject} · {site}"),
        text,
        html,
    }
}
