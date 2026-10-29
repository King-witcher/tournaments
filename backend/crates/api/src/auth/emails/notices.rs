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
    /// A conta (que só entrava pelo Discord ou pelo Google) ganhou username e
    /// senha.
    PasswordCreated {
        username: String,
    },
    /// Senha trocada na conta, e o username também se `renamed`.
    PasswordChanged {
        username: String,
        renamed: bool,
    },
    /// Só o username trocado.
    UsernameChanged {
        username: String,
    },
    /// Senha trocada pelo link de recuperação.
    PasswordReset {
        username: String,
    },
    PasswordRemoved,
    /// `discord` é o nome de usuário no Discord.
    DiscordLinked {
        discord: String,
    },
    /// `google` é o nome na conta Google.
    GoogleLinked {
        google: String,
    },
    /// `username` é o do login com senha que fica; sem senha, a conta segue
    /// pelo Google.
    DiscordUnlinked {
        username: Option<String>,
    },
    /// `username` é o do login com senha que fica; sem senha, a conta segue
    /// pelo Discord.
    GoogleUnlinked {
        username: Option<String>,
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
            alert: Some(Alert {
                title: "Não foi você que pediu?".into(),
                text: "Entre agora: a exclusão é cancelada.".into(),
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
            alert: Some(Alert {
                title: "¿No fuiste tú quien lo pidió?".into(),
                text: "Entra ahora: la eliminación se cancela.".into(),
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
    const KNEW_PASSWORD: &str = "Quem fez isso sabia a sua senha. Recupere a senha pelo botão abaixo: todas as sessões da conta são encerradas, e só a senha nova passa a valer.";
    match notice {
        Notice::PasswordCreated { username } => (
            "Sua conta agora tem senha",
            format!(
                "Foram criados um nome de usuário e uma senha para entrar na sua conta. O nome de usuário é {username}."
            ),
            "Alguém pode estar com a sua conta aberta. Recupere a senha pelo botão abaixo: a senha criada deixa de valer, e todas as sessões da conta são encerradas.",
            Remedy::Recover,
        ),
        Notice::PasswordChanged {
            username,
            renamed: false,
        } => (
            "Sua senha foi trocada",
            format!(
                "A senha da sua conta, de nome de usuário {username}, foi trocada, e as outras sessões abertas foram encerradas."
            ),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::PasswordChanged {
            username,
            renamed: true,
        } => (
            "Seu nome de usuário e sua senha foram trocados",
            format!(
                "O nome de usuário da sua conta agora é {username}, a senha foi trocada, e as outras sessões abertas foram encerradas."
            ),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::UsernameChanged { username } => (
            "Seu nome de usuário foi trocado",
            format!(
                "O nome de usuário da sua conta agora é {username}. A senha continua a mesma."
            ),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::PasswordReset { username } => (
            "Sua senha foi trocada",
            format!(
                "A senha da sua conta, de nome de usuário {username}, foi trocada pelo link de recuperação mandado a este email, e todas as sessões foram encerradas."
            ),
            "Então alguém tem acesso a este email. Troque a senha do email e depois recupere a senha da conta de novo pelo botão abaixo.",
            Remedy::Recover,
        ),
        Notice::PasswordRemoved => (
            "A senha da sua conta foi removida",
            "O nome de usuário e a senha da sua conta foram removidos: agora ela só entra pelo Discord ou pelo Google que você vinculou.".into(),
            "Entre por ele e encerre as sessões que você não reconhecer. Depois, se quiser, crie um nome de usuário e uma senha de novo, na aba Logins.",
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
        Notice::DiscordUnlinked {
            username: Some(username),
        } => (
            "O Discord foi desvinculado da sua conta",
            format!(
                "Sua conta não entra mais pelo Discord, só com o nome de usuário {username} e a senha. O avatar do Discord saiu junto."
            ),
            "Quem fez isso sabia a sua senha. Recupere a senha pelo botão abaixo, o que encerra todas as sessões da conta, e depois vincule o Discord de novo.",
            Remedy::Recover,
        ),
        Notice::DiscordUnlinked { username: None } => (
            "O Discord foi desvinculado da sua conta",
            "Sua conta não entra mais pelo Discord, só pelo Google. O avatar do Discord saiu junto.".into(),
            "Quem fez isso estava com a sua conta aberta. Entre pelo Google, encerre as sessões que você não reconhecer e vincule o Discord de novo na aba Logins.",
            Remedy::Sessions,
        ),
        Notice::GoogleUnlinked {
            username: Some(username),
        } => (
            "O Google foi desvinculado da sua conta",
            format!(
                "Sua conta não entra mais pelo Google, só com o nome de usuário {username} e a senha."
            ),
            "Quem fez isso sabia a sua senha. Recupere a senha pelo botão abaixo, o que encerra todas as sessões da conta, e depois vincule o Google de novo.",
            Remedy::Recover,
        ),
        Notice::GoogleUnlinked { username: None } => (
            "O Google foi desvinculado da sua conta",
            "Sua conta não entra mais pelo Google, só pelo Discord.".into(),
            "Quem fez isso estava com a sua conta aberta. Entre pelo Discord, encerre as sessões que você não reconhecer e vincule o Google de novo na aba Logins.",
            Remedy::Sessions,
        ),
        Notice::EmailChanged {
            new, has_password, ..
        } => (
            "O email da sua conta foi trocado",
            format!(
                "O email da sua conta agora é {new}. Este endereço não recebe mais os avisos da conta nem o link de recuperação da senha."
            ),
            if *has_password {
                "Quem fez isso sabia a sua senha. Entre na conta, troque a senha na aba Senha (isso encerra as outras sessões) e ponha este email de volta na aba Logins."
            } else {
                "Entre pelo Discord ou pelo Google que você vinculou, encerre as sessões que você não reconhecer e ponha este email de volta na aba Logins."
            },
            Remedy::Account,
        ),
        Notice::EmailRemoved { .. } => (
            "Este email foi removido da sua conta",
            "Sua conta, que entra pelo Discord ou pelo Google, não tem mais email: este endereço não recebe mais os avisos dela.".into(),
            "Entre pelo que você vinculou, encerre as sessões que você não reconhecer e cadastre este email de novo na aba Logins.",
            Remedy::Sessions,
        ),
    }
}

fn es(notice: &Notice) -> Text {
    const KNEW_PASSWORD: &str = "Quien lo hizo sabía tu contraseña. Recupera la contraseña con el botón de abajo: todas las sesiones de la cuenta se cierran y solo vale la contraseña nueva.";
    match notice {
        Notice::PasswordCreated { username } => (
            "Tu cuenta ahora tiene contraseña",
            format!(
                "Se crearon un nombre de usuario y una contraseña para entrar en tu cuenta. El nombre de usuario es {username}."
            ),
            "Alguien puede tener tu cuenta abierta. Recupera la contraseña con el botón de abajo: la contraseña creada deja de valer y todas las sesiones de la cuenta se cierran.",
            Remedy::Recover,
        ),
        Notice::PasswordChanged {
            username,
            renamed: false,
        } => (
            "Tu contraseña fue cambiada",
            format!(
                "La contraseña de tu cuenta, con nombre de usuario {username}, fue cambiada, y las otras sesiones abiertas se cerraron."
            ),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::PasswordChanged {
            username,
            renamed: true,
        } => (
            "Tu nombre de usuario y tu contraseña fueron cambiados",
            format!(
                "El nombre de usuario de tu cuenta ahora es {username}, la contraseña fue cambiada y las otras sesiones abiertas se cerraron."
            ),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::UsernameChanged { username } => (
            "Tu nombre de usuario fue cambiado",
            format!(
                "El nombre de usuario de tu cuenta ahora es {username}. La contraseña sigue igual."
            ),
            KNEW_PASSWORD,
            Remedy::Recover,
        ),
        Notice::PasswordReset { username } => (
            "Tu contraseña fue cambiada",
            format!(
                "La contraseña de tu cuenta, con nombre de usuario {username}, fue cambiada con el enlace de recuperación enviado a este email, y todas las sesiones se cerraron."
            ),
            "Entonces alguien tiene acceso a este email. Cambia la contraseña del email y después recupera de nuevo la contraseña de la cuenta con el botón de abajo.",
            Remedy::Recover,
        ),
        Notice::PasswordRemoved => (
            "Se quitó la contraseña de tu cuenta",
            "El nombre de usuario y la contraseña de tu cuenta se quitaron: ahora solo entra con el Discord o el Google que vinculaste.".into(),
            "Entra con él y cierra las sesiones que no reconozcas. Después, si quieres, crea de nuevo un nombre de usuario y una contraseña en la pestaña Accesos.",
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
        Notice::DiscordUnlinked {
            username: Some(username),
        } => (
            "Se desvinculó Discord de tu cuenta",
            format!(
                "Tu cuenta ya no entra con Discord, solo con el nombre de usuario {username} y la contraseña. El avatar de Discord se quitó también."
            ),
            "Quien lo hizo sabía tu contraseña. Recupera la contraseña con el botón de abajo, lo que cierra todas las sesiones de la cuenta, y después vincula Discord de nuevo.",
            Remedy::Recover,
        ),
        Notice::DiscordUnlinked { username: None } => (
            "Se desvinculó Discord de tu cuenta",
            "Tu cuenta ya no entra con Discord, solo con Google. El avatar de Discord se quitó también.".into(),
            "Quien lo hizo tenía tu cuenta abierta. Entra con Google, cierra las sesiones que no reconozcas y vuelve a vincular Discord en la pestaña Accesos.",
            Remedy::Sessions,
        ),
        Notice::GoogleUnlinked {
            username: Some(username),
        } => (
            "Se desvinculó Google de tu cuenta",
            format!(
                "Tu cuenta ya no entra con Google, solo con el nombre de usuario {username} y la contraseña."
            ),
            "Quien lo hizo sabía tu contraseña. Recupera la contraseña con el botón de abajo, lo que cierra todas las sesiones de la cuenta, y después vincula Google de nuevo.",
            Remedy::Recover,
        ),
        Notice::GoogleUnlinked { username: None } => (
            "Se desvinculó Google de tu cuenta",
            "Tu cuenta ya no entra con Google, solo con Discord.".into(),
            "Quien lo hizo tenía tu cuenta abierta. Entra con Discord, cierra las sesiones que no reconozcas y vuelve a vincular Google en la pestaña Accesos.",
            Remedy::Sessions,
        ),
        Notice::EmailChanged {
            new, has_password, ..
        } => (
            "Se cambió el email de tu cuenta",
            format!(
                "El email de tu cuenta ahora es {new}. Esta dirección ya no recibe los avisos de la cuenta ni el enlace de recuperación de la contraseña."
            ),
            if *has_password {
                "Quien lo hizo sabía tu contraseña. Entra en la cuenta, cambia la contraseña en la pestaña Contraseña (eso cierra las otras sesiones) y vuelve a poner este email en la pestaña Accesos."
            } else {
                "Entra con el Discord o el Google que vinculaste, cierra las sesiones que no reconozcas y vuelve a poner este email en la pestaña Accesos."
            },
            Remedy::Account,
        ),
        Notice::EmailRemoved { .. } => (
            "Se quitó este email de tu cuenta",
            "Tu cuenta, que entra con Discord o con Google, ya no tiene email: esta dirección ya no recibe sus avisos.".into(),
            "Entra con el que vinculaste, cierra las sesiones que no reconozcas y vuelve a registrar este email en la pestaña Accesos.",
            Remedy::Sessions,
        ),
    }
}
