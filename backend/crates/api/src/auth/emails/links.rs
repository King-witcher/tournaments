//! Os emails com link: confirmar o cadastro (que leva também o código),
//! avisar que o email já tem conta
//! (no cadastro e na troca de email de outra conta), recuperar a senha e
//! confirmar o email novo.

use super::{VALID_FOR, compose, page};
use crate::{
    mail::{Action, Content, Email},
    shared::Locale,
};

/// Confirmação do cadastro com senha: o código, para digitar na página do
/// cadastro, e o link, que cria a conta ao abrir. Vale o que vier primeiro.
pub fn signup_confirmation(web: &str, locale: Locale, to: &str, token: &str, code: &str) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/signup/confirm"));
    let content = match locale {
        Locale::Pt => Content {
            title: "Confirme seu email".into(),
            paragraphs: vec![
                "Para criar a sua conta, digite este código na página do cadastro ou use o botão abaixo."
                    .into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Criar conta".into(),
                url,
            },
            note: Some(format!(
                "O código e o link valem por {VALID_FOR}. Se não foi você, ignore este email, e a conta não é criada."
            )),
        },
        Locale::Es => Content {
            title: "Confirma tu email".into(),
            paragraphs: vec![
                "Para crear tu cuenta, escribe este código en la página de registro o usa el botón de abajo."
                    .into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Crear cuenta".into(),
                url,
            },
            note: Some(format!(
                "El código y el enlace valen por {VALID_FOR}. Si no fuiste tú, ignora este email, y la cuenta no se crea."
            )),
        },
    };
    compose(web, locale, to, content)
}

/// Alguém tentou criar conta com um email que já é de uma conta: o dono fica
/// sabendo, sem a resposta do cadastro revelar nada. Conta com senha recebe o
/// caminho da recuperação; só com Discord ou Google, o login.
pub fn account_exists(web: &str, locale: Locale, to: &str, has_password: bool) -> Email {
    let url = page(
        web,
        locale,
        if has_password {
            "/login/forgot"
        } else {
            "/login"
        },
    );
    let (title, paragraphs, label, note) = match (locale, has_password) {
        (Locale::Pt, true) => (
            "Você já tem uma conta",
            [
                "Alguém tentou criar uma conta com este email, mas ele já é da sua conta.",
                "Se foi você e esqueceu a senha, crie uma nova pelo botão abaixo.",
            ],
            "Recuperar a senha",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        (Locale::Pt, false) => (
            "Você já tem uma conta",
            [
                "Alguém tentou criar uma conta com este email, mas ele já é da sua conta, que entra pelo Discord ou pelo Google.",
                "Se foi você, é só entrar pelo que você vinculou.",
            ],
            "Entrar",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        (Locale::Es, true) => (
            "Ya tienes una cuenta",
            [
                "Alguien intentó crear una cuenta con este email, pero ya es de tu cuenta.",
                "Si fuiste tú y olvidaste la contraseña, crea una nueva con el botón de abajo.",
            ],
            "Recuperar la contraseña",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
        (Locale::Es, false) => (
            "Ya tienes una cuenta",
            [
                "Alguien intentó crear una cuenta con este email, pero ya es de tu cuenta, que entra con Discord o con Google.",
                "Si fuiste tú, solo tienes que entrar con el que vinculaste.",
            ],
            "Entrar",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
    };
    let content = Content {
        title: title.into(),
        paragraphs: paragraphs.map(String::from).into(),
        code: None,
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
pub fn email_in_use(web: &str, locale: Locale, to: &str, has_password: bool) -> Email {
    let (title, paragraphs, label, note) = match locale {
        Locale::Pt => (
            "Este email já é da sua conta",
            [
                if has_password {
                    "Alguém tentou pôr este email em outra conta daqui, mas ele já é da sua conta e continua nela."
                } else {
                    "Alguém tentou pôr este email em outra conta daqui, mas ele já é da sua conta, que entra pelo Discord ou pelo Google, e continua nela."
                },
                "Um email fica em uma conta só. Se foi você e quer usá-lo na outra, troque antes o email desta, na aba Logins.",
            ],
            "Entrar",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        Locale::Es => (
            "Este email ya es de tu cuenta",
            [
                if has_password {
                    "Alguien intentó poner este email en otra cuenta de aquí, pero ya es de tu cuenta y sigue en ella."
                } else {
                    "Alguien intentó poner este email en otra cuenta de aquí, pero ya es de tu cuenta, que entra con Discord o con Google, y sigue en ella."
                },
                "Un email está en una sola cuenta. Si fuiste tú y quieres usarlo en la otra, cambia antes el email de esta, en la pestaña Accesos.",
            ],
            "Entrar",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
    };
    let content = Content {
        title: title.into(),
        paragraphs: paragraphs.map(String::from).into(),
        code: None,
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
pub fn password_reset(web: &str, locale: Locale, to: &str, token: &str) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/login/reset"));
    let content = match locale {
        Locale::Pt => Content {
            title: "Crie uma senha nova".into(),
            paragraphs: vec!["Recebemos um pedido de senha nova para a sua conta.".into()],
            code: None,
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
            paragraphs: vec!["Recibimos un pedido de contraseña nueva para tu cuenta.".into()],
            code: None,
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
            paragraphs: vec!["Confirme para que este passe a ser o email da sua conta.".into()],
            code: None,
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
            paragraphs: vec!["Confirma para que este pase a ser el email de tu cuenta.".into()],
            code: None,
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
