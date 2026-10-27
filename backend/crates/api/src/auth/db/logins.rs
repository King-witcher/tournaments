//! Formas de login da conta (Discord e senha), ligadas e desligadas sem que
//! a conta fique sem nenhuma.

use sqlx::{PgConnection, PgPool};

use crate::{auth::types::LoginMethods, shared::ApiError};

/// Login com senha de um usuário.
pub struct PasswordLogin {
    pub user_id: i64,
    /// Como foi digitado no cadastro.
    pub username: String,
    pub password_hash: String,
}

/// Acha o usuário pela identidade do Discord (atualizando o avatar) ou cria
/// usuário e identidade. Devolve o id do usuário.
pub async fn upsert_discord_user(
    db: &PgPool,
    discord_id: &str,
    display_name: &str,
    avatar_url: Option<String>,
) -> Result<i64, sqlx::Error> {
    let mut tx = db.begin().await?;
    let existing = sqlx::query_scalar!(
        "select user_id from user_identities where provider = 'discord' and provider_user_id = $1",
        discord_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let user_id = match existing {
        Some(id) => {
            sqlx::query!(
                "update users set avatar_url = $1 where id = $2",
                avatar_url,
                id
            )
            .execute(&mut *tx)
            .await?;
            id
        }
        None => {
            let id = sqlx::query_scalar!(
                "insert into users (display_name, avatar_url) values ($1, $2) returning id",
                display_name,
                avatar_url
            )
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query!(
                "insert into user_identities (user_id, provider, provider_user_id) values ($1, 'discord', $2)",
                id,
                discord_id
            )
            .execute(&mut *tx)
            .await?;
            id
        }
    };
    tx.commit().await?;
    Ok(user_id)
}

/// Liga o Discord à conta e passa a usar o avatar dele. Ligar de novo o mesmo
/// Discord só atualiza o avatar.
pub async fn link_discord(
    db: &PgPool,
    user_id: i64,
    discord_id: &str,
    avatar_url: Option<String>,
) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    let owner = sqlx::query_scalar!(
        "select user_id from user_identities where provider = 'discord' and provider_user_id = $1",
        discord_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    match owner {
        Some(id) if id == user_id => {}
        Some(_) => return Err(ApiError::conflict("auth.discord_taken")),
        None => {
            if has_login(&mut tx, user_id, "discord").await? {
                return Err(ApiError::conflict("auth.discord_already_linked"));
            }
            // Conflito aqui só se outra conta ligou este Discord agora há pouco.
            sqlx::query_scalar!(
                r#"insert into user_identities (user_id, provider, provider_user_id)
                   values ($1, 'discord', $2)
                   on conflict do nothing
                   returning user_id"#,
                user_id,
                discord_id
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(ApiError::conflict("auth.discord_taken"))?;
        }
    }

    sqlx::query!(
        "update users set avatar_url = $1 where id = $2",
        avatar_url,
        user_id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Desliga o Discord, se a conta tiver outro login, e tira o avatar que vinha dele.
pub async fn unlink_discord(db: &PgPool, user_id: i64) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    remove_login(&mut tx, user_id, "discord", "auth.discord_not_linked").await?;
    sqlx::query!("update users set avatar_url = null where id = $1", user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn login_methods(db: &PgPool, user_id: i64) -> Result<LoginMethods, sqlx::Error> {
    sqlx::query_as!(
        LoginMethods,
        r#"select
             exists (select 1 from user_identities
                     where user_id = $1 and provider = 'discord') as "discord!",
             (select provider_user_id from user_identities
              where user_id = $1 and provider = 'password') as username"#,
        user_id
    )
    .fetch_one(db)
    .await
}

/// Login com senha pelo username, sem diferenciar maiúsculas.
pub async fn password_login_by_username(
    db: &PgPool,
    username: &str,
) -> Result<Option<PasswordLogin>, sqlx::Error> {
    sqlx::query_as!(
        PasswordLogin,
        r#"select user_id, provider_user_id as username, password_hash as "password_hash!"
           from user_identities
           where provider = 'password' and lower(provider_user_id) = lower($1)"#,
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
        r#"select user_id, provider_user_id as username, password_hash as "password_hash!"
           from user_identities
           where provider = 'password' and user_id = $1"#,
        user_id
    )
    .fetch_optional(db)
    .await
}

/// Cria o usuário, com o username como nome exibido, e o login com senha.
/// Devolve o id do usuário.
pub async fn create_password_user(
    db: &PgPool,
    username: &str,
    password_hash: &str,
) -> Result<i64, ApiError> {
    let mut tx = db.begin().await?;
    let user_id = sqlx::query_scalar!(
        "insert into users (display_name) values ($1) returning id",
        username
    )
    .fetch_one(&mut *tx)
    .await?;
    insert_password(&mut tx, user_id, username, password_hash).await?;
    tx.commit().await?;
    Ok(user_id)
}

/// Dá username e senha a uma conta que ainda não tem.
pub async fn add_password(
    db: &PgPool,
    user_id: i64,
    username: &str,
    password_hash: &str,
) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if has_login(&mut tx, user_id, "password").await? {
        return Err(ApiError::conflict("auth.password_exists"));
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
        r#"update user_identities
           set provider_user_id = coalesce($2, provider_user_id),
               password_hash = coalesce($3, password_hash)
           where user_id = $1 and provider = 'password'"#,
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

/// Tira username e senha, se a conta tiver outro login.
pub async fn remove_password(db: &PgPool, user_id: i64) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    remove_login(&mut tx, user_id, "password", "auth.no_password").await?;
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
        r#"insert into user_identities (user_id, provider, provider_user_id, password_hash)
           values ($1, 'password', $2, $3)
           on conflict do nothing
           returning user_id"#,
        user_id,
        username,
        password_hash
    )
    .fetch_optional(conn)
    .await?
    .ok_or(ApiError::conflict("auth.username_taken"))?;
    Ok(())
}

/// Remove o login `provider` da conta. Sem ele → 409 `missing`; se for o
/// único → 409 `auth.last_login`. Trava o usuário, para dois pedidos ao mesmo
/// tempo não removerem os dois logins.
async fn remove_login(
    conn: &mut PgConnection,
    user_id: i64,
    provider: &str,
    missing: &'static str,
) -> Result<(), ApiError> {
    lock_user(conn, user_id).await?;
    if !has_login(conn, user_id, provider).await? {
        return Err(ApiError::conflict(missing));
    }
    let others = sqlx::query_scalar!(
        r#"select count(*) as "n!" from user_identities where user_id = $1 and provider <> $2"#,
        user_id,
        provider
    )
    .fetch_one(&mut *conn)
    .await?;
    if others == 0 {
        return Err(ApiError::conflict("auth.last_login"));
    }
    sqlx::query!(
        "delete from user_identities where user_id = $1 and provider = $2",
        user_id,
        provider
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Trava a linha do usuário até o fim da transação: mudanças de login da mesma
/// conta passam uma de cada vez.
async fn lock_user(conn: &mut PgConnection, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("select id from users where id = $1 for update", user_id)
        .fetch_one(conn)
        .await?;
    Ok(())
}

async fn has_login(
    conn: &mut PgConnection,
    user_id: i64,
    provider: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists (select 1 from user_identities where user_id = $1 and provider = $2) as "exists!""#,
        user_id,
        provider
    )
    .fetch_one(conn)
    .await
}
