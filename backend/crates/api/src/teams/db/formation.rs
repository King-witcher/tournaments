//! SQL da formação dos times do TDM (criar, mover jogadores, nome e
//! capitão) e da substituição durante o torneio.

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use crate::{shared::ApiError, tenants::TenantId, tournaments::TournamentId};

/// Um inscrito, como a formação o vê.
#[derive(Debug, Clone)]
pub struct Player {
    pub user_id: i64,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub checked_in: bool,
    pub left: bool,
    pub team_id: Option<i64>,
}

/// Os inscritos do torneio, na ordem de inscrição.
pub async fn players(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<Vec<Player>, sqlx::Error> {
    sqlx::query_as!(
        Player,
        r#"select p.user_id, m.nickname, u.avatar_url, p.registered_at,
                  p.checked_in_at is not null as "checked_in!",
                  m.left_at is not null as "left!", p.team_id
           from participants p
           join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           join users u on u.id = p.user_id
           where p.tournament_id = $1
           order by p.registered_at, p.user_id"#,
        tournament.get()
    )
    .fetch_all(conn)
    .await
}

/// Um time, travado para mudar.
#[derive(Debug, Clone)]
pub struct TeamRow {
    pub id: i64,
    pub captain: i64,
    pub seed: Option<i32>,
    pub withdrawn: bool,
    /// Os jogadores, o capitão primeiro e depois na ordem de inscrição.
    pub members: Vec<i64>,
}

/// O time `team` do torneio, travado (`for update`).
pub async fn lock_team(
    conn: &mut PgConnection,
    tournament: TournamentId,
    team: i64,
) -> Result<Option<TeamRow>, sqlx::Error> {
    let Some(row) = sqlx::query!(
        r#"select id, captain, seed, withdrawn_at is not null as "withdrawn!"
           from teams where tournament_id = $1 and id = $2
           for update"#,
        tournament.get(),
        team
    )
    .fetch_optional(&mut *conn)
    .await?
    else {
        return Ok(None);
    };
    let members = sqlx::query_scalar!(
        r#"select user_id from participants
           where tournament_id = $1 and team_id = $2
           order by user_id <> $3, registered_at, user_id"#,
        tournament.get(),
        team,
        row.captain
    )
    .fetch_all(conn)
    .await?;
    Ok(Some(TeamRow {
        id: row.id,
        captain: row.captain,
        seed: row.seed,
        withdrawn: row.withdrawn,
        members,
    }))
}

/// Apaga todos os times do torneio (os inscritos ficam sem time).
pub async fn clear(conn: &mut PgConnection, tournament: TournamentId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from teams where tournament_id = $1",
        tournament.get()
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Cria um time com `players` (o primeiro é o capitão), tirando cada um do
/// time em que estava (quem chama já esvaziou ou reorganizou os de origem).
pub async fn create(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    players: &[i64],
) -> Result<i64, sqlx::Error> {
    let id = sqlx::query_scalar!(
        r#"insert into teams (tenant_id, tournament_id, captain)
           values ($1, $2, $3)
           returning id"#,
        tenant.get(),
        tournament.get(),
        players[0]
    )
    .fetch_one(&mut *conn)
    .await?;
    sqlx::query!(
        "update participants set team_id = $3 where tournament_id = $1 and user_id = any($2)",
        tournament.get(),
        players,
        id
    )
    .execute(conn)
    .await?;
    Ok(id)
}

/// Põe `user_id` no time `team` (`None`: sem time). `false` se ele não está
/// inscrito.
pub async fn set_team(
    conn: &mut PgConnection,
    tournament: TournamentId,
    user_id: i64,
    team: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "update participants set team_id = $3 where tournament_id = $1 and user_id = $2",
        tournament.get(),
        user_id,
        team
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Tira `user_id` do time dele (ele fica sem time). O time que fica vazio
/// some; o capitão que sai passa a capitania ao jogador do time inscrito há
/// mais tempo (`hand_over`).
pub async fn leave_team(
    conn: &mut PgConnection,
    tournament: TournamentId,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    let Some(Some(team)) = sqlx::query_scalar!(
        "select team_id from participants where tournament_id = $1 and user_id = $2",
        tournament.get(),
        user_id
    )
    .fetch_optional(&mut *conn)
    .await?
    else {
        return Ok(());
    };
    if !hand_over(conn, tournament, team, user_id, false).await? {
        // Era o único: o time some (e ele fica sem time pela FK).
        sqlx::query!(
            "delete from teams where tournament_id = $1 and id = $2",
            tournament.get(),
            team
        )
        .execute(conn)
        .await?;
        return Ok(());
    }
    set_team(conn, tournament, user_id, None).await?;
    Ok(())
}

/// Se `leaving` é o capitão do time `team`, a capitania passa ao jogador do
/// time inscrito há mais tempo (`active_only`: entre os que estão na
/// comunidade). `false` se não há a quem passar (e o capitão fica).
pub async fn hand_over(
    conn: &mut PgConnection,
    tournament: TournamentId,
    team: i64,
    leaving: i64,
    active_only: bool,
) -> Result<bool, sqlx::Error> {
    let next = sqlx::query_scalar!(
        r#"select p.user_id from participants p
           join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           where p.tournament_id = $1 and p.team_id = $2 and p.user_id <> $3
             and (not $4 or m.left_at is null)
           order by p.registered_at, p.user_id
           limit 1"#,
        tournament.get(),
        team,
        leaving,
        active_only
    )
    .fetch_optional(&mut *conn)
    .await?;
    let Some(next) = next else {
        return Ok(false);
    };
    sqlx::query!(
        "update teams set captain = $4 where tournament_id = $1 and id = $2 and captain = $3",
        tournament.get(),
        team,
        leaving,
        next
    )
    .execute(conn)
    .await?;
    Ok(true)
}

/// O capitão do time `team` passa a ser `captain`.
pub async fn set_captain(
    conn: &mut PgConnection,
    tournament: TournamentId,
    team: i64,
    captain: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update teams set captain = $3 where tournament_id = $1 and id = $2",
        tournament.get(),
        team,
        captain
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O nome do time (`None` tira). 409 `team.name_taken` se outro time do
/// torneio já tem esse nome, sem diferenciar maiúsculas.
pub async fn set_name(
    conn: &mut PgConnection,
    tournament: TournamentId,
    team: i64,
    name: Option<&str>,
) -> Result<(), ApiError> {
    let done = sqlx::query!(
        "update teams set name = $3 where tournament_id = $1 and id = $2",
        tournament.get(),
        team,
        name
    )
    .execute(conn)
    .await;
    match done {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(e)) if e.constraint() == Some("teams_name_key") => {
            Err(ApiError::conflict("team.name_taken"))
        }
        Err(e) => Err(e.into()),
    }
}

/// Guarda a seed do sorteio dos times (o u64 bit a bit no bigint).
pub async fn save_teams_seed(
    conn: &mut PgConnection,
    tournament: TournamentId,
    seed: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update tournaments set teams_seed = $2 where id = $1",
        tournament.get(),
        seed as i64
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Inscreve o substituto `user_id` já no time `team`, com o torneio em
/// andamento (só membro ativo; `added_by`: o organizador). `false` se ele
/// não é membro ativo.
pub async fn insert_substitute(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    user_id: i64,
    team: i64,
    added_by: i64,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        r#"insert into participants (tenant_id, tournament_id, user_id, added_by, team_id)
           select $1::bigint, $2::bigint, $3::bigint, $4::bigint, $5::bigint
           from tenant_members
           where tenant_id = $1 and user_id = $3 and left_at is null"#,
        tenant.get(),
        tournament.get(),
        user_id,
        added_by,
        team
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}
