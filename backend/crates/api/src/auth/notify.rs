//! Avisos por email do que muda no acesso à conta (senha, username, Discord,
//! Google, email), para quem não fez a mudança saber a tempo de agir. Saem depois da
//! mudança gravada, em segundo plano: a resposta não espera, e uma falha (do
//! Redis, do provedor) fica no log sem desfazer nada. Contam no teto de
//! emails do dia, mas não na espera entre envios (`Mail::reserve_notice`).
//! Conta sem email não recebe aviso.

use super::{
    db,
    emails::{self, Notice},
};
use crate::{
    AppState,
    shared::{ApiError, Locale},
};

/// Manda `notice` à conta `user_id`: ao email dela ou, nos avisos de email,
/// ao endereço que saiu (`Notice::recipient`).
pub(super) fn notify(state: &AppState, user_id: i64, notice: Notice) {
    let state = state.clone();
    tokio::spawn(async move {
        if let Err(err) = send(&state, user_id, &notice).await {
            tracing::error!(error = ?err, "aviso por email falhou");
        }
    });
}

async fn send(state: &AppState, user_id: i64, notice: &Notice) -> Result<(), ApiError> {
    let Some(contact) = db::contact_by_id(&state.db, user_id).await? else {
        return Ok(());
    };
    let Some(to) = notice.recipient().map(str::to_owned).or(contact.email) else {
        return Ok(());
    };
    let Some(ticket) = state.mail.reserve_notice(&state.redis, &to).await? else {
        return Ok(());
    };
    let locale = Locale::from_db(&contact.locale)?;
    ticket.send(emails::notice(&state.auth.web_origin, locale, &to, notice));
    Ok(())
}
