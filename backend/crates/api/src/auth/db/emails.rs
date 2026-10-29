//! Email da conta (`users.email`): só confirmado e na forma canônica, único.
//! A conta com senha sempre tem (o banco recusa o contrário).

use sqlx::{PgConnection, PgPool};

use super::{logins::lock_user, passwords::password_login_exists};
use crate::shared::ApiError;

/// Conta como a recuperação de senha e os avisos a veem.
pub struct AccountContact {
    pub user_id: i64,
    pub email: Option<String>,
    /// Username do login com senha, se a conta tem senha.
    pub username: Option<String>,
    pub locale: String,
}

/// Grava o email confirmado e devolve o de antes (para o aviso), com a conta
/// ter senha ou não. Em uso por outra conta → 409 `auth.email_taken`.
pub async fn set_email(
    db: &PgPool,
    user_id: i64,
    email: &str,
) -> Result<(Option<String>, bool), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    let before = current_email(&mut tx, user_id).await?;
    let has_password = password_login_exists(&mut tx, user_id).await?;
    let result = sqlx::query!("update users set email = $2 where id = $1", user_id, email)
        .execute(&mut *tx)
        .await;
    match result {
        Ok(_) => {}
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            return Err(ApiError::conflict("auth.email_taken"));
        }
        Err(err) => return Err(err.into()),
    }
    tx.commit().await?;
    Ok((before, has_password))
}

/// Tira o email da conta e devolve o que saiu (para o aviso). Sem email → 409
/// `auth.no_email`; com senha → 409 `auth.password_needs_email`: é por ele
/// que a senha esquecida volta.
pub async fn remove_email(db: &PgPool, user_id: i64) -> Result<String, ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    let email = current_email(&mut tx, user_id)
        .await?
        .ok_or(ApiError::conflict("auth.no_email"))?;
    if password_login_exists(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.password_needs_email"));
    }
    sqlx::query!("update users set email = null where id = $1", user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(email)
}

pub(super) async fn has_email(conn: &mut PgConnection, user_id: i64) -> Result<bool, sqlx::Error> {
    Ok(current_email(conn, user_id).await?.is_some())
}

async fn current_email(
    conn: &mut PgConnection,
    user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!("select email from users where id = $1", user_id)
        .fetch_one(conn)
        .await
}

/// A conta, para os avisos. `None` se não existe mais.
pub async fn contact_by_id(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<AccountContact>, sqlx::Error> {
    sqlx::query_as!(
        AccountContact,
        "select u.id as user_id, u.email, p.username as \"username?\", u.locale
         from users u
         left join password_credentials p on p.user_id = u.id
         where u.id = $1",
        user_id
    )
    .fetch_optional(db)
    .await
}

/// Conta deste email (canônico).
pub async fn contact_by_email(
    db: &PgPool,
    email: &str,
) -> Result<Option<AccountContact>, sqlx::Error> {
    sqlx::query_as!(
        AccountContact,
        "select u.id as user_id, u.email, p.username as \"username?\", u.locale
         from users u
         left join password_credentials p on p.user_id = u.id
         where u.email = $1",
        email
    )
    .fetch_optional(db)
    .await
}

/// Conta deste username, sem diferenciar maiúsculas.
pub async fn contact_by_username(
    db: &PgPool,
    username: &str,
) -> Result<Option<AccountContact>, sqlx::Error> {
    sqlx::query_as!(
        AccountContact,
        "select u.id as user_id, u.email, p.username as \"username?\", u.locale
         from password_credentials p
         join users u on u.id = p.user_id
         where lower(p.username) = lower($1)",
        username
    )
    .fetch_optional(db)
    .await
}
