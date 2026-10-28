//! Email da conta (`users.email`): só confirmado e na forma canônica, único.
//! A conta com senha sempre tem (o banco recusa o contrário).

use sqlx::{PgConnection, PgPool};

use super::{logins::lock_user, passwords::password_login_exists};
use crate::shared::ApiError;

/// Conta como a recuperação de senha a vê.
pub struct AccountContact {
    pub user_id: i64,
    pub email: Option<String>,
    /// Username do login com senha, se a conta tem senha.
    pub username: Option<String>,
    pub locale: String,
}

pub async fn email_taken(db: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists (select 1 from users where email = $1) as "exists!""#,
        email
    )
    .fetch_one(db)
    .await
}

/// Grava o email confirmado. Em uso por outra conta → 409 `auth.email_taken`.
pub async fn set_email(db: &PgPool, user_id: i64, email: &str) -> Result<(), ApiError> {
    let result = sqlx::query!("update users set email = $2 where id = $1", user_id, email)
        .execute(db)
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(ApiError::conflict("auth.email_taken"))
        }
        Err(err) => Err(err.into()),
    }
}

/// Tira o email da conta. Sem email → 409 `auth.no_email`; com senha → 409
/// `auth.password_needs_email`: é por ele que a senha esquecida volta.
pub async fn remove_email(db: &PgPool, user_id: i64) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if !has_email(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.no_email"));
    }
    if password_login_exists(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.password_needs_email"));
    }
    sqlx::query!("update users set email = null where id = $1", user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn has_email(conn: &mut PgConnection, user_id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select email is not null as "email!" from users where id = $1"#,
        user_id
    )
    .fetch_one(conn)
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
