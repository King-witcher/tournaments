//! Login com Discord e com Google (OAuth2), com email e senha e só com o email
//! (`email_login.rs`, que também recupera a senha), sessão em cookie e o
//! extractor `CurrentUser`. Email confirmado é um jeito de entrar: a conta liga
//! e desliga cada login, mas nunca fica sem nenhum, e com senha tem sempre email
//! (o banco também confere). Os provedores (`provider.rs`)
//! dividem o fluxo de `oauth.rs`, com PKCE gerado no navegador (o formato do
//! state, do verifier e do challenge em `pkce.rs`); o que é de cada um fica em
//! `discord.rs` e `google.rs`. O cadastro com senha exige email e só cria a
//! conta quando o link é aberto ou o código do mesmo email é digitado
//! (`signup.rs`); entrar por email e a troca de email (`email_change.rs`) também
//! aceitam os dois, com os textos em `emails/`. Na conta que só entra pelo email,
//! mudar os logins pede antes o código mandado a ele (`email_proof.rs`). O que
//! manda email por pedido da pessoa conta por IP e, com login, por conta
//! (`Mail::limit_ip`, `Mail::limit_account`). Toda mudança de
//! login ou de email manda um aviso ao email da conta (`notify.rs`). Sessões,
//! erros de senha, cadastros por confirmar, links e códigos ficam no Redis
//! (`kv/`); contas, logins e email, no banco (`db/`).
//! Excluir a conta (`deletion.rs`) agenda para 7 dias, e entrar de novo antes
//! (por qualquer login) cancela; na data, `members` a tira
//! das comunidades e daqui sai a anonimização.

mod attempts;
mod credentials;
mod current_user;
mod db;
mod deletion;
mod discord;
mod email_change;
mod email_login;
mod email_proof;
mod emails;
mod google;
mod handlers;
mod kv;
mod notify;
mod oauth;
mod password;
mod pkce;
mod provider;
mod session;
mod signup;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use current_user::CurrentUser;
pub use db::{
    anonymize as anonymize_account, due_deletions as due_account_deletions,
    lock_due_deletion as lock_due_account_deletion,
};
pub use deletion::{end_all_sessions, remind_due_deletions as remind_due_account_deletions};
pub use discord::{DiscordApi, DiscordLimit, DiscordUser, HttpDiscord, discord_gate};
pub use google::{GoogleApi, GoogleUser, HttpGoogle};
/// Os testes criam sessões por aqui, sem passar pelo login.
pub use kv::create_session;
pub use session::{AuthSettings, OAuthClient};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(oauth::discord_authorize))
        .routes(routes!(oauth::discord_login))
        .routes(routes!(oauth::discord_link))
        .routes(routes!(oauth::unlink_discord))
        .routes(routes!(oauth::google_authorize))
        .routes(routes!(oauth::google_login))
        .routes(routes!(oauth::google_link))
        .routes(routes!(oauth::unlink_google))
        .routes(routes!(signup::signup))
        .routes(routes!(signup::confirm_signup))
        .routes(routes!(signup::confirm_signup_code))
        .routes(routes!(credentials::login))
        .routes(routes!(
            credentials::create_password,
            credentials::change_password,
            credentials::remove_password
        ))
        .routes(routes!(email_login::request_email_login))
        .routes(routes!(email_login::email_login_by_link))
        .routes(routes!(email_login::email_login_by_code))
        .routes(routes!(
            email_change::change_email,
            email_change::remove_email
        ))
        .routes(routes!(email_change::confirm_email))
        .routes(routes!(email_change::confirm_email_by_code))
        .routes(routes!(
            email_proof::email_proof_status,
            email_proof::request_email_proof
        ))
        .routes(routes!(email_proof::confirm_email_proof))
        .routes(routes!(handlers::me, handlers::update_me))
        .routes(routes!(handlers::logins))
        .routes(routes!(handlers::logout))
        .routes(routes!(handlers::clear_expired_session))
        .routes(routes!(handlers::sessions, handlers::end_other_sessions))
        .routes(routes!(handlers::end_session))
        .routes(routes!(deletion::schedule_account_deletion))
}
