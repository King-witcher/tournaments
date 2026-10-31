//! Formas de login da conta: os provedores externos (Discord e Google, em
//! `user_identities`) e a senha (`passwords.rs`). Ligar e desligar cada uma
//! sem que a conta fique sem nenhuma, nem sem como recuperar o acesso.

use sqlx::{PgConnection, PgPool};

use super::passwords::password_login_exists;
use crate::{
    auth::{provider::Provider, types::LoginMethods},
    shared::ApiError,
};

/// Quem um provedor diz que entrou.
pub struct ExternalLogin {
    pub provider: Provider,
    /// O id da pessoa no provedor (`provider_user_id`).
    pub id: String,
    /// Nome exibido da conta, se o login a criar: entrar numa conta que já
    /// existe não muda o nome.
    pub display_name: String,
    /// O avatar no provedor; só vale para o que `sets_avatar`.
    pub avatar_url: Option<String>,
    /// Como a pessoa se reconhece no provedor, para o aviso de vínculo: o
    /// username no Discord, o nome no Google.
    pub account: String,
}

/// Acha o usuário pela identidade do provedor (atualizando o avatar, se é o
/// Discord) ou cria usuário e identidade. Devolve o id do usuário.
pub async fn upsert_external_user(db: &PgPool, login: &ExternalLogin) -> Result<i64, sqlx::Error> {
    let provider = login.provider.as_str();
    let avatar_url = login
        .avatar_url
        .as_deref()
        .filter(|_| login.provider.sets_avatar());
    let mut tx = db.begin().await?;
    let existing = sqlx::query_scalar!(
        "select user_id from user_identities where provider = $1 and provider_user_id = $2",
        provider,
        login.id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let user_id = match existing {
        Some(id) => {
            if login.provider.sets_avatar() {
                set_avatar(&mut tx, id, avatar_url).await?;
            }
            id
        }
        None => {
            let id = sqlx::query_scalar!(
                "insert into users (display_name, avatar_url) values ($1, $2) returning id",
                login.display_name,
                avatar_url
            )
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query!(
                "insert into user_identities (user_id, provider, provider_user_id) values ($1, $2, $3)",
                id,
                provider,
                login.id
            )
            .execute(&mut *tx)
            .await?;
            id
        }
    };
    tx.commit().await?;
    Ok(user_id)
}

/// Liga o provedor à conta e, se é o Discord, passa a usar o avatar dele.
/// Ligar de novo a mesma identidade só atualiza o avatar. Identidade de outra
/// conta → 409 `taken`; a conta já com outra desse provedor → 409
/// `already_linked` (os códigos do provedor). Devolve se ligou agora (falso
/// no de novo).
pub async fn link_external(
    db: &PgPool,
    user_id: i64,
    login: &ExternalLogin,
) -> Result<bool, ApiError> {
    let codes = login.provider.codes();
    let provider = login.provider.as_str();
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    let owner = sqlx::query_scalar!(
        "select user_id from user_identities where provider = $1 and provider_user_id = $2",
        provider,
        login.id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let linked = owner.is_none();
    match owner {
        Some(id) if id == user_id => {}
        Some(_) => return Err(ApiError::conflict(codes.taken)),
        None => {
            if has_identity(&mut tx, user_id, login.provider).await? {
                return Err(ApiError::conflict(codes.already_linked));
            }
            // Conflito aqui só se outra conta ligou a mesma identidade agora há pouco.
            sqlx::query_scalar!(
                r#"insert into user_identities (user_id, provider, provider_user_id)
                   values ($1, $2, $3)
                   on conflict do nothing
                   returning user_id"#,
                user_id,
                provider,
                login.id
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(ApiError::conflict(codes.taken))?;
        }
    }

    if login.provider.sets_avatar() {
        set_avatar(&mut tx, user_id, login.avatar_url.as_deref()).await?;
    }
    tx.commit().await?;
    Ok(linked)
}

/// Desliga o provedor e, se é o Discord, tira o avatar que vinha dele. Sem o
/// provedor → 409 `not_linked` (o código dele); sem outro login (senha ou
/// outro provedor) → 409 `auth.last_login`. A senha que fica sempre tem o
/// email da conta para recuperá-la (regra do banco).
pub async fn unlink_external(
    db: &PgPool,
    user_id: i64,
    provider: Provider,
) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;
    lock_user(&mut tx, user_id).await?;
    if !has_identity(&mut tx, user_id, provider).await? {
        return Err(ApiError::conflict(provider.codes().not_linked));
    }
    if !password_login_exists(&mut tx, user_id).await?
        && !has_identity_except(&mut tx, user_id, Some(provider)).await?
    {
        return Err(ApiError::conflict("auth.last_login"));
    }
    sqlx::query!(
        "delete from user_identities where user_id = $1 and provider = $2",
        user_id,
        provider.as_str()
    )
    .execute(&mut *tx)
    .await?;
    if provider.sets_avatar() {
        set_avatar(&mut tx, user_id, None).await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn login_methods(db: &PgPool, user_id: i64) -> Result<LoginMethods, sqlx::Error> {
    sqlx::query_as!(
        LoginMethods,
        r#"select
             exists (select 1 from user_identities
                     where user_id = u.id and provider = 'discord') as "discord!",
             exists (select 1 from user_identities
                     where user_id = u.id and provider = 'google') as "google!",
             exists (select 1 from password_credentials
                     where user_id = u.id) as "password!",
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

async fn has_identity(
    conn: &mut PgConnection,
    user_id: i64,
    provider: Provider,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists (select 1 from user_identities
                          where user_id = $1 and provider = $2) as "exists!""#,
        user_id,
        provider.as_str()
    )
    .fetch_one(conn)
    .await
}

/// A conta entra por algum provedor além de `except` (`None`: qualquer um).
pub(super) async fn has_identity_except(
    conn: &mut PgConnection,
    user_id: i64,
    except: Option<Provider>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists (select 1 from user_identities
                          where user_id = $1 and provider is distinct from $2) as "exists!""#,
        user_id,
        except.map(Provider::as_str)
    )
    .fetch_one(conn)
    .await
}

async fn set_avatar(
    conn: &mut PgConnection,
    user_id: i64,
    avatar_url: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update users set avatar_url = $1 where id = $2",
        avatar_url,
        user_id
    )
    .execute(conn)
    .await?;
    Ok(())
}
