//! Login com Discord e com Google (OAuth2) e com username e senha, sessão em
//! cookie e o extractor `CurrentUser`. A conta pode ter os três logins, ligar
//! e desligar cada um, mas nunca fica sem nenhum, e com senha tem sempre email
//! (a recuperação; o banco também confere). Os provedores (`provider.rs`)
//! dividem o fluxo de `oauth.rs`; o que é de cada um fica em `discord.rs` e
//! `google.rs`. O cadastro com senha exige email e só cria a conta quando o
//! link chega (`signup.rs`); a recuperação de senha (`recovery.rs`) e a troca
//! de email (`email_change.rs`) também vão por link, com os textos em
//! `emails/`. Toda mudança de login ou de email manda um aviso ao email da
//! conta (`notify.rs`). Sessões, erros de senha, cadastros por confirmar e
//! links ficam no Redis (`kv/`); contas, logins e email, no banco (`db/`).
//! Excluir a conta (`deletion.rs`) agenda para 7 dias, e entrar de novo antes
//! (pela senha ou por qualquer provedor) cancela; na data, `members` a tira
//! das comunidades e daqui sai a anonimização.

mod attempts;
mod credentials;
mod current_user;
mod db;
mod deletion;
mod discord;
mod email_change;
mod emails;
mod google;
mod handlers;
mod kv;
mod notify;
mod oauth;
mod password;
mod provider;
mod recovery;
mod session;
mod signup;
mod types;
mod username;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use current_user::CurrentUser;
pub use db::{
    anonymize as anonymize_account, due_deletions as due_account_deletions,
    lock_due_deletion as lock_due_account_deletion,
};
pub use deletion::end_all_sessions;
pub use discord::{DiscordApi, DiscordUser, HttpDiscord};
pub use google::{GoogleApi, GoogleUser, HttpGoogle};
/// Os testes criam sessões por aqui, sem passar pelo login.
pub use kv::create_session;
pub use session::{AuthSettings, OAuthClient, current_session_id};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(oauth::discord_login))
        .routes(routes!(oauth::discord_link))
        .routes(routes!(oauth::discord_callback))
        .routes(routes!(oauth::unlink_discord))
        .routes(routes!(oauth::google_login))
        .routes(routes!(oauth::google_link))
        .routes(routes!(oauth::google_callback))
        .routes(routes!(oauth::unlink_google))
        .routes(routes!(signup::signup))
        .routes(routes!(signup::confirm_signup))
        .routes(routes!(credentials::login))
        .routes(routes!(
            credentials::create_password,
            credentials::change_password,
            credentials::remove_password
        ))
        .routes(routes!(recovery::request_password_reset))
        .routes(routes!(recovery::reset_password))
        .routes(routes!(
            email_change::change_email,
            email_change::remove_email
        ))
        .routes(routes!(email_change::confirm_email))
        .routes(routes!(handlers::me, handlers::update_me))
        .routes(routes!(handlers::logins))
        .routes(routes!(handlers::logout))
        .routes(routes!(handlers::sessions, handlers::end_other_sessions))
        .routes(routes!(handlers::end_session))
        .routes(routes!(deletion::schedule_account_deletion))
}
