//! Login com email e senha (`password_credentials`, no máximo um por conta,
//! só com o hash; o email é o da conta): cadastro, criar, trocar, remover e
//! achar pelo email.

use sqlx::{PgConnection, PgPool};

use super::{
    emails::has_email,
    logins::{has_identity_except, lock_user},
};
use crate::{shared::ApiError, shared::Locale};

/// Login com senha de um usuário.
pub struct PasswordLogin {
    pub user_id: i64,
    /// O email da conta, na forma canônica: a chave do limite de tentativas.
    pub email: String,
    pub password_hash: String,
}

/// Conta nova do cadastro com senha, já com o email confirmado.
pub struct NewPasswordUser<'a> {
    pub display_name: &'a str,
    pub email: &'a str,
    pub locale: Locale,
    pub password_hash: &'a str,
}

/// Login com senha pelo email (canônico).
pub async fn password_login_by_email(
    db: &PgPool,
    email: &str,
) -> Result<Option<PasswordLogin>, sqlx::Error> {
    sqlx::query_as!(
        PasswordLogin,
        r#"select p.user_id, u.email as "email!", p.password_hash
           from users u
           join password_credentials p on p.user_id = u.id
           where u.email = $1"#,
        email
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
        r#"select p.user_id, u.email as "email!", p.password_hash
           from password_credentials p
           join users u on u.id = p.user_id
           where p.user_id = $1"#,
        user_id
    )
    .fetch_optional(db)
    .await
}

/// Cria o usuário, com o nome exibido, o email e o login com senha. Email que
/// outra conta pegou desde o pedido → 409 `auth.email_taken`. Devolve o id do
/// usuário.
pub async fn create_password_user(db: &PgPool, user: NewPasswordUser<'_>) -> Result<i64, ApiError> {
    let mut tx = db.begin().await?;
    let user_id = sqlx::query_scalar!(
        "insert into users (display_name, email, locale) values ($1, $2, $3)
         on conflict (email) do nothing
         returning id",
        user.display_name,
        user.email,
        user.locale.as_str()
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::conflict("auth.email_taken"))?;
    insert_password(&mut tx, user_id, user.password_hash).await?;
    tx.commit().await?;
    Ok(user_id)
}

/// Dá senha a uma conta que ainda não tem. Sem email → 409
/// `auth.email_required`: é o login e por ele a senha esquecida volta.
pub async fn add_password(db: &PgPool, user_id: i64, password_hash: &str) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if password_login_exists(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.password_exists"));
    }
    if !has_email(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.email_required"));
    }
    insert_password(&mut tx, user_id, password_hash).await?;
    tx.commit().await?;
    Ok(())
}

/// Troca o hash da senha. Sem senha → 409 `auth.no_password`.
pub async fn update_password(
    db: &PgPool,
    user_id: i64,
    password_hash: &str,
) -> Result<(), ApiError> {
    let done = sqlx::query!(
        "update password_credentials set password_hash = $2 where user_id = $1",
        user_id,
        password_hash
    )
    .execute(db)
    .await?;
    if done.rows_affected() == 0 {
        return Err(ApiError::conflict("auth.no_password"));
    }
    Ok(())
}

/// Tira a senha. Sem senha → 409 `auth.no_password`; sem Discord nem Google →
/// 409 `auth.last_login`.
pub async fn remove_password(db: &PgPool, user_id: i64) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if !password_login_exists(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.no_password"));
    }
    if !has_identity_except(&mut tx, user_id, None).await? {
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

async fn insert_password(
    conn: &mut PgConnection,
    user_id: i64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "insert into password_credentials (user_id, password_hash) values ($1, $2)",
        user_id,
        password_hash
    )
    .execute(conn)
    .await?;
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
