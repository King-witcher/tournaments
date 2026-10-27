//! Senha conferida com limite de tentativas, no login e quando a conta logada
//! mexe nos próprios logins. O limite é por username (exista a conta ou não),
//! não por IP: não depende do proxy, mas quem erra de propósito bloqueia o
//! login com senha do outro por um tempo (o Discord continua entrando).

use axum::http::StatusCode;
use chrono::Utc;
use sqlx::PgPool;

use super::{db, password::verify_password, username::username_key};
use crate::shared::ApiError;

/// Erros de senha que a janela aceita; o seguinte é recusado sem conferir.
const MAX_PASSWORD_FAILURES: i64 = 5;
const FAILURE_WINDOW_MINUTES: i32 = 15;

/// Confere `password` contra o hash do login `username`. Bloqueado → 429
/// `auth.too_many_attempts` com `{"minutes"}`, sem nem olhar a senha. Erro
/// conta uma tentativa; acerto zera a contagem.
pub async fn check_password(
    db: &PgPool,
    username: &str,
    hash: Option<String>,
    password: String,
) -> Result<bool, ApiError> {
    let key = username_key(username);
    if let Some(until) =
        db::password_blocked_until(db, &key, MAX_PASSWORD_FAILURES, FAILURE_WINDOW_MINUTES).await?
    {
        let minutes = ((until - Utc::now()).num_seconds() + 59) / 60;
        return Err(
            ApiError::new(StatusCode::TOO_MANY_REQUESTS, "auth.too_many_attempts")
                .with_params([("minutes", minutes.max(1))]),
        );
    }

    let ok = verify_password(hash, password).await?;
    if ok {
        db::clear_password_failures(db, &key).await?;
    } else {
        db::record_password_failure(db, &key, FAILURE_WINDOW_MINUTES).await?;
    }
    Ok(ok)
}

/// Exige a senha atual da conta logada antes de mudar seus logins: sem senha
/// → 409 `missing`; errada → 403 `auth.wrong_password`.
pub async fn confirm_password(
    db: &PgPool,
    user_id: i64,
    current_password: String,
    missing: &'static str,
) -> Result<(), ApiError> {
    let login = db::password_login_of_user(db, user_id)
        .await?
        .ok_or(ApiError::conflict(missing))?;
    if check_password(
        db,
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
