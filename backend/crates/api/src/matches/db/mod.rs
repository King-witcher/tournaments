//! SQL da chave: resultados aqui; histórico em `events`, detalhe por mapa em
//! `games`.

mod events;
mod games;

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use super::result::{ResultRow, StoredResult};
use crate::{shared::ApiError, tournaments::TournamentId};

pub use events::{EventRow, events};
pub use games::{GameRow, games};

/// Resultados da fase `stage`: os confirmados na ordem do replay, depois os
/// outros.
pub async fn results(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<Vec<StoredResult>, ApiError> {
    sqlx::query_as!(
        ResultRow,
        r#"select id, match_key, side_a, side_b, status, winner, a_wins, b_wins, reported_side,
                  reported_at, confirm_by, contested_at, contest_comment,
                  confirmed_via, confirmed_at, position
           from match_results
           where tournament_id = $1 and stage = $2
           order by position nulls last, id"#,
        tournament.get(),
        stage
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(StoredResult::try_from)
    .collect()
}

/// Há resultado pendente com a janela de contestação vencida em `now`.
pub async fn has_due(
    conn: &mut PgConnection,
    tournament: TournamentId,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from match_results
                         where tournament_id = $1 and status = 'pending' and confirm_by <= $2)
           as "due!""#,
        tournament.get(),
        now
    )
    .fetch_one(conn)
    .await
}

/// Confirma os pendentes com a janela vencida em `now`, como se tivessem
/// confirmado no fim da janela (com o histórico), e entram no replay nessa
/// ordem. Quem chama trava o torneio: as posições são `max + 1`. Devolve
/// quantos confirmou.
pub async fn confirm_due(
    conn: &mut PgConnection,
    tournament: TournamentId,
    now: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        r#"with due as (
               select id, row_number() over (order by confirm_by, id) as n
               from match_results
               where tournament_id = $1 and status = 'pending' and confirm_by <= $2
           ),
           top as (
               select coalesce(max(position), 0) as position
               from match_results where tournament_id = $1
           ),
           confirmed as (
               update match_results r
               set status = 'confirmed', confirmed_via = 'deadline', confirmed_at = r.confirm_by,
                   position = (top.position + due.n)::integer
               from due, top
               where r.id = due.id
               returning r.tenant_id, r.tournament_id, r.stage, r.match_key, r.confirm_by
           )
           insert into match_events (tenant_id, tournament_id, stage, match_key, kind, created_at)
           select tenant_id, tournament_id, stage, match_key, 'auto_confirmed', confirm_by
           from confirmed"#,
        tournament.get(),
        now
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected())
}

/// Apaga resultados e histórico do torneio (desfazer o início). Devolve
/// quantos resultados havia.
pub async fn discard(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "delete from match_events where tournament_id = $1",
        tournament.get()
    )
    .execute(&mut *conn)
    .await?;
    let deleted = sqlx::query!(
        "delete from match_results where tournament_id = $1",
        tournament.get()
    )
    .execute(conn)
    .await?;
    Ok(deleted.rows_affected())
}
