use sqlx::PgConnection;

use super::types::Participant;
use crate::{tenants::TenantId, tournaments::TournamentId};

/// Inscritos do torneio (ou só `only_user`), na ordem de inscrição.
pub async fn list(
    conn: &mut PgConnection,
    tournament: TournamentId,
    only_user: Option<i64>,
) -> Result<Vec<Participant>, sqlx::Error> {
    sqlx::query_as!(
        Participant,
        r#"select p.user_id, m.nickname, u.avatar_url, p.registered_at
           from participants p
           join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           join users u on u.id = p.user_id
           where p.tournament_id = $1 and ($2::bigint is null or p.user_id = $2)
           order by p.registered_at, p.id"#,
        tournament.get(),
        only_user
    )
    .fetch_all(conn)
    .await
}

pub async fn is_registered(
    conn: &mut PgConnection,
    tournament: TournamentId,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from participants where tournament_id = $1 and user_id = $2)
           as "registered!""#,
        tournament.get(),
        user_id
    )
    .fetch_one(conn)
    .await
}

/// Inscreve `user_id` se ele for membro do tenant; `false` se não é. Quem
/// chama já conferiu que ele não está inscrito, com o torneio travado.
pub async fn insert(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    user_id: i64,
    invite_id: Option<i64>,
    added_by: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"insert into participants (tenant_id, tournament_id, user_id, invite_id, added_by)
           select $1::bigint, $2::bigint, $3::bigint, $4::bigint, $5::bigint
           from tenant_members
           where tenant_id = $1 and user_id = $3"#,
        tenant.get(),
        tournament.get(),
        user_id,
        invite_id,
        added_by
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// `false` se `user_id` não estava inscrito.
pub async fn delete(
    conn: &mut PgConnection,
    tournament: TournamentId,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "delete from participants where tournament_id = $1 and user_id = $2",
        tournament.get(),
        user_id
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}
