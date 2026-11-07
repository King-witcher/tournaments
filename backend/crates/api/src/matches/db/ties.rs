//! Os empates do fim da fase (`stage_tie_decisions`, `stage_pending_ties`):
//! gravar a decisão da organização e marcar ou desmarcar a fase que espera
//! por ela. A leitura vem com o resto da fase (`stage_rows`, parte `ties`);
//! apagar fica no `discard`.

use engine::core::ParticipantId;
use serde::Deserialize;
use sqlx::PgConnection;

use crate::{tenants::TenantId, tournaments::TournamentId};

/// Uma decisão como vem do JSON de `stage_rows`: os times na ordem decidida.
#[derive(Deserialize, Debug)]
pub struct TieRow {
    pub teams: Vec<i64>,
    pub reason: Option<String>,
}

/// Grava a ordem `teams` (do melhor ao pior) de um grupo empatado da fase `at`,
/// decidida por `by`, com o motivo, numa ida.
pub async fn insert_decision(
    conn: &mut PgConnection,
    (tenant, tournament, stage): (TenantId, TournamentId, i16),
    teams: &[ParticipantId],
    by: i64,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    let teams: Vec<i64> = teams.iter().map(|p| p.0).collect();
    sqlx::query!(
        r#"with d as (
               insert into stage_tie_decisions (tenant_id, tournament_id, stage, decided_by, reason)
               values ($1, $2, $3, $4, $5)
               returning id, tenant_id, tournament_id
           )
           insert into stage_tie_decision_teams
               (tenant_id, tournament_id, decision_id, position, team_id)
           select d.tenant_id, d.tournament_id, d.id, t.n::smallint, t.team
           from d, unnest($6::bigint[]) with ordinality as t(team, n)"#,
        tenant.get(),
        tournament.get(),
        stage,
        by,
        reason,
        &teams
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Marca a fase como esperando a organização. Devolve se marcou agora (já
/// marcada, nada muda).
pub async fn mark_pending(
    conn: &mut PgConnection,
    (tenant, tournament, stage): (TenantId, TournamentId, i16),
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "insert into stage_pending_ties (tenant_id, tournament_id, stage) values ($1, $2, $3)
         on conflict do nothing",
        tenant.get(),
        tournament.get(),
        stage
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Tira a marca da fase.
pub async fn clear_pending(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from stage_pending_ties where tournament_id = $1 and stage = $2",
        tournament.get(),
        stage
    )
    .execute(conn)
    .await?;
    Ok(())
}
