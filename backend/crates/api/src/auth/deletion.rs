//! Excluir a conta. O pedido agenda a exclusão para daqui a 7 dias, encerra
//! todas as sessões (também a do pedido) e manda a data por email; entrar de
//! novo antes cancela (`credentials::start_session` e o login pelo Discord ou
//! pelo Google, em `oauth.rs`). Na data, o agendador tira a conta das
//! comunidades (`members`) e a anonimiza (`db::anonymize`): os torneios
//! jogados continuam inteiros, com o marcador de conta excluída no lugar do
//! nome.

use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;

use super::{
    CurrentUser,
    attempts::confirm_password,
    db, emails,
    handlers::sessions_ended,
    kv,
    types::{AccountDeletion, ScheduleAccountDeletion},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, Locale},
};

/// Dias entre o pedido e a exclusão da conta.
const DELETION_DAYS: i32 = 7;

/// Para onde o login que cancelou a exclusão leva (o front avisa; sem o
/// idioma, que o proxy do front põe).
pub(super) const CANCELLED_PATH: &str = "/account?account_deletion=cancelled";

#[utoipa::path(
    post,
    path = "/me/deletion",
    tag = "auth",
    request_body = ScheduleAccountDeletion,
    responses(
        (status = OK, body = AccountDeletion, description = "Exclusão agendada para daqui a 7 dias. Todas as sessões da conta foram encerradas, esta também (o cookie sai); entrar de novo antes da data cancela. Se a conta tem email, ele recebe a data. Na data, a conta sai de todas as comunidades (desiste dos torneios em andamento, sai das inscrições dos que não começaram, os convites dela são revogados) e é anonimizada: nome e apelidos viram o marcador de conta excluída (vazio), avatar, email, logins e inscrições de push saem, e o username, o Discord e o Google ficam livres"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "auth.wrong_password (a conta tem senha, e a atual não veio ou não confere)"),
        (status = CONFLICT, body = ErrorBody, description = "auth.owns_tenants (owner de alguma comunidade: transfere a posse ou exclui a comunidade antes)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "auth.too_many_attempts {minutes}"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn schedule_account_deletion(
    State(state): State<AppState>,
    user: CurrentUser,
    jar: CookieJar,
    JsonBody(body): JsonBody<ScheduleAccountDeletion>,
) -> Result<(CookieJar, Json<AccountDeletion>), ApiError> {
    let methods = db::login_methods(&state.db, user.id).await?;
    // Com senha, quem pegou a sessão não exclui a conta.
    if methods.username.is_some() {
        let current = body
            .current_password
            .ok_or(ApiError::forbidden("auth.wrong_password"))?;
        confirm_password(&state, user.id, current, "auth.no_password").await?;
    }
    let due = db::schedule_deletion(&state.db, user.id, DELETION_DAYS).await?;
    end_all_sessions(&state, user.id).await?;

    if let Some(email) = methods.email {
        // Fora dos limites de envio, a exclusão vale do mesmo jeito.
        match state.mail.reserve_quietly(&state.redis, &email).await? {
            Some(ticket) => ticket.send(emails::account_deletion_scheduled(
                &state.auth.web_origin,
                Locale::from_db(&user.locale)?,
                &email,
                due,
            )),
            None => tracing::warn!(user = user.id, "aviso de exclusão da conta não saiu"),
        }
    }
    Ok((
        jar.remove(state.auth.clear_session_cookie()),
        Json(AccountDeletion { scheduled_for: due }),
    ))
}

/// Encerra todas as sessões da conta; os sockets delas fecham.
pub async fn end_all_sessions(state: &AppState, user_id: i64) -> Result<(), ApiError> {
    let ended = kv::delete_all_sessions(&state.redis, user_id).await?;
    sessions_ended(state, user_id, &ended).await
}
