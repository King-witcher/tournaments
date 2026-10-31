//! Os avisos do que mudou no acesso à conta: o que aconteceu, "se foi você,
//! nada a fazer" e, em destaque, o que fazer se não foi, com o botão para o
//! primeiro passo. E o aviso da exclusão da conta agendada, com a data e como
//! cancelar.

use chrono::{DateTime, Utc};

use super::{compose, page};
use crate::{
    mail::{Action, Alert, Content, Email},
    shared::Locale,
};

/// O que mudou no acesso à conta.
pub enum Notice {
    /// A conta (que só entrava pelo Discord ou pelo Google) ganhou senha e
    /// passou a entrar também com o email.
    PasswordCreated,
    PasswordChanged,
    /// Senha trocada pelo link de recuperação.
    PasswordReset,
    PasswordRemoved,
    /// `discord` é o nome de usuário no Discord.
    DiscordLinked {
        discord: String,
    },
    /// `google` é o nome na conta Google.
    GoogleLinked {
        google: String,
    },
    /// `password`: a conta segue com o email e a senha; sem senha, pelo
    /// Google.
    DiscordUnlinked {
        password: bool,
    },
    /// `password`: a conta segue com o email e a senha; sem senha, pelo
    /// Discord.
    GoogleUnlinked {
        password: bool,
    },
    /// Vai para o endereço de antes (`old`).
    EmailChanged {
        old: String,
        new: String,
        has_password: bool,
    },
    /// Vai para o endereço que saiu.
    EmailRemoved {
        email: String,
    },
}

impl Notice {
    /// O endereço que saiu da conta, nos avisos de email; nos outros, `None`
    /// (o aviso vai para o email da conta).
    pub fn recipient(&self) -> Option<&str> {
        match self {
            Notice::EmailChanged { old, .. } => Some(old),
            Notice::EmailRemoved { email } => Some(email),
            _ => None,
        }
    }
}

/// O primeiro passo de quem não fez a mudança.
enum Remedy {
    /// Recuperar a senha: troca a senha e encerra todas as sessões.
    Recover,
    Sessions,
    Logins,
    Account,
}

pub fn notice(web: &str, locale: Locale, to: &str, notice: &Notice) -> Email {
    let (title, what, remedy, step) = match locale {
        Locale::Pt => pt(notice),
        Locale::Es => es(notice),
    };
    let (label, path) = match (locale, step) {
        (Locale::Pt, Remedy::Recover) => ("Recuperar a senha", "/login/forgot"),
        (Locale::Pt, Remedy::Sessions) => ("Ver as sessões", "/account/sessions"),
        (Locale::Pt, Remedy::Logins) => ("Ver os logins", "/account/logins"),
        (Locale::Pt, Remedy::Account) => ("Ir para a conta", "/account"),
        (Locale::Es, Remedy::Recover) => ("Recuperar la contraseña", "/login/forgot"),
        (Locale::Es, Remedy::Sessions) => ("Ver las sesiones", "/account/sessions"),
        (Locale::Es, Remedy::Logins) => ("Ver los accesos", "/account/logins"),
        (Locale::Es, Remedy::Account) => ("Ir a la cuenta", "/account"),
    };
    let (fine, not_you, why) = match locale {
        Locale::Pt => (
            "Se foi você, não precisa fazer nada.",
            "Não foi você?",
            "Mandamos este aviso sempre que muda algo no acesso à sua conta.",
        ),
        Locale::Es => (
            "Si fuiste tú, no necesitas hacer nada.",
            "¿No fuiste tú?",
            "Enviamos este aviso siempre que cambia algo en el acceso a tu cuenta.",
        ),
    };
    let content = Content {
        title: title.into(),
        paragraphs: vec![what, fine.into()],
        code: None,
        alert: Some(Alert {
            title: not_you.into(),
            text: remedy.into(),
        }),
        action: Action {
            label: label.into(),
            url: page(web, locale, path),
        },
        note: Some(why.into()),
    };
    compose(web, locale, to, content)
}

/// A exclusão da conta foi agendada: a data (em UTC: a API não sabe o fuso de
/// quem lê) e que entrar de novo antes cancela. O botão leva ao login.
pub fn account_deletion_scheduled(
    web: &str,
    locale: Locale,
    to: &str,
    due: DateTime<Utc>,
) -> Email {
    let url = page(web, locale, "/login");
    let when = due.format("%d/%m/%Y %H:%M");
    let content = match locale {
        Locale::Pt => Content {
            title: "Exclusão da conta agendada".into(),
            paragraphs: vec![
                format!(
                    "Sua conta vai ser excluída em {when} (UTC), e todas as sessões dela foram encerradas."
                ),
                "Para cancelar, entre de novo no site antes disso, pelo botão abaixo.".into(),
            ],
            code: None,
            alert: Some(Alert {
                title: "Não foi você que pediu?".into(),
                text: "Entre agora. A exclusão é cancelada.".into(),
            }),
            action: Action {
                label: "Entrar".into(),
                url,
            },
            note: None,
        },
        Locale::Es => Content {
            title: "Eliminación de la cuenta programada".into(),
            paragraphs: vec![
                format!(
                    "Tu cuenta se va a eliminar el {when} (UTC), y se cerraron todas sus sesiones."
                ),
                "Para cancelar, vuelve a entrar al sitio antes de esa fecha, con el botón de abajo."
                    .into(),
            ],
            code: None,
            alert: Some(Alert {
                title: "¿No fuiste tú quien lo pidió?".into(),
                text: "Entra ahora. La eliminación se cancela.".into(),
            }),
            action: Action {
                label: "Entrar".into(),
                url,
            },
            note: None,
        },
    };
    compose(web, locale, to, content)
}

/// Título, o que aconteceu, o que fazer se não foi você e o primeiro passo.
type Text = (&'static str, String, &'static str, Remedy);

fn pt(notice: &Notice) -> Text {
    const KNEW_PASSWORD: &str = "Quem fez isso sabia a sua senha. Recupere a senha pelo botão abaixo. Todas as sessões da conta são encerradas, e só a senha nova passa a valer.";
    match notice {
        Notice::PasswordCreated => (
            "Sua conta agora tem senha",
            "Foi criada uma senha para a sua conta, que agora também entra com este email e a senha.".into(),
            "Alguém pode estar com a sua conta aberta. Recupere a senha pelo botão abaixo. A senha criada deixa de valer, e todas as sessões da conta são encerradas.",
            Remedy::Recover,
        ),
        Notice::PasswordChanged => (
            "Sua senha foi trocada",
            "A senha da sua conta foi trocada, e as outras sessões abertas foram encerradas.".into(),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::PasswordReset => (
            "Sua senha foi trocada",
            "A senha da sua conta foi trocada pelo link de recuperação mandado a este email, e todas as sessões foram encerradas.".into(),
            "Então alguém tem acesso a este email. Troque a senha do email e depois recupere a senha da conta de novo pelo botão abaixo.",
            Remedy::Recover,
        ),
        Notice::PasswordRemoved => (
            "A senha da sua conta foi removida",
            "A senha da sua conta foi removida. Agora ela só entra pelo Discord ou pelo Google que você vinculou.".into(),
            "Entre por ele e encerre as sessões que você não reconhecer. Depois, se quiser, crie uma senha de novo, na aba Logins.",
            Remedy::Sessions,
        ),
        Notice::DiscordLinked { discord } => (
            "Um Discord foi vinculado à sua conta",
            format!(
                "O Discord {discord} foi vinculado à sua conta, que agora também entra por ele."
            ),
            "Quem vinculou pode entrar na sua conta pelo Discord. Entre na conta, desvincule o Discord na aba Logins e encerre as sessões que você não reconhecer.",
            Remedy::Logins,
        ),
        Notice::GoogleLinked { google } => (
            "Uma conta Google foi vinculada à sua conta",
            format!(
                "A conta Google de {google} foi vinculada à sua conta, que agora também entra por ela."
            ),
            "Quem vinculou pode entrar na sua conta pelo Google. Entre na conta, desvincule o Google na aba Logins e encerre as sessões que você não reconhecer.",
            Remedy::Logins,
        ),
        Notice::DiscordUnlinked { password: true } => (
            "O Discord foi desvinculado da sua conta",
            "Sua conta não entra mais pelo Discord, só com este email e a senha. O avatar do Discord saiu junto.".into(),
            "Quem fez isso sabia a sua senha. Recupere a senha pelo botão abaixo, o que encerra todas as sessões da conta, e depois vincule o Discord de novo.",
            Remedy::Recover,
        ),
        Notice::DiscordUnlinked { password: false } => (
            "O Discord foi desvinculado da sua conta",
            "Sua conta não entra mais pelo Discord, só pelo Google. O avatar do Discord saiu junto.".into(),
            "Quem fez isso estava com a sua conta aberta. Entre pelo Google, encerre as sessões que você não reconhecer e vincule o Discord de novo na aba Logins.",
            Remedy::Sessions,
        ),
        Notice::GoogleUnlinked { password: true } => (
            "O Google foi desvinculado da sua conta",
            "Sua conta não entra mais pelo Google, só com este email e a senha.".into(),
            "Quem fez isso sabia a sua senha. Recupere a senha pelo botão abaixo, o que encerra todas as sessões da conta, e depois vincule o Google de novo.",
            Remedy::Recover,
        ),
        Notice::GoogleUnlinked { password: false } => (
            "O Google foi desvinculado da sua conta",
            "Sua conta não entra mais pelo Google, só pelo Discord.".into(),
            "Quem fez isso estava com a sua conta aberta. Entre pelo Discord, encerre as sessões que você não reconhecer e vincule o Google de novo na aba Logins.",
            Remedy::Sessions,
        ),
        Notice::EmailChanged {
            new, has_password, ..
        } => (
            "O email da sua conta foi trocado",
            if *has_password {
                format!(
                    "O email da sua conta agora é {new}. Este endereço não serve mais para entrar nem recebe os avisos da conta ou o link de recuperação da senha."
                )
            } else {
                format!(
                    "O email da sua conta agora é {new}. Este endereço não recebe mais os avisos da conta nem o link de recuperação da senha."
                )
            },
            if *has_password {
                "Quem fez isso sabia a sua senha. Entre na conta e, na aba Logins, troque a senha (isso encerra as outras sessões) e ponha este email de volta."
            } else {
                "Entre pelo Discord ou pelo Google que você vinculou, encerre as sessões que você não reconhecer e ponha este email de volta na aba Logins."
            },
            Remedy::Account,
        ),
        Notice::EmailRemoved { .. } => (
            "Este email foi removido da sua conta",
            "Sua conta, que entra pelo Discord ou pelo Google, não tem mais email. Este endereço não recebe mais os avisos dela.".into(),
            "Entre pelo que você vinculou, encerre as sessões que você não reconhecer e cadastre este email de novo na aba Logins.",
            Remedy::Sessions,
        ),
    }
}

fn es(notice: &Notice) -> Text {
    const KNEW_PASSWORD: &str = "Quien lo hizo sabía tu contraseña. Recupera la contraseña con el botón de abajo. Todas las sesiones de la cuenta se cierran y solo vale la contraseña nueva.";
    match notice {
        Notice::PasswordCreated => (
            "Tu cuenta ahora tiene contraseña",
            "Se creó una contraseña para tu cuenta, que ahora también entra con este email y la contraseña.".into(),
            "Alguien puede tener tu cuenta abierta. Recupera la contraseña con el botón de abajo. La contraseña creada deja de valer y todas las sesiones de la cuenta se cierran.",
            Remedy::Recover,
        ),
        Notice::PasswordChanged => (
            "Tu contraseña fue cambiada",
            "La contraseña de tu cuenta fue cambiada, y las otras sesiones abiertas se cerraron."
                .into(),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::PasswordReset => (
            "Tu contraseña fue cambiada",
            "La contraseña de tu cuenta fue cambiada con el enlace de recuperación enviado a este email, y todas las sesiones se cerraron.".into(),
            "Entonces alguien tiene acceso a este email. Cambia la contraseña del email y después recupera de nuevo la contraseña de la cuenta con el botón de abajo.",
            Remedy::Recover,
        ),
        Notice::PasswordRemoved => (
            "Se quitó la contraseña de tu cuenta",
            "Se quitó la contraseña de tu cuenta. Ahora solo entra con el Discord o el Google que vinculaste.".into(),
            "Entra con él y cierra las sesiones que no reconozcas. Después, si quieres, crea de nuevo una contraseña en la pestaña Accesos.",
            Remedy::Sessions,
        ),
        Notice::DiscordLinked { discord } => (
            "Se vinculó un Discord a tu cuenta",
            format!("El Discord {discord} se vinculó a tu cuenta, que ahora también entra con él."),
            "Quien lo vinculó puede entrar en tu cuenta con Discord. Entra en la cuenta, desvincula Discord en la pestaña Accesos y cierra las sesiones que no reconozcas.",
            Remedy::Logins,
        ),
        Notice::GoogleLinked { google } => (
            "Se vinculó una cuenta de Google a tu cuenta",
            format!(
                "La cuenta de Google de {google} se vinculó a tu cuenta, que ahora también entra con ella."
            ),
            "Quien la vinculó puede entrar en tu cuenta con Google. Entra en la cuenta, desvincula Google en la pestaña Accesos y cierra las sesiones que no reconozcas.",
            Remedy::Logins,
        ),
        Notice::DiscordUnlinked { password: true } => (
            "Se desvinculó Discord de tu cuenta",
            "Tu cuenta ya no entra con Discord, solo con este email y la contraseña. El avatar de Discord se quitó también.".into(),
            "Quien lo hizo sabía tu contraseña. Recupera la contraseña con el botón de abajo, lo que cierra todas las sesiones de la cuenta, y después vincula Discord de nuevo.",
            Remedy::Recover,
        ),
        Notice::DiscordUnlinked { password: false } => (
            "Se desvinculó Discord de tu cuenta",
            "Tu cuenta ya no entra con Discord, solo con Google. El avatar de Discord se quitó también.".into(),
            "Quien lo hizo tenía tu cuenta abierta. Entra con Google, cierra las sesiones que no reconozcas y vuelve a vincular Discord en la pestaña Accesos.",
            Remedy::Sessions,
        ),
        Notice::GoogleUnlinked { password: true } => (
            "Se desvinculó Google de tu cuenta",
            "Tu cuenta ya no entra con Google, solo con este email y la contraseña.".into(),
            "Quien lo hizo sabía tu contraseña. Recupera la contraseña con el botón de abajo, lo que cierra todas las sesiones de la cuenta, y después vincula Google de nuevo.",
            Remedy::Recover,
        ),
        Notice::GoogleUnlinked { password: false } => (
            "Se desvinculó Google de tu cuenta",
            "Tu cuenta ya no entra con Google, solo con Discord.".into(),
            "Quien lo hizo tenía tu cuenta abierta. Entra con Discord, cierra las sesiones que no reconozcas y vuelve a vincular Google en la pestaña Accesos.",
            Remedy::Sessions,
        ),
        Notice::EmailChanged {
            new, has_password, ..
        } => (
            "Se cambió el email de tu cuenta",
            if *has_password {
                format!(
                    "El email de tu cuenta ahora es {new}. Esta dirección ya no sirve para entrar ni recibe los avisos de la cuenta o el enlace de recuperación de la contraseña."
                )
            } else {
                format!(
                    "El email de tu cuenta ahora es {new}. Esta dirección ya no recibe los avisos de la cuenta ni el enlace de recuperación de la contraseña."
                )
            },
            if *has_password {
                "Quien lo hizo sabía tu contraseña. Entra en la cuenta y, en la pestaña Accesos, cambia la contraseña (eso cierra las otras sesiones) y vuelve a poner este email."
            } else {
                "Entra con el Discord o el Google que vinculaste, cierra las sesiones que no reconozcas y vuelve a poner este email en la pestaña Accesos."
            },
            Remedy::Account,
        ),
        Notice::EmailRemoved { .. } => (
            "Se quitó este email de tu cuenta",
            "Tu cuenta, que entra con Discord o con Google, ya no tiene email. Esta dirección ya no recibe sus avisos.".into(),
            "Entra con el que vinculaste, cierra las sesiones que no reconozcas y vuelve a registrar este email en la pestaña Accesos.",
            Remedy::Sessions,
        ),
    }
}
