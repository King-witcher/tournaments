//! SQL do que o agendador do tempo real faz nos torneios sem ninguém pedir:
//! achar os que têm o check-in para abrir e marcar o aviso de hora de
//! iniciar. A abertura em si é a mesma da leitura (`read::current_status`).
//! Na comunidade com a exclusão agendada (só leitura), nenhum dos dois.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::TournamentId;
use crate::{
    shared::ApiError,
    tenants::TenantId,
    tournaments::{schedule::Schedule, status::TournamentStatus},
};

/// Um torneio na inscrição cuja janela de check-in já abriu em `now`, com o
/// que `current_status` confere de novo antes de abrir.
pub struct DueCheckIn {
    pub id: TournamentId,
    pub status: TournamentStatus,
    pub schedule: Schedule,
}

/// Torneios na inscrição com a janela aberta em `now` (`Schedule::check_in_due`
/// confere de novo).
pub async fn due_check_ins(db: &PgPool, now: DateTime<Utc>) -> Result<Vec<DueCheckIn>, ApiError> {
    sqlx::query!(
        r#"select id, status, starts_at, check_in_minutes
           from tournaments
           where status = 'registration'
             and starts_at - make_interval(mins => check_in_minutes) <= $1
             and not exists (select 1 from tenants n
                             where n.id = tournaments.tenant_id and n.deletion_due_at is not null)
           order by id"#,
        now
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| {
        Ok(DueCheckIn {
            id: TournamentId(r.id),
            status: TournamentStatus::from_db(&r.status)?,
            schedule: Schedule {
                starts_at: r.starts_at,
                check_in_minutes: r.check_in_minutes,
            },
        })
    })
    .collect()
}

/// Torneios publicados e não iniciados cujo início chegou em `now` sem aviso
/// (ou com o aviso de antes de o início ser adiado).
pub async fn starts_to_remind(
    db: &PgPool,
    now: DateTime<Utc>,
) -> Result<Vec<TournamentId>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        r#"select id from tournaments
           where status in ('registration', 'check_in') and starts_at <= $1
             and (start_reminded_at is null or start_reminded_at < starts_at)
             and not exists (select 1 from tenants n
                             where n.id = tournaments.tenant_id and n.deletion_due_at is not null)
           order by id"#,
        now
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(TournamentId)
    .collect())
}

/// Marca o aviso de hora de iniciar, se ainda vale (nada mudou desde a
/// busca). Devolve o tenant do torneio, ou `None` se outro já marcou.
pub async fn mark_start_reminded(
    conn: &mut PgConnection,
    id: TournamentId,
    now: DateTime<Utc>,
) -> Result<Option<TenantId>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        r#"update tournaments set start_reminded_at = $2
           where id = $1 and status in ('registration', 'check_in') and starts_at <= $2
             and (start_reminded_at is null or start_reminded_at < starts_at)
             and not exists (select 1 from tenants n
                             where n.id = tournaments.tenant_id and n.deletion_due_at is not null)
           returning tenant_id"#,
        id.get(),
        now
    )
    .fetch_optional(conn)
    .await?
    .map(TenantId::new))
}
