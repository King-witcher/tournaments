//! Login com username e senha (`password_credentials`, no máximo um por
//! conta): cadastro, criar, trocar, remover e achar pelo username.

use sqlx::{PgConnection, PgPool};

use super::{
    emails::has_email,
    logins::{has_discord, lock_user},
};
use crate::{shared::ApiError, shared::Locale};

/// Login com senha de um usuário.
pub struct PasswordLogin {
    pub user_id: i64,
    /// Como foi digitado no cadastro.
    pub username: String,
    pub password_hash: String,
}

/// Conta nova do cadastro com senha, já com o email confirmado.
pub struct NewPasswordUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub locale: Locale,
    pub password_hash: &'a str,
}

/// Login com senha pelo username, sem diferenciar maiúsculas.
pub async fn password_login_by_username(
    db: &PgPool,
    username: &str,
) -> Result<Option<PasswordLogin>, sqlx::Error> {
    sqlx::query_as!(
        PasswordLogin,
        "select user_id, username, password_hash from password_credentials
         where lower(username) = lower($1)",
        username
    )
    .fetch_optional(db)
    .await
}

pub async fn password_login_of_user(
    db: &PgPool,
    user_id: i64,
) -> Result<Option<PasswordLogin>, sqlx::Error> {
    sqlx::query_as!(
        PasswordLogin,
        "select user_id, username, password_hash from password_credentials where user_id = $1",
        user_id
    )
    .fetch_optional(db)
    .await
}

/// O username está em uso por alguma conta (sem diferenciar maiúsculas).
pub async fn username_taken(db: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists (select 1 from password_credentials
                          where lower(username) = lower($1)) as "exists!""#,
        username
    )
    .fetch_one(db)
    .await
}

/// Cria o usuário, com o username como nome exibido, o email e o login com
/// senha. Username ou email que outra conta pegou desde o pedido → 409
/// `auth.username_taken` ou `auth.email_taken`. Devolve o id do usuário.
pub async fn create_password_user(db: &PgPool, user: NewPasswordUser<'_>) -> Result<i64, ApiError> {
    let mut tx = db.begin().await?;
    let user_id = sqlx::query_scalar!(
        "insert into users (display_name, email, locale) values ($1, $2, $3)
         on conflict (email) do nothing
         returning id",
        user.username,
        user.email,
        user.locale.as_str()
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::conflict("auth.email_taken"))?;
    insert_password(&mut tx, user_id, user.username, user.password_hash).await?;
    tx.commit().await?;
    Ok(user_id)
}

/// Dá username e senha a uma conta que ainda não tem. Sem email → 409
/// `auth.email_required`: é por ele que a senha esquecida volta.
pub async fn add_password(
    db: &PgPool,
    user_id: i64,
    username: &str,
    password_hash: &str,
) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if password_login_exists(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.password_exists"));
    }
    if !has_email(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.email_required"));
    }
    insert_password(&mut tx, user_id, username, password_hash).await?;
    tx.commit().await?;
    Ok(())
}

/// Troca username e/ou hash da senha; `None` mantém o atual.
pub async fn update_password(
    db: &PgPool,
    user_id: i64,
    username: Option<&str>,
    password_hash: Option<&str>,
) -> Result<(), ApiError> {
    let result = sqlx::query!(
        "update password_credentials
         set username = coalesce($2, username),
             password_hash = coalesce($3, password_hash)
         where user_id = $1",
        user_id,
        username,
        password_hash
    )
    .execute(db)
    .await;
    match result {
        Ok(done) if done.rows_affected() == 0 => Err(ApiError::conflict("auth.no_password")),
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(ApiError::conflict("auth.username_taken"))
        }
        Err(err) => Err(err.into()),
    }
}

/// Tira username e senha. Sem senha → 409 `auth.no_password`; sem Discord →
/// 409 `auth.last_login`.
pub async fn remove_password(db: &PgPool, user_id: i64) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if !password_login_exists(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.no_password"));
    }
    if !has_discord(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.last_login"));
    }
    sqlx::query!(
        "delete from password_credentials where user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Insere o login com senha; username em uso (sem diferenciar maiúsculas) → 409.
async fn insert_password(
    conn: &mut PgConnection,
    user_id: i64,
    username: &str,
    password_hash: &str,
) -> Result<(), ApiError> {
    sqlx::query_scalar!(
        "insert into password_credentials (user_id, username, password_hash)
         values ($1, $2, $3)
         on conflict do nothing
         returning user_id",
        user_id,
        username,
        password_hash
    )
    .fetch_optional(conn)
    .await?
    .ok_or(ApiError::conflict("auth.username_taken"))?;
    Ok(())
}

pub(super) async fn password_login_exists(
    conn: &mut PgConnection,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists (select 1 from password_credentials where user_id = $1) as "exists!""#,
        user_id
    )
    .fetch_one(conn)
    .await
}
