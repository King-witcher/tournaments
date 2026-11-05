//! Senha conferida com limite de tentativas, no login e quando a conta logada
//! mexe nos próprios logins. O limite é por email, na forma canônica (exista
//! a conta ou não), não por IP: não depende do proxy, mas quem sabe o email
//! de outro e erra de propósito bloqueia o login com senha dele por um tempo
//! (Discord e Google continuam entrando). A contagem fica em memória
//! (`AppState::password_attempts`): um deploy a zera, e quem insiste com emails
//! inventados não gasta a cota do Redis. Cada tentativa conta antes da
//! conferência: pedidos ao mesmo tempo não passam do limite.

use std::time::Duration;

use axum::http::StatusCode;

use super::{db, password::verify_password};
use crate::{
    AppState,
    shared::{ApiError, WindowLimit},
};

/// Erros de senha que a janela aceita; o seguinte é recusado sem conferir.
const MAX_PASSWORD_FAILURES: u32 = 5;
/// Começa no primeiro erro.
const FAILURE_WINDOW: Duration = Duration::from_secs(15 * 60);

/// Confere `password` contra o hash do login de `email` (canônico).
/// Bloqueado → 429 `auth.too_many_attempts` com `{"minutes"}`, sem nem olhar
/// a senha. A tentativa conta antes de conferir e fica como erro; o acerto
/// zera a contagem.
pub async fn check_password(
    attempts: &WindowLimit,
    email: &str,
    hash: Option<String>,
    password: String,
) -> Result<bool, ApiError> {
    if let Err(wait) = attempts.hit(email.to_owned(), MAX_PASSWORD_FAILURES, FAILURE_WINDOW) {
        let minutes = (wait.as_secs() as i64 + 59) / 60;
        return Err(
            ApiError::new(StatusCode::TOO_MANY_REQUESTS, "auth.too_many_attempts")
                .with_params([("minutes", minutes.max(1))]),
        );
    }

    let ok = verify_password(hash, password).await?;
    if ok {
        attempts.forget(email);
    }
    Ok(ok)
}

/// Exige a senha atual da conta logada antes de mudar seus logins: sem senha
/// → 409 `missing`; errada → 403 `auth.wrong_password`. Conta no limite do
/// email da conta, como o login.
pub async fn confirm_password(
    state: &AppState,
    user_id: i64,
    current_password: String,
    missing: &'static str,
) -> Result<(), ApiError> {
    let login = db::password_login_of_user(&state.db, user_id)
        .await?
        .ok_or(ApiError::conflict(missing))?;
    confirm_login(state, login, current_password).await
}

/// Como [`confirm_password`], mas a conta sem senha passa sem conferir nada
/// (quem tem só a sessão já pode tudo nela). Com senha, a atual que não veio
/// conta como errada.
pub async fn confirm_password_if_set(
    state: &AppState,
    user_id: i64,
    current_password: Option<String>,
) -> Result<(), ApiError> {
    let Some(login) = db::password_login_of_user(&state.db, user_id).await? else {
        return Ok(());
    };
    let current = current_password.ok_or(ApiError::forbidden("auth.wrong_password"))?;
    confirm_login(state, login, current).await
}

async fn confirm_login(
    state: &AppState,
    login: db::PasswordLogin,
    current_password: String,
) -> Result<(), ApiError> {
    if check_password(
        &state.password_attempts,
        &login.email,
        Some(login.password_hash),
        current_password,
    )
    .await?
    {
        Ok(())
    } else {
        Err(ApiError::forbidden("auth.wrong_password"))
    }
}
