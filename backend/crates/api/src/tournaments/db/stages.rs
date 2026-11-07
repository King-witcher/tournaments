//! O andamento de cada fase em `tournament_stages` (`StageProgress`): o
//! início, a abertura do check-in e o fim. Quem chama já travou o torneio; a
//! regra de quando cada um pode mudar fica com ele (`matches`, `stages`), e o
//! banco confere a sequência no commit (`tournament_stages_sequence`).

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use super::TournamentId;

/// A fase `stage` começou agora.
pub async fn mark_stage_started(
    conn: &mut PgConnection,
    id: TournamentId,
    stage: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournament_stages set started_at = now(), finished_at = null
         where tournament_id = $1 and position = $2",
        id.get(),
        stage
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O início da fase `stage` foi desfeito (o check-in dela, se abriu, fica).
pub async fn unmark_stage_started(
    conn: &mut PgConnection,
    id: TournamentId,
    stage: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournament_stages set started_at = null, finished_at = null
         where tournament_id = $1 and position = $2",
        id.get(),
        stage
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// A fase `stage` terminou agora (`finished`) ou foi reaberta, se no banco
/// ainda não está assim. Devolve o fim gravado (`Some(None)`: reaberta) se
/// mudou; `None` se já estava assim.
pub async fn mark_stage_finished(
    conn: &mut PgConnection,
    id: TournamentId,
    stage: i16,
    finished: bool,
) -> Result<Option<Option<DateTime<Utc>>>, sqlx::Error> {
    let row = sqlx::query!(
        "update tournament_stages set finished_at = case when $3 then now() end
         where tournament_id = $1 and position = $2
           and (finished_at is not null) is distinct from $3
         returning finished_at",
        id.get(),
        stage,
        finished
    )
    .fetch_optional(conn)
    .await?;
    Ok(row.map(|r| r.finished_at))
}

/// A organização abriu o check-in da fase `stage`. `false` se já estava
/// aberto.
pub async fn open_stage_check_in(
    conn: &mut PgConnection,
    id: TournamentId,
    stage: i16,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "update tournament_stages set check_in_opened_at = now()
         where tournament_id = $1 and position = $2 and check_in_opened_at is null",
        id.get(),
        stage
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Desfazer o início do torneio: nenhuma fase começou, terminou ou abriu o
/// check-in.
pub async fn reset_stages(conn: &mut PgConnection, id: TournamentId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournament_stages
         set started_at = null, check_in_opened_at = null, finished_at = null
         where tournament_id = $1
           and (started_at, check_in_opened_at, finished_at) is distinct from (null, null, null)",
        id.get()
    )
    .execute(conn)
    .await?;
    Ok(())
}
