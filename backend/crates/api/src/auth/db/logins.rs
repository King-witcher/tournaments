//! Formas de login da conta: o Discord (em `user_identities`, a tabela dos
//! provedores externos) e a senha (`passwords.rs`). Ligar e desligar cada uma
//! sem que a conta fique sem nenhuma, nem sem como recuperar o acesso.

use sqlx::{PgConnection, PgPool};

use crate::{auth::types::LoginMethods, shared::ApiError};

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
            if has_discord(&mut tx, user_id).await? {
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

/// Desliga o Discord e tira o avatar que vinha dele. Sem Discord → 409
/// `auth.discord_not_linked`; sem senha → 409 `auth.last_login`; sem email
/// → 409 `auth.email_required`: só com a senha, a conta precisa do email para
/// recuperar o acesso.
pub async fn unlink_discord(db: &PgPool, user_id: i64) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if !has_discord(&mut tx, user_id).await? {
        return Err(ApiError::conflict("auth.discord_not_linked"));
    }
    let left = sqlx::query!(
        r#"select
             exists (select 1 from password_credentials where user_id = $1) as "password!",
             email is not null as "email!"
           from users where id = $1"#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    if !left.password {
        return Err(ApiError::conflict("auth.last_login"));
    }
    if !left.email {
        return Err(ApiError::conflict("auth.email_required"));
    }
    sqlx::query!(
        "delete from user_identities where user_id = $1 and provider = 'discord'",
        user_id
    )
    .execute(&mut *tx)
    .await?;
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
                     where user_id = u.id and provider = 'discord') as "discord!",
             (select username from password_credentials where user_id = u.id) as username,
             u.email
           from users u
           where u.id = $1"#,
        user_id
    )
    .fetch_one(db)
    .await
}

/// Trava a linha do usuário até o fim da transação: mudanças de login da mesma
/// conta passam uma de cada vez.
pub(super) async fn lock_user(conn: &mut PgConnection, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("select id from users where id = $1 for update", user_id)
        .fetch_one(conn)
        .await?;
    Ok(())
}

pub(super) async fn has_discord(
    conn: &mut PgConnection,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists (select 1 from user_identities
                          where user_id = $1 and provider = 'discord') as "exists!""#,
        user_id
    )
    .fetch_one(conn)
    .await
}
