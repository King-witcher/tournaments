//! Senha conferida com limite de tentativas, no login e quando a conta logada
//! mexe nos próprios logins. O limite é por username (exista a conta ou não),
//! não por IP: não depende do proxy, mas quem erra de propósito bloqueia o
//! login com senha do outro por um tempo (Discord e Google continuam
//! entrando). A contagem fica no Redis (`kv::begin_password_attempt`), e cada
//! tentativa conta antes da conferência: pedidos ao mesmo tempo não passam
//! do limite.

use axum::http::StatusCode;
use chrono::{TimeDelta, Utc};

use super::{db, kv, password::verify_password, username::username_key};
use crate::{
    AppState,
    shared::{ApiError, Redis},
};

/// Erros de senha que a janela aceita; o seguinte é recusado sem conferir.
const MAX_PASSWORD_FAILURES: usize = 5;
const FAILURE_WINDOW: TimeDelta = TimeDelta::minutes(15);

/// Confere `password` contra o hash do login `username`. Bloqueado → 429
/// `auth.too_many_attempts` com `{"minutes"}`, sem nem olhar a senha. A
/// tentativa conta antes de conferir e fica como erro; o acerto zera a
/// contagem.
pub async fn check_password(
    redis: &Redis,
    username: &str,
    hash: Option<String>,
    password: String,
) -> Result<bool, ApiError> {
    let key = username_key(username);
    if let Some(until) =
        kv::begin_password_attempt(redis, &key, MAX_PASSWORD_FAILURES, FAILURE_WINDOW).await?
    {
        let minutes = ((until - Utc::now()).num_seconds() + 59) / 60;
        return Err(
            ApiError::new(StatusCode::TOO_MANY_REQUESTS, "auth.too_many_attempts")
                .with_params([("minutes", minutes.max(1))]),
        );
    }

    let ok = verify_password(hash, password).await?;
    if ok {
        kv::clear_password_failures(redis, &key).await?;
    }
    Ok(ok)
}

/// Exige a senha atual da conta logada antes de mudar seus logins: sem senha
/// → 409 `missing`; errada → 403 `auth.wrong_password`. O acerto zera os
/// erros do username, então trocar ou remover o username não deixa erros
/// para quem pegar o nome depois.
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
        &state.redis,
        &login.username,
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
