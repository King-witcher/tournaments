//! Os emails com link: confirmar o cadastro, avisar que o email já tem conta
//! (no cadastro e na troca de email de outra conta), recuperar a senha e
//! confirmar o email novo.

use super::{VALID_FOR, compose, page};
use crate::{
    mail::{Action, Content, Email},
    shared::Locale,
};

/// Link de confirmação do cadastro com senha.
pub fn signup_confirmation(
    web: &str,
    locale: Locale,
    to: &str,
    username: &str,
    token: &str,
) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/signup/confirm"));
    let content = match locale {
        Locale::Pt => Content {
            title: "Confirme seu email".into(),
            paragraphs: vec![format!(
                "Falta só confirmar este email para criar a sua conta, de nome de usuário {username}."
            )],
            alert: None,
            action: Action {
                label: "Confirmar email e criar conta".into(),
                url,
            },
            note: Some(format!(
                "O link vale por {VALID_FOR}. Se não foi você, ignore este email. A conta não é criada sem ele."
            )),
        },
        Locale::Es => Content {
            title: "Confirma tu email".into(),
            paragraphs: vec![format!(
                "Solo falta confirmar este email para crear tu cuenta, con nombre de usuario {username}."
            )],
            alert: None,
            action: Action {
                label: "Confirmar email y crear cuenta".into(),
                url,
            },
            note: Some(format!(
                "El enlace vale por {VALID_FOR}. Si no fuiste tú, ignora este email. La cuenta no se crea sin él."
            )),
        },
    };
    compose(web, locale, to, content)
}

/// Alguém tentou criar conta com um email que já é de uma conta: o dono fica
/// sabendo (e lembra do username), sem a resposta do cadastro revelar nada.
/// Conta com senha recebe o caminho da recuperação; só com Discord ou Google,
/// o login.
pub fn account_exists(web: &str, locale: Locale, to: &str, username: Option<&str>) -> Email {
    let url = page(
        web,
        locale,
        if username.is_some() {
            "/login/forgot"
        } else {
            "/login"
        },
    );
    let (title, paragraphs, label, note) = match (locale, username) {
        (Locale::Pt, Some(name)) => (
            "Você já tem uma conta",
            [
                format!("Alguém tentou criar uma conta com este email, mas ele já é da sua conta, de nome de usuário {name}."),
                "Se foi você e esqueceu a senha, crie uma nova pelo botão abaixo.".to_owned(),
            ],
            "Recuperar a senha",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        (Locale::Pt, None) => (
            "Você já tem uma conta",
            [
                "Alguém tentou criar uma conta com este email, mas ele já é da sua conta, que entra pelo Discord ou pelo Google.".to_owned(),
                "Se foi você, é só entrar pelo que você vinculou.".to_owned(),
            ],
            "Entrar",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        (Locale::Es, Some(name)) => (
            "Ya tienes una cuenta",
            [
                format!("Alguien intentó crear una cuenta con este email, pero ya es de tu cuenta, con nombre de usuario {name}."),
                "Si fuiste tú y olvidaste la contraseña, crea una nueva con el botón de abajo.".to_owned(),
            ],
            "Recuperar la contraseña",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
        (Locale::Es, None) => (
            "Ya tienes una cuenta",
            [
                "Alguien intentó crear una cuenta con este email, pero ya es de tu cuenta, que entra con Discord o con Google.".to_owned(),
                "Si fuiste tú, solo tienes que entrar con el que vinculaste.".to_owned(),
            ],
            "Entrar",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
    };
    let content = Content {
        title: title.into(),
        paragraphs: paragraphs.into(),
        alert: None,
        action: Action {
            label: label.into(),
            url,
        },
        note: Some(note.into()),
    };
    compose(web, locale, to, content)
}

/// Alguém tentou pôr na própria conta um email que já é de outra: o dono fica
/// sabendo, sem a resposta da troca revelar nada. O email continua com ele.
pub fn email_in_use(web: &str, locale: Locale, to: &str, username: Option<&str>) -> Email {
    let (title, paragraphs, label, note) = match locale {
        Locale::Pt => (
            "Este email já é da sua conta",
            [
                match username {
                    Some(name) => format!("Alguém tentou pôr este email em outra conta daqui, mas ele já é da sua conta, de nome de usuário {name}, e continua nela."),
                    None => "Alguém tentou pôr este email em outra conta daqui, mas ele já é da sua conta, que entra pelo Discord ou pelo Google, e continua nela.".to_owned(),
                },
                "Um email fica em uma conta só. Se foi você e quer usá-lo na outra, troque antes o email desta, na aba Logins.".to_owned(),
            ],
            "Entrar",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        Locale::Es => (
            "Este email ya es de tu cuenta",
            [
                match username {
                    Some(name) => format!("Alguien intentó poner este email en otra cuenta de aquí, pero ya es de tu cuenta, con nombre de usuario {name}, y sigue en ella."),
                    None => "Alguien intentó poner este email en otra cuenta de aquí, pero ya es de tu cuenta, que entra con Discord o con Google, y sigue en ella.".to_owned(),
                },
                "Un email está en una sola cuenta. Si fuiste tú y quieres usarlo en la otra, cambia antes el email de esta, en la pestaña Accesos.".to_owned(),
            ],
            "Entrar",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
    };
    let content = Content {
        title: title.into(),
        paragraphs: paragraphs.into(),
        alert: None,
        action: Action {
            label: label.into(),
            url: page(web, locale, "/login"),
        },
        note: Some(note.into()),
    };
    compose(web, locale, to, content)
}

/// Link para criar uma senha nova.
pub fn password_reset(web: &str, locale: Locale, to: &str, username: &str, token: &str) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/login/reset"));
    let content = match locale {
        Locale::Pt => Content {
            title: "Crie uma senha nova".into(),
            paragraphs: vec![format!(
                "Recebemos um pedido de senha nova para a sua conta, de nome de usuário {username}."
            )],
            alert: None,
            action: Action {
                label: "Criar senha nova".into(),
                url,
            },
            note: Some(format!(
                "O link vale por {VALID_FOR} e só uma vez. Se não foi você, ignore este email. A senha continua a mesma."
            )),
        },
        Locale::Es => Content {
            title: "Crea una contraseña nueva".into(),
            paragraphs: vec![format!(
                "Recibimos un pedido de contraseña nueva para tu cuenta, con nombre de usuario {username}."
            )],
            alert: None,
            action: Action {
                label: "Crear contraseña nueva".into(),
                url,
            },
            note: Some(format!(
                "El enlace vale por {VALID_FOR} y una sola vez. Si no fuiste tú, ignora este email. La contraseña sigue igual."
            )),
        },
    };
    compose(web, locale, to, content)
}

/// Link que confirma o email novo da conta.
pub fn email_change(web: &str, locale: Locale, to: &str, token: &str) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/email/confirm"));
    let content = match locale {
        Locale::Pt => Content {
            title: "Confirme seu email".into(),
            paragraphs: vec![
                "Este email foi indicado para a sua conta. Ele só passa a ser o email da conta quando você confirmar.".into(),
            ],
            alert: None,
            action: Action {
                label: "Confirmar email".into(),
                url,
            },
            note: Some(format!(
                "O link vale por {VALID_FOR}. Se não foi você, ignore este email. Nada muda na conta."
            )),
        },
        Locale::Es => Content {
            title: "Confirma tu email".into(),
            paragraphs: vec![
                "Este email se indicó para tu cuenta. Solo pasa a ser el email de la cuenta cuando lo confirmes.".into(),
            ],
            alert: None,
            action: Action {
                label: "Confirmar email".into(),
                url,
            },
            note: Some(format!(
                "El enlace vale por {VALID_FOR}. Si no fuiste tú, ignora este email. Nada cambia en la cuenta."
            )),
        },
    };
    compose(web, locale, to, content)
}
