//! O usuário da conta logada.

use sqlx::PgPool;

use crate::auth::CurrentUser;

/// Usuário de id `id`; `None` se a conta não existe mais ou foi excluída (a
/// sessão dela, no Redis, fica sem efeito até expirar).
pub async fn find_user(db: &PgPool, id: i64) -> Result<Option<CurrentUser>, sqlx::Error> {
    let row = sqlx::query!(
        r#"select id, display_name, avatar_key, avatar_url, locale
           from users where id = $1 and deleted_at is null"#,
        id
    )
    .fetch_optional(db)
    .await?;
    Ok(row
        .map(|r| CurrentUser::from_row(r.id, r.display_name, r.avatar_key, r.avatar_url, r.locale)))
}

pub async fn user_by_id(db: &PgPool, id: i64) -> Result<CurrentUser, sqlx::Error> {
    let r = sqlx::query!(
        "select id, display_name, avatar_key, avatar_url, locale from users where id = $1",
        id
    )
    .fetch_one(db)
    .await?;
    Ok(CurrentUser::from_row(
        r.id,
        r.display_name,
        r.avatar_key,
        r.avatar_url,
        r.locale,
    ))
}

pub async fn set_display_name(db: &PgPool, id: i64, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("update users set display_name = $1 where id = $2", name, id)
        .execute(db)
        .await?;
    Ok(())
}
