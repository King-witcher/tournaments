//! Excluir a conta. O pedido agenda a exclusão para daqui a 7 dias, encerra
//! todas as sessões (também a do pedido) e manda a data por email, e o
//! agendador lembra por email na véspera; entrar de novo antes cancela
//! (`credentials::start_session`, também a do login pelo Discord ou pelo
//! Google). Na data, o agendador tira a conta das
//! comunidades (`members`) e a anonimiza (`db::anonymize`): os torneios
//! jogados continuam inteiros, com o marcador de conta excluída no lugar do
//! nome.

use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, TimeDelta, Utc};

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

/// Antecedência do lembrete por email, antes da exclusão.
const REMINDER_HOURS: i64 = 24;

#[utoipa::path(
    post,
    path = "/me/deletion",
    tag = "auth",
    request_body = ScheduleAccountDeletion,
    responses(
        (status = OK, body = AccountDeletion, description = "Exclusão agendada para daqui a 7 dias. Todas as sessões da conta foram encerradas, esta também (o cookie sai); entrar de novo antes da data cancela. Se a conta tem email, ele recebe a data. Na data, a conta sai de todas as comunidades (desiste dos torneios em andamento, sai das inscrições dos que não começaram) e é anonimizada: nome e apelidos viram o marcador de conta excluída (vazio), avatar, email, logins e inscrições de push saem, e o email, o Discord e o Google ficam livres"),
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
    if methods.password {
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
            Some(ticket) => ticket.send_in_background(emails::account_deletion_scheduled(
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

/// Manda o lembrete às contas com email cuja exclusão vence nas próximas
/// `REMINDER_HOURS` e devolve quantos saíram. Marca antes de mandar: o
/// lembrete sai no máximo uma vez, e o que não sai (teto do dia, provedor
/// fora) fica no log. O erro de uma conta não para as outras.
pub async fn remind_due_deletions(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<usize, sqlx::Error> {
    let until = now + TimeDelta::hours(REMINDER_HOURS);
    let mut sent = 0;
    for due in db::deletions_to_remind(&state.db, now, until).await? {
        match remind(state, &due, now).await {
            Ok(true) => sent += 1,
            Ok(false) => {}
            Err(e) => tracing::error!(
                user = due.user_id,
                code = e.code,
                "lembrete de exclusão falhou"
            ),
        }
    }
    Ok(sent)
}

async fn remind(
    state: &AppState,
    due: &db::DeletionReminder,
    now: DateTime<Utc>,
) -> Result<bool, ApiError> {
    if !db::mark_deletion_reminded(&state.db, due.user_id, due.due, now).await? {
        return Ok(false);
    }
    let Some(ticket) = state.mail.reserve_notice(&state.redis, &due.email).await? else {
        return Ok(false);
    };
    let locale = Locale::from_db(&due.locale)?;
    let email =
        emails::account_deletion_reminder(&state.auth.web_origin, locale, &due.email, due.due);
    Ok(ticket.send(email).await.is_ok())
}

/// Encerra todas as sessões da conta; os sockets delas fecham.
pub async fn end_all_sessions(state: &AppState, user_id: i64) -> Result<(), ApiError> {
    let ended = kv::delete_all_sessions(&state.redis, user_id).await?;
    sessions_ended(state, user_id, &ended).await
}
