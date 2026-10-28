//! Login com Discord (OAuth2) e com username e senha, sessão em cookie e o
//! extractor `CurrentUser`. A conta pode ter os dois logins, ligar e desligar
//! cada um, mas nunca fica sem nenhum, e com senha tem sempre email (a
//! recuperação; o banco também confere). O cadastro com senha exige email e
//! só cria a conta quando o link chega (`signup.rs`); a recuperação de senha
//! (`recovery.rs`) e a troca de email (`email_change.rs`) também vão por link,
//! com os textos em `emails.rs`. Sessões, erros de senha, cadastros por
//! confirmar e links ficam no Redis (`kv/`); contas, logins e email, no banco
//! (`db/`).

mod attempts;
mod credentials;
mod current_user;
mod db;
mod discord;
mod email_change;
mod emails;
mod handlers;
mod kv;
mod oauth;
mod password;
mod recovery;
mod session;
mod signup;
mod types;
mod username;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use current_user::CurrentUser;
pub use discord::{DiscordApi, DiscordUser, HttpDiscord};
/// Os testes criam sessões por aqui, sem passar pelo login.
pub use kv::create_session;
pub use session::{AuthSettings, current_session_id};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(oauth::discord_login))
        .routes(routes!(oauth::discord_link))
        .routes(routes!(oauth::discord_callback))
        .routes(routes!(oauth::unlink_discord))
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
}
