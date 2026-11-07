//! SQL da chave: confirmação por tempo dos mapas e descarte aqui; a leitura
//! da fase inteira numa query (séries decididas, mapas, pareamentos, veto) em
//! `stage`; escritas da série em `writes`, mapas em `games`, histórico da
//! partida em `events` (com as escalações dela, `match_history`), o que a
//! chave lê dos screenshots em `screenshots`, os pareamentos gravados do
//! suíço em `pairings`, as decisões dos empates do fim em `ties`.

mod events;
mod games;
mod pairings;
mod screenshots;
mod stage;
mod ties;
mod writes;

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use crate::{tenants::TenantId, tournaments::TournamentId};

pub use events::{EventRow, insert_event, match_history};
pub use games::{
    Deadline, LineupRow, NewGame, accept_contest, confirm_game, contest_game, contest_notices_due,
    insert_report, put_game, take_contest_notices, void_games,
};
pub use pairings::insert_round;
pub use screenshots::has_screenshot;
pub use stage::{Parts, StageRows, Which, stage_rows};
pub use ties::{TieRow, clear_pending, insert_decision, mark_pending};
pub use writes::{put_by_organizer, put_decided, put_walkover, remove_result};

/// Uma partida no banco: tenant, torneio, fase e identificador.
pub struct MatchRef<'a> {
    pub tenant: TenantId,
    pub tournament: TournamentId,
    pub stage: i16,
    pub key: &'a str,
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

/// Apaga séries, mapas, histórico, vetos, pareamentos e os empates do fim do
/// torneio (desfazer o início). Devolve quantas partidas tinham resultado ou mapa.
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
    sqlx::query!("delete from match_veto_events where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!("delete from match_events where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!("delete from match_games where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!("delete from match_results where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!("delete from stage_pairings where tournament_id = $1", id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!(
        "with p as (delete from stage_pending_ties where tournament_id = $1)
         delete from stage_tie_decisions where tournament_id = $1",
        id
    )
    .execute(&mut *conn)
    .await?;
    Ok(touched as u64)
}

/// Algo foi feito na fase `stage`: um evento de veto, um mapa enviado, uma
/// série decidida (W.O. inclusive), um evento no histórico de uma partida
/// (também de mapa já anulado) ou um screenshot (também o que ainda não foi
/// anexado). Tudo isso fica preso à fase, e desfazer o início o deixaria
/// numa série nova com o mesmo código.
pub async fn stage_activity(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select (exists(select 1 from match_veto_events where tournament_id = $1 and stage = $2)
                   or exists(select 1 from match_games where tournament_id = $1 and stage = $2)
                   or exists(select 1 from match_results where tournament_id = $1 and stage = $2)
                   or exists(select 1 from match_events where tournament_id = $1 and stage = $2)
                   or exists(select 1 from match_screenshots
                             where tournament_id = $1 and stage = $2))
           as "active!""#,
        tournament.get(),
        stage
    )
    .fetch_one(conn)
    .await
}

/// Apaga os pareamentos da fase `stage` (desfazer o início dela, sem nada
/// jogado).
pub async fn discard_pairings(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from stage_pairings where tournament_id = $1 and stage = $2",
        tournament.get(),
        stage
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Torneios em andamento com mapa pendente de janela vencida em `now`, para
/// o agendador (que age pelo sistema, sem membro: o tenant sai daqui).
pub async fn due_tournaments(
    db: &sqlx::PgPool,
    now: DateTime<Utc>,
) -> Result<Vec<(TenantId, String)>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"select t.tenant_id, t.slug from tournaments t
           where t.status = 'in_progress'
             and exists(select 1 from match_games g
                        where g.tournament_id = t.id and g.status = 'pending'
                          and g.confirm_by <= $1)
           order by t.id"#,
        now
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| (TenantId::new(r.tenant_id), r.slug))
    .collect())
}
