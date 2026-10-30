use sqlx::PgConnection;

use super::types::Participant;
use crate::{tenants::TenantId, tournaments::TournamentId};

/// Inscritos do torneio (ou só `only_user`), na ordem de inscrição, com o
/// seed e a desistência do time de cada um.
pub async fn list(
    conn: &mut PgConnection,
    tournament: TournamentId,
    only_user: Option<i64>,
) -> Result<Vec<Participant>, sqlx::Error> {
    sqlx::query_as!(
        Participant,
        r#"select p.user_id, m.nickname, u.avatar_url, p.registered_at,
                  p.checked_in_at is not null as "checked_in!", p.team_id,
                  t.seed as "seed?",
                  t.withdrawn_at is not null as "withdrawn!",
                  m.left_at is not null as "left!"
           from participants p
           join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           join users u on u.id = p.user_id
           left join teams t on t.tournament_id = p.tournament_id and t.id = p.team_id
           where p.tournament_id = $1 and ($2::bigint is null or p.user_id = $2)
           order by p.registered_at, p.user_id"#,
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

/// Uma inscrição de um membro, num torneio do tenant.
pub struct Registration {
    /// Slug do torneio.
    pub tournament: String,
    pub checked_in: bool,
}

/// As inscrições de `user_id` nos torneios do tenant, em qualquer status.
pub async fn registrations_of(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
) -> Result<Vec<Registration>, sqlx::Error> {
    sqlx::query_as!(
        Registration,
        r#"select t.slug as tournament, p.checked_in_at is not null as "checked_in!"
           from participants p
           join tournaments t on t.tenant_id = p.tenant_id and t.id = p.tournament_id
           where p.tenant_id = $1 and p.user_id = $2"#,
        tenant.get(),
        user_id
    )
    .fetch_all(conn)
    .await
}

/// Inscreve `user_id` se ele for membro ativo do tenant; `false` se não é.
/// No duelo, o banco cria o time dele junto. `invite_id` é o convite usado e
/// `added_by`, o organizador que inscreveu. Com `checked_in`, o check-in já
/// entra feito (pelo organizador, se foi ele quem inscreveu). Quem chama já
/// conferiu que ele não está inscrito, com o torneio travado.
pub async fn insert(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    user_id: i64,
    invite_id: Option<i64>,
    added_by: Option<i64>,
    checked_in: bool,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"insert into participants (tenant_id, tournament_id, user_id, invite_id, added_by,
                                     checked_in_at, checked_in_by)
           select $1::bigint, $2::bigint, $3::bigint, $4::bigint, $5::bigint,
                  case when $6 then now() end, case when $6 then $5::bigint end
           from tenant_members
           where tenant_id = $1 and user_id = $3 and left_at is null"#,
        tenant.get(),
        tournament.get(),
        user_id,
        invite_id,
        added_by,
        checked_in
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// `false` se `user_id` não estava inscrito. No duelo, o time dele sai junto
/// (o time sai com a inscrição do capitão).
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

/// Tira as inscrições de quem saiu do tenant ou está num time que desistiu,
/// ao desfazer o início (com o torneio já fora de andamento: inscrito de
/// torneio iniciado não sai). No duelo, o time sai junto.
pub async fn delete_departed(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"delete from participants p
           using tenant_members m
           where m.tenant_id = p.tenant_id and m.user_id = p.user_id and p.tournament_id = $1
             and (m.left_at is not null
                  or exists(select 1 from teams t
                            where t.tournament_id = p.tournament_id and t.id = p.team_id
                              and t.withdrawn_at is not null))"#,
        tournament.get()
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}

/// Marca o check-in de `user_id` (`by`: o organizador que marcou; `None`, o
/// próprio). Quem já tinha feito continua com o horário e o autor de antes.
/// `false` se ele não está inscrito.
pub async fn check_in(
    conn: &mut PgConnection,
    tournament: TournamentId,
    user_id: i64,
    by: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"update participants
           set checked_in_by = case when checked_in_at is null then $3 else checked_in_by end,
               checked_in_at = coalesce(checked_in_at, now())
           where tournament_id = $1 and user_id = $2"#,
        tournament.get(),
        user_id,
        by
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Desfaz o check-in de `user_id`. `false` se ele não está inscrito.
pub async fn undo_check_in(
    conn: &mut PgConnection,
    tournament: TournamentId,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"update participants set checked_in_at = null, checked_in_by = null
           where tournament_id = $1 and user_id = $2"#,
        tournament.get(),
        user_id
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}
