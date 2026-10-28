//! Email da conta (`users.email`): só confirmado e na forma canônica, único.

use sqlx::PgPool;

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
