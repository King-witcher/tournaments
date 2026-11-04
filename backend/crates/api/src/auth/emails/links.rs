//! Os emails com link e código: confirmar o cadastro, entrar por email e
//! confirmar o email novo. O código que prova o email de agora. E avisar que o
//! email já tem conta (no cadastro e na troca de email de outra conta), com o
//! caminho de entrar por email.

use super::{compose, login_valid_for, page, valid_for};
use crate::{
    mail::{Action, Alert, Content, Email},
    shared::Locale,
};

/// Confirmação do cadastro com senha: o código, para digitar na página do
/// cadastro, e o link, que abre a página com o botão que cria a conta. Vale o que
/// vier primeiro.
pub fn signup_confirmation(web: &str, locale: Locale, to: &str, token: &str, code: &str) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/signup/confirm"));
    let valid = valid_for(locale);
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
                "O código e o link valem por {valid}. Se não foi você, ignore este email, e a conta não é criada."
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
                "El código y el enlace valen por {valid}. Si no fuiste tú, ignora este email, y la cuenta no se crea."
            )),
        },
        Locale::En => Content {
            title: "Confirm your email".into(),
            paragraphs: vec![
                "To create your account, enter this code on the sign-up page or use the button below."
                    .into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Create account".into(),
                url,
            },
            note: Some(format!(
                "The code and the link are valid for {valid}. If this wasn't you, ignore this email and no account will be created."
            )),
        },
    };
    compose(web, locale, to, content)
}

/// Alguém tentou criar conta com um email que já é de uma conta: o dono fica
/// sabendo, sem a resposta do cadastro revelar nada, e recebe o caminho de
/// entrar por email, que serve a toda conta com email.
pub fn account_exists(web: &str, locale: Locale, to: &str) -> Email {
    let url = page(web, locale, "/login/email");
    let (title, paragraphs, label, note) = match locale {
        Locale::Pt => (
            "Você já tem uma conta",
            [
                "Alguém tentou criar uma conta com este email, mas ele já é da sua conta.",
                "Se foi você, entre por este email pelo botão abaixo. Lá também dá para definir uma senha nova.",
            ],
            "Entrar por email",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        Locale::Es => (
            "Ya tienes una cuenta",
            [
                "Alguien intentó crear una cuenta con este email, pero ya es de tu cuenta.",
                "Si fuiste tú, entra con este email usando el botón de abajo. Ahí también puedes definir una contraseña nueva.",
            ],
            "Entrar por email",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
        Locale::En => (
            "You already have an account",
            [
                "Someone tried to create an account with this email, but it already belongs to your account.",
                "If it was you, sign in with this email using the button below. You can also set a new password there.",
            ],
            "Sign in with email",
            "If this wasn't you, ignore this email. Nothing changes on your account.",
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
pub fn email_in_use(web: &str, locale: Locale, to: &str) -> Email {
    let (title, paragraphs, label, note) = match locale {
        Locale::Pt => (
            "Este email já é da sua conta",
            [
                "Alguém tentou pôr este email em outra conta daqui, mas ele já é da sua conta e continua nela.",
                "Um email fica em uma conta só. Se foi você e quer usá-lo na outra, troque antes o email desta, na aba Logins.",
            ],
            "Entrar por email",
            "Se não foi você, ignore este email. Nada muda na sua conta.",
        ),
        Locale::Es => (
            "Este email ya es de tu cuenta",
            [
                "Alguien intentó poner este email en otra cuenta de aquí, pero ya es de tu cuenta y sigue en ella.",
                "Un email está en una sola cuenta. Si fuiste tú y quieres usarlo en la otra, cambia antes el email de esta, en la pestaña Accesos.",
            ],
            "Entrar por email",
            "Si no fuiste tú, ignora este email. Nada cambia en tu cuenta.",
        ),
        Locale::En => (
            "This email is already on your account",
            [
                "Someone tried to add this email to another account here, but it already belongs to your account and stays there.",
                "An email can belong to only one account. If it was you and you want to use it on the other account, first change the email on this one, in the Logins tab.",
            ],
            "Sign in with email",
            "If this wasn't you, ignore this email. Nothing changes on your account.",
        ),
    };
    let content = Content {
        title: title.into(),
        paragraphs: paragraphs.map(String::from).into(),
        code: None,
        alert: None,
        action: Action {
            label: label.into(),
            url: page(web, locale, "/login/email"),
        },
        note: Some(note.into()),
    };
    compose(web, locale, to, content)
}

/// Entrar por email: o código, para digitar na página de entrar, e o link, que
/// abre uma página com o botão de entrar. Vale o que vier primeiro.
pub fn email_login(web: &str, locale: Locale, to: &str, token: &str, code: &str) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/login/email/confirm"));
    let login_valid = login_valid_for(locale);
    let content = match locale {
        Locale::Pt => Content {
            title: "Entre na sua conta".into(),
            paragraphs: vec![
                "Para entrar, digite este código na página de entrar ou use o botão abaixo.".into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Entrar".into(),
                url,
            },
            note: Some(format!(
                "O código e o link valem por {login_valid}. Se não foi você, ignore este email. Nada muda na conta."
            )),
        },
        Locale::Es => Content {
            title: "Entra en tu cuenta".into(),
            paragraphs: vec![
                "Para entrar, escribe este código en la página de entrar o usa el botón de abajo."
                    .into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Entrar".into(),
                url,
            },
            note: Some(format!(
                "El código y el enlace valen por {login_valid}. Si no fuiste tú, ignora este email. Nada cambia en la cuenta."
            )),
        },
        Locale::En => Content {
            title: "Sign in to your account".into(),
            paragraphs: vec![
                "To sign in, enter this code on the sign-in page or use the button below.".into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Sign in".into(),
                url,
            },
            note: Some(format!(
                "The code and the link are valid for {login_valid}. If this wasn't you, ignore this email. Nothing changes on the account."
            )),
        },
    };
    compose(web, locale, to, content)
}

/// O código que prova o email de agora, na conta que só entra por ele, antes de
/// mudar os logins. Sem link: é digitado na aba Logins, na sessão que o pediu. É
/// também o aviso de que alguém tentou, se não foi a pessoa.
pub fn email_proof(web: &str, locale: Locale, to: &str, code: &str) -> Email {
    let url = page(web, locale, "/account/sessions");
    let login_valid = login_valid_for(locale);
    let content = match locale {
        Locale::Pt => Content {
            title: "Confirme que é você".into(),
            paragraphs: vec![
                "Para mudar os logins da sua conta, digite este código na aba Logins.".into(),
            ],
            code: Some(code.into()),
            alert: Some(Alert {
                title: "Não foi você?".into(),
                text: "Alguém está com a sua conta aberta. Encerre as sessões que você não reconhecer, e sem este código ele não muda os logins.".into(),
            }),
            action: Action {
                label: "Ver as sessões".into(),
                url,
            },
            note: Some(format!("O código vale por {login_valid}.")),
        },
        Locale::Es => Content {
            title: "Confirma que eres tú".into(),
            paragraphs: vec![
                "Para cambiar los accesos de tu cuenta, escribe este código en la pestaña Accesos."
                    .into(),
            ],
            code: Some(code.into()),
            alert: Some(Alert {
                title: "¿No fuiste tú?".into(),
                text: "Alguien tiene tu cuenta abierta. Cierra las sesiones que no reconozcas; sin este código no puede cambiar los accesos.".into(),
            }),
            action: Action {
                label: "Ver las sesiones".into(),
                url,
            },
            note: Some(format!("El código vale por {login_valid}.")),
        },
        Locale::En => Content {
            title: "Confirm it's you".into(),
            paragraphs: vec![
                "To change your account's logins, enter this code in the Logins tab.".into(),
            ],
            code: Some(code.into()),
            alert: Some(Alert {
                title: "Wasn't you?".into(),
                text: "Someone has your account open. End the sessions you don't recognize; without this code they can't change the logins.".into(),
            }),
            action: Action {
                label: "View sessions".into(),
                url,
            },
            note: Some(format!("The code is valid for {login_valid}.")),
        },
    };
    compose(web, locale, to, content)
}

/// O código, para digitar na aba Logins, e o link que confirmam o email novo da
/// conta. Vale o que vier primeiro.
pub fn email_change(web: &str, locale: Locale, to: &str, token: &str, code: &str) -> Email {
    let url = format!("{}#{token}", page(web, locale, "/email/confirm"));
    let valid = valid_for(locale);
    let content = match locale {
        Locale::Pt => Content {
            title: "Confirme seu email".into(),
            paragraphs: vec![
                "Para que este passe a ser o email da sua conta, digite este código na aba Logins ou use o botão abaixo."
                    .into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Confirmar email".into(),
                url,
            },
            note: Some(format!(
                "O código e o link valem por {valid}. Se não foi você, ignore este email. Nada muda na conta."
            )),
        },
        Locale::Es => Content {
            title: "Confirma tu email".into(),
            paragraphs: vec![
                "Para que este pase a ser el email de tu cuenta, escribe este código en la pestaña Accesos o usa el botón de abajo."
                    .into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Confirmar email".into(),
                url,
            },
            note: Some(format!(
                "El código y el enlace valen por {valid}. Si no fuiste tú, ignora este email. Nada cambia en la cuenta."
            )),
        },
        Locale::En => Content {
            title: "Confirm your email".into(),
            paragraphs: vec![
                "To make this the email on your account, enter this code in the Logins tab or use the button below."
                    .into(),
            ],
            code: Some(code.into()),
            alert: None,
            action: Action {
                label: "Confirm email".into(),
                url,
            },
            note: Some(format!(
                "The code and the link are valid for {valid}. If this wasn't you, ignore this email. Nothing changes on the account."
            )),
        },
    };
    compose(web, locale, to, content)
}
