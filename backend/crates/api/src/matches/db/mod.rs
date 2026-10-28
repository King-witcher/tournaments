//! SQL da chave: leitura das séries decididas, confirmação por tempo dos
//! mapas e descarte aqui; escritas da série em `writes`, mapas em `games`,
//! histórico em `events`.

mod events;
mod games;
mod writes;

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use super::result::{ResultRow, StoredResult};
use crate::{shared::ApiError, tenants::TenantId, tournaments::TournamentId};

pub use events::{EventRow, events, insert_event};
pub use games::{NewGame, confirm_game, contest_game, games, insert_report, put_game, void_games};
pub use writes::{put_by_organizer, put_decided, put_walkover, remove_result};

/// Uma partida no banco: tenant, torneio, fase e identificador.
pub struct MatchRef<'a> {
    pub tenant: TenantId,
    pub tournament: TournamentId,
    pub stage: i16,
    pub key: &'a str,
}

/// Séries decididas da fase `stage`, na ordem do replay.
pub async fn results(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<Vec<StoredResult>, ApiError> {
    sqlx::query_as!(
        ResultRow,
        r#"select id, match_key, side_a, side_b, winner, a_wins, b_wins,
                  confirmed_via as "confirmed_via!", confirmed_at as "confirmed_at!",
                  position as "position!"
           from match_results
           where tournament_id = $1 and stage = $2
           order by position"#,
        tournament.get(),
        stage
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(StoredResult::try_from)
    .collect()
}

/// Há mapa pendente com a janela de contestação vencida em `now`.
pub async fn has_due(
    conn: &mut PgConnection,
    tournament: TournamentId,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from match_games
                         where tournament_id = $1 and status = 'pending' and confirm_by <= $2)
           as "due!""#,
        tournament.get(),
        now
    )
    .fetch_one(conn)
    .await
}

/// Confirma os mapas pendentes com a janela vencida em `now`, como se
/// tivessem confirmado no fim da janela (com o histórico). Fechar as séries
/// que isso decide fica com quem chama. Devolve quantos confirmou.
pub async fn confirm_due(
    conn: &mut PgConnection,
    tournament: TournamentId,
    now: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        r#"with confirmed as (
               update match_games
               set status = 'confirmed', confirmed_via = 'deadline', confirmed_at = confirm_by
               where tournament_id = $1 and status = 'pending' and confirm_by <= $2
               returning tenant_id, tournament_id, stage, match_key, number, confirm_by
           )
           insert into match_events
               (tenant_id, tournament_id, stage, match_key, game, kind, created_at)
           select tenant_id, tournament_id, stage, match_key, number, 'auto_confirmed', confirm_by
           from confirmed"#,
        tournament.get(),
        now
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected())
}

/// Apaga séries, mapas e histórico do torneio (desfazer o início). Devolve
/// quantas partidas tinham resultado ou mapa.
pub async fn discard(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<u64, sqlx::Error> {
    let touched = sqlx::query_scalar!(
        r#"select count(*) as "n!" from (
               select stage, match_key from match_results where tournament_id = $1
               union
               select stage, match_key from match_games where tournament_id = $1
           ) touched"#,
        tournament.get()
    )
    .fetch_one(&mut *conn)
    .await?;
    let id = tournament.get();
    sqlx::query!("delete from match_events where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!("delete from match_games where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!("delete from match_results where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    Ok(touched as u64)
}
