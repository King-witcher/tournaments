//! Os times nas fases da 2 em diante (`stage_teams`): a presença confirmada
//! no check-in da fase e o seed do time nela, gravado quando ela começa. O
//! banco confere quando cada um pode mudar (`stage_teams_guard`) e os seeds de
//! 1 a K no commit (`stage_teams_seeds`).

use sqlx::PgConnection;

use crate::{tenants::TenantId, tournaments::TournamentId};

/// `by` confirmou a presença do time `team` na fase `stage`. `false` se já
/// estava confirmada.
pub async fn check_in_stage(
    conn: &mut PgConnection,
    (tenant, tournament): (TenantId, TournamentId),
    stage: i16,
    team: i64,
    by: i64,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"insert into stage_teams (tenant_id, tournament_id, stage, team_id, checked_in_at,
                                    checked_in_by)
           values ($1, $2, $3, $4, now(), $5)
           on conflict (tournament_id, stage, team_id) do update
               set checked_in_at = now(), checked_in_by = excluded.checked_in_by
               where stage_teams.checked_in_at is null"#,
        tenant.get(),
        tournament.get(),
        stage,
        team,
        by
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Desfaz a presença do time `team` na fase `stage` (que não começou: a linha
/// só tem a presença). `false` se não estava confirmada.
pub async fn undo_stage_check_in(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
    team: i64,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "delete from stage_teams
         where tournament_id = $1 and stage = $2 and team_id = $3 and seed is null",
        tournament.get(),
        stage,
        team
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Grava os seeds da fase `stage`, que vai começar: `order` (ids de times)
/// recebe 1..K, na ordem.
pub async fn save_stage_seeds(
    conn: &mut PgConnection,
    (tenant, tournament): (TenantId, TournamentId),
    stage: i16,
    order: &[i64],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into stage_teams (tenant_id, tournament_id, stage, team_id, seed)
           select $1, $2, $3, o.id, o.seed::smallint
           from unnest($4::bigint[]) with ordinality as o(id, seed)
           on conflict (tournament_id, stage, team_id) do update set seed = excluded.seed"#,
        tenant.get(),
        tournament.get(),
        stage,
        order
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Apaga os seeds da fase `stage` (desfazer o início dela, já desmarcado): as
/// presenças confirmadas ficam.
pub async fn clear_stage_seeds(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from stage_teams
         where tournament_id = $1 and stage = $2 and checked_in_at is null",
        tournament.get(),
        stage
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "update stage_teams set seed = null
         where tournament_id = $1 and stage = $2 and seed is not null",
        tournament.get(),
        stage
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Desfazer o início do torneio (com as fases já desmarcadas): nenhum time
/// fica em fase nenhuma, nem com presença confirmada.
pub async fn clear_stages(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from stage_teams where tournament_id = $1",
        tournament.get()
    )
    .execute(conn)
    .await?;
    Ok(())
}
