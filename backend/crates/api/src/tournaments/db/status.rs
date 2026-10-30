//! Mudanças de status: publicar, a abertura automática do check-in e as
//! transições manuais que o domínio `lifecycle` decide.

use sqlx::{PgConnection, PgExecutor};

use super::TournamentId;
use crate::tournaments::{schedule::Schedule, status::TournamentStatus};

/// Rascunho → inscrição.
pub async fn publish(conn: &mut PgConnection, id: TournamentId) -> Result<(), sqlx::Error> {
    set_status(conn, id, TournamentStatus::Registration).await
}

/// Grava o status; quem chama já conferiu a transição com o torneio travado.
pub async fn set_status(
    conn: &mut PgConnection,
    id: TournamentId,
    status: TournamentStatus,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournaments set status = $2, updated_at = now() where id = $1",
        id.get(),
        status.as_str()
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Em andamento ↔ encerrado conforme a chave: encerra quando a partida
/// decisiva é confirmada e volta para em andamento se uma correção deixa
/// partida por jogar. Só mexe nesses dois status (desfazer o início é
/// transição do `lifecycle`).
pub async fn sync_finished(
    conn: &mut PgConnection,
    id: TournamentId,
    finished: bool,
) -> Result<TournamentStatus, sqlx::Error> {
    let status = if finished {
        TournamentStatus::Finished
    } else {
        TournamentStatus::InProgress
    };
    sqlx::query!(
        r#"update tournaments set status = $2, updated_at = now()
           where id = $1 and status in ('in_progress', 'finished') and status <> $2"#,
        id.get(),
        status.as_str()
    )
    .execute(conn)
    .await?;
    Ok(status)
}

/// Inscrição → check-in pela abertura automática, só se nada mudou desde a
/// leitura (`schedule` é o lido): uma mudança de data no meio do caminho vence; na comunidade com a exclusão
/// agendada (só leitura), não abre. `false` se não mudou.
pub(super) async fn open_due_check_in<'e>(
    db: impl PgExecutor<'e>,
    id: TournamentId,
    schedule: &Schedule,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"update tournaments set status = 'check_in', updated_at = now()
           where id = $1 and status = 'registration'
             and starts_at is not distinct from $2
             and check_in_minutes is not distinct from $3
             and not exists (select 1 from tenants n
                             where n.id = tournaments.tenant_id and n.deletion_due_at is not null)"#,
        id.get(),
        schedule.starts_at,
        schedule.check_in_minutes
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
