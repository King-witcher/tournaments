//! SQL da formação dos times do TDM (gravar o arranjo inteiro, nome e
//! capitão) e da substituição durante o torneio.

use std::collections::HashMap;

use sqlx::PgConnection;

use crate::{
    auth::resolve_avatars, shared::ApiError, tenants::TenantId, tournaments::TournamentId,
};

/// Um inscrito, como a formação o vê.
#[derive(Debug, Clone)]
pub struct Player {
    pub user_id: i64,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub checked_in: bool,
    pub left: bool,
    pub team_id: Option<i64>,
}

/// Os inscritos do torneio, na ordem de inscrição.
pub async fn players(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<Vec<Player>, sqlx::Error> {
    let mut rows = sqlx::query_as!(
        Player,
        r#"select p.user_id, m.nickname, coalesce(u.avatar_key, u.avatar_url) as avatar_url,
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
    .await?;
    resolve_avatars(&mut rows, |p| &mut p.avatar_url);
    Ok(rows)
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

/// Um time como a formação salva o quer: sem `id`, um time novo.
pub struct ArrangedTeam {
    pub id: Option<i64>,
    /// Pelo menos um jogador.
    pub members: Vec<i64>,
    /// Um dos `members`.
    pub captain: i64,
}

/// Os times do torneio e o capitão de cada um.
pub async fn captains(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<HashMap<i64, i64>, sqlx::Error> {
    let rows = sqlx::query!(
        "select id, captain from teams where tournament_id = $1",
        tournament.get()
    )
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|r| (r.id, r.captain)).collect())
}

/// Grava a formação inteira: os times que não vêm somem, quem não está em
/// nenhum fica sem time, os que vêm com id ficam com esses jogadores e esse
/// capitão (o nome e o seed continuam) e os sem id nascem agora. `current` são
/// os capitães de agora (`captains`).
///
/// O capitão é único por torneio num índice comum, conferido linha a linha:
/// quem troca de capitão passa antes por um valor provisório (o id negativo do
/// time, que não é nenhum inscrito), que a FK do capitão aceita porque só
/// confere no commit, e depois recebe o definitivo. Os times novos nascem por
/// último, com o capitão deles já livre.
pub async fn arrange(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: TournamentId,
    current: &HashMap<i64, i64>,
    teams: &[ArrangedTeam],
) -> Result<(), sqlx::Error> {
    let kept: Vec<(i64, &ArrangedTeam)> = teams.iter().filter_map(|t| Some((t.id?, t))).collect();
    let kept_ids: Vec<i64> = kept.iter().map(|(id, _)| *id).collect();
    sqlx::query!(
        "delete from teams where tournament_id = $1 and id <> all($2)",
        tournament.get(),
        &kept_ids
    )
    .execute(&mut *conn)
    .await?;

    let placed: Vec<i64> = teams.iter().flat_map(|t| t.members.clone()).collect();
    sqlx::query!(
        r#"update participants set team_id = null
           where tournament_id = $1 and team_id is not null and user_id <> all($2)"#,
        tournament.get(),
        &placed
    )
    .execute(&mut *conn)
    .await?;

    let (changed, captains): (Vec<i64>, Vec<i64>) = kept
        .iter()
        .filter(|(id, team)| current.get(id) != Some(&team.captain))
        .map(|(id, team)| (*id, team.captain))
        .unzip();
    sqlx::query!(
        "update teams set captain = -id where tournament_id = $1 and id = any($2)",
        tournament.get(),
        &changed
    )
    .execute(&mut *conn)
    .await?;

    let (users, targets): (Vec<i64>, Vec<i64>) = kept
        .iter()
        .flat_map(|(id, team)| team.members.iter().map(move |user| (*user, *id)))
        .unzip();
    sqlx::query!(
        r#"update participants p set team_id = m.team
           from unnest($2::bigint[], $3::bigint[]) as m(user_id, team)
           where p.tournament_id = $1 and p.user_id = m.user_id"#,
        tournament.get(),
        &users,
        &targets
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query!(
        r#"update teams t set captain = c.captain
           from unnest($2::bigint[], $3::bigint[]) as c(id, captain)
           where t.tournament_id = $1 and t.id = c.id"#,
        tournament.get(),
        &changed,
        &captains
    )
    .execute(&mut *conn)
    .await?;

    for team in teams.iter().filter(|t| t.id.is_none()) {
        let players: Vec<i64> = std::iter::once(team.captain)
            .chain(team.members.iter().copied().filter(|u| *u != team.captain))
            .collect();
        create(conn, tenant, tournament, &players).await?;
    }
    Ok(())
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
