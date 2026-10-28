use sqlx::PgConnection;

use super::{seed_order::Entrant, types::Participant};
use crate::{tenants::TenantId, tournaments::TournamentId};

/// Inscritos do torneio (ou só `only_user`), na ordem de inscrição.
pub async fn list(
    conn: &mut PgConnection,
    tournament: TournamentId,
    only_user: Option<i64>,
) -> Result<Vec<Participant>, sqlx::Error> {
    sqlx::query_as!(
        Participant,
        r#"select p.user_id, m.nickname, u.avatar_url, p.registered_at,
                  p.checked_in_at is not null as "checked_in!", p.seed,
                  m.left_at is not null as "left!"
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

/// Um participante da chave.
pub struct Seeded {
    /// `participants.id`: o `ParticipantId` do `engine`.
    pub id: i64,
    pub user_id: i64,
    /// Apelido no tenant.
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub seed: i32,
    /// Saiu da comunidade (ou foi removido).
    pub left: bool,
}

/// Quem está na chave: os inscritos com seed, do 1 em diante. Depois de
/// iniciar, os seeds são 1..N entre os elegíveis (`seeding::finalize`).
pub async fn seeded(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<Vec<Seeded>, sqlx::Error> {
    sqlx::query_as!(
        Seeded,
        r#"select p.id, p.user_id, m.nickname, u.avatar_url, p.seed as "seed!",
                  m.left_at is not null as "left!"
           from participants p
           join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           join users u on u.id = p.user_id
           where p.tournament_id = $1 and p.seed is not null
           order by p.seed"#,
        tournament.get()
    )
    .fetch_all(conn)
    .await
}

/// Inscritos para o seeding, na ordem de inscrição (a mesma de `list`).
pub async fn entrants(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<Vec<Entrant>, sqlx::Error> {
    sqlx::query_as!(
        Entrant,
        r#"select id, user_id, seed, checked_in_at is not null as "checked_in!"
           from participants
           where tournament_id = $1
           order by registered_at, id"#,
        tournament.get()
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

/// Inscreve `user_id` se ele for membro ativo do tenant; `false` se não é.
/// Quem chama já conferiu que ele não está inscrito, com o torneio travado.
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
           where tenant_id = $1 and user_id = $3 and left_at is null"#,
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

/// Tira as inscrições de quem saiu do tenant, ao desfazer o início (com o
/// torneio já fora de andamento: inscrito de torneio iniciado não sai).
pub async fn delete_departed(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"delete from participants p
           using tenant_members m
           where m.tenant_id = p.tenant_id and m.user_id = p.user_id
             and p.tournament_id = $1 and m.left_at is not null"#,
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

/// Grava o seeding: `order` (ids de `participants`) recebe 1..N, na ordem, e
/// os outros inscritos ficam sem seed.
pub async fn save_seeds(
    conn: &mut PgConnection,
    tournament: TournamentId,
    order: &[i64],
) -> Result<(), sqlx::Error> {
    // Limpa antes, para a troca de posições não esbarrar no unique.
    sqlx::query!(
        "update participants set seed = null where tournament_id = $1",
        tournament.get()
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        r#"update participants p set seed = o.seed::integer
           from unnest($2::bigint[]) with ordinality as o(id, seed)
           where p.tournament_id = $1 and p.id = o.id"#,
        tournament.get(),
        order
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Guarda a seed do sorteio no torneio (o u64 bit a bit no bigint).
pub async fn save_draw_seed(
    conn: &mut PgConnection,
    tournament: TournamentId,
    seed: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournaments set seeding_seed = $2 where id = $1",
        tournament.get(),
        seed as i64
    )
    .execute(conn)
    .await?;
    Ok(())
}
