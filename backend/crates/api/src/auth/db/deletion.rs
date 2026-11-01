//! Exclusão da conta: agendar, cancelar (entrar de novo) e, na data,
//! anonimizar. Quem tira a conta das comunidades antes é `members`.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::shared::ApiError;

/// Agenda a exclusão da conta para daqui a `days` dias e devolve a data.
/// Owner de alguma comunidade → 409 `auth.owns_tenants`: a regra é do banco
/// (`users_deletion_owner`), que confere com a linha do usuário travada, como
/// criar comunidade e transferir a posse.
pub async fn schedule_deletion(
    db: &PgPool,
    user_id: i64,
    days: i32,
) -> Result<DateTime<Utc>, ApiError> {
    let result = sqlx::query_scalar!(
        r#"update users set deletion_due_at = now() + make_interval(days => $2)
           where id = $1
           returning deletion_due_at as "due!""#,
        user_id,
        days
    )
    .fetch_one(db)
    .await;
    match result {
        Ok(due) => Ok(due),
        Err(sqlx::Error::Database(err)) if err.constraint() == Some("users_deletion_owner") => {
            Err(ApiError::conflict("auth.owns_tenants"))
        }
        Err(err) => Err(err.into()),
    }
}

/// Entrar de novo cancela a exclusão agendada. `true` se havia uma.
pub async fn cancel_deletion(db: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"update users set deletion_due_at = null, deletion_reminded_at = null
           where id = $1 and deletion_due_at is not null"#,
        user_id
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Uma conta que recebe o lembrete da exclusão.
pub struct DeletionReminder {
    pub user_id: i64,
    pub email: String,
    pub locale: String,
    pub due: DateTime<Utc>,
}

/// Contas com email cuja exclusão vence entre `now` e `until`, sem o
/// lembrete, para o agendador.
pub async fn deletions_to_remind(
    db: &PgPool,
    now: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<Vec<DeletionReminder>, sqlx::Error> {
    sqlx::query_as!(
        DeletionReminder,
        r#"select id as user_id, email as "email!", locale, deletion_due_at as "due!"
           from users
           where deletion_due_at > $1 and deletion_due_at <= $2
             and deletion_reminded_at is null and email is not null
           order by id"#,
        now,
        until
    )
    .fetch_all(db)
    .await
}

/// Marca o lembrete da exclusão que vence em `due`, se ainda vale (ninguém
/// entrou nem marcou desde a busca).
pub async fn mark_deletion_reminded(
    db: &PgPool,
    user_id: i64,
    due: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"update users set deletion_reminded_at = $3
           where id = $1 and deletion_due_at = $2 and deletion_reminded_at is null"#,
        user_id,
        due,
        now
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Contas cuja exclusão venceu em `now`, para o agendador.
pub async fn due_deletions(db: &PgPool, now: DateTime<Utc>) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar!(
        "select id from users where deletion_due_at <= $1 order by id",
        now
    )
    .fetch_all(db)
    .await
}

/// Trava a conta até o fim da transação, se a exclusão dela ainda vence em
/// `now` (ninguém entrou desde a busca). `false`: foi cancelada.
pub async fn lock_due_deletion(
    conn: &mut PgConnection,
    user_id: i64,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        "select id from users where id = $1 and deletion_due_at <= $2 for update",
        user_id,
        now
    )
    .fetch_optional(conn)
    .await?
    .is_some())
}

/// Anonimiza a conta: sem logins (o email, o Discord e o Google ficam livres
/// para outra conta), sem nome (o vazio é o marcador de conta excluída), sem
/// avatar e sem email. Quem chama já a tirou das comunidades, trocou os
/// apelidos dela e tirou as inscrições de push: o banco confere tudo no
/// commit.
pub async fn anonymize(conn: &mut PgConnection, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from user_identities where user_id = $1", user_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!(
        "delete from password_credentials where user_id = $1",
        user_id
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        r#"update users
           set display_name = '', avatar_url = null, email = null,
               deletion_due_at = null, deletion_reminded_at = null, deleted_at = now()
           where id = $1"#,
        user_id
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
