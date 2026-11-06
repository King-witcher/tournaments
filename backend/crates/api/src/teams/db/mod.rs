mod formation;

use std::collections::HashMap;

use sqlx::PgConnection;

pub use formation::*;

use super::{
    seed_order::Entrant,
    team::{StoredMember, StoredTeam},
};
use crate::{auth::resolve_avatar, tenants::TenantId, tournaments::TournamentId};

/// Uma linha de time com um jogador; `group` junta as do mesmo time.
struct Row {
    id: i64,
    name: Option<String>,
    seed: Option<i32>,
    captain: i64,
    withdrawn: bool,
    user_id: i64,
    nickname: String,
    avatar_url: Option<String>,
    left: bool,
}

/// Junta as linhas consecutivas de cada time, na ordem em que vieram.
fn group(rows: impl IntoIterator<Item = Row>) -> Vec<StoredTeam> {
    let mut teams: Vec<StoredTeam> = Vec::new();
    for r in rows {
        let member = StoredMember {
            user_id: r.user_id,
            nickname: r.nickname,
            avatar_url: resolve_avatar(r.avatar_url),
            left: r.left,
        };
        match teams.last_mut() {
            Some(team) if team.id == r.id => team.members.push(member),
            _ => teams.push(StoredTeam {
                id: r.id,
                name: r.name,
                seed: r.seed,
                captain: r.captain,
                withdrawn: r.withdrawn,
                members: vec![member],
            }),
        }
    }
    teams
}

/// Os times do torneio com os jogadores (o capitão primeiro, depois na ordem
/// de inscrição): pelo seed e, sem seed, na ordem de criação (no duelo, a de
/// inscrição). `seeded_only`: só os que têm seed, os da chave depois de
/// iniciar.
pub async fn list(
    conn: &mut PgConnection,
    tournament: TournamentId,
    seeded_only: bool,
) -> Result<Vec<StoredTeam>, sqlx::Error> {
    let mut all = list_in(conn, &[tournament], seeded_only).await?;
    Ok(all.pop().unwrap_or_default())
}

/// Como `list`, de vários torneios numa query só: os times de cada um, na
/// ordem de `tournaments`.
async fn list_in(
    conn: &mut PgConnection,
    tournaments: &[TournamentId],
    seeded_only: bool,
) -> Result<Vec<Vec<StoredTeam>>, sqlx::Error> {
    let ids: Vec<i64> = tournaments.iter().map(|t| t.get()).collect();
    let rows = sqlx::query!(
        r#"select t.tournament_id, t.id, t.name, t.seed, t.captain,
                  t.withdrawn_at is not null as "withdrawn!",
                  p.user_id, m.nickname, coalesce(u.avatar_key, u.avatar_url) as avatar_url,
                  m.left_at is not null as "left!"
           from teams t
           join participants p on p.tournament_id = t.tournament_id and p.team_id = t.id
           join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           join users u on u.id = p.user_id
           where t.tournament_id = any($1) and (not $2 or t.seed is not null)
           order by t.tournament_id, t.seed nulls last, t.created_at, t.id,
                    p.user_id <> t.captain, p.registered_at, p.user_id"#,
        &ids,
        seeded_only
    )
    .fetch_all(conn)
    .await?;
    let mut by_tournament: HashMap<i64, Vec<Row>> = HashMap::new();
    for r in rows {
        by_tournament.entry(r.tournament_id).or_default().push(Row {
            id: r.id,
            name: r.name,
            seed: r.seed,
            captain: r.captain,
            withdrawn: r.withdrawn,
            user_id: r.user_id,
            nickname: r.nickname,
            avatar_url: r.avatar_url,
            left: r.left,
        });
    }
    Ok(ids
        .iter()
        .map(|id| group(by_tournament.remove(id).unwrap_or_default()))
        .collect())
}

/// Os times da chave: os que têm seed, pelo seed, com os jogadores. Depois
/// de iniciar, os seeds são 1..N entre os elegíveis (`finalize_seeding`).
pub async fn seeded(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<Vec<StoredTeam>, sqlx::Error> {
    list(conn, tournament, true).await
}

/// Como `seeded`, de vários torneios numa query só, na ordem de
/// `tournaments`.
pub async fn seeded_in(
    conn: &mut PgConnection,
    tournaments: &[TournamentId],
) -> Result<Vec<Vec<StoredTeam>>, sqlx::Error> {
    list_in(conn, tournaments, true).await
}

/// Os times `ids` de qualquer torneio do tenant, com os jogadores, pelo id.
pub async fn by_ids(
    conn: &mut PgConnection,
    tenant: TenantId,
    ids: &[i64],
) -> Result<Vec<StoredTeam>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Row,
        r#"select t.id, t.name, t.seed, t.captain, t.withdrawn_at is not null as "withdrawn!",
                  p.user_id, m.nickname, coalesce(u.avatar_key, u.avatar_url) as avatar_url,
                  m.left_at is not null as "left!"
           from teams t
           join participants p on p.tournament_id = t.tournament_id and p.team_id = t.id
           join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           join users u on u.id = p.user_id
           where t.tenant_id = $1 and t.id = any($2)
           order by t.id, p.user_id <> t.captain, p.registered_at, p.user_id"#,
        tenant.get(),
        ids
    )
    .fetch_all(conn)
    .await?;
    Ok(group(rows))
}

/// Os times para o seeding, na ordem de criação (no duelo, a de inscrição),
/// com o check-in de todos os jogadores e quantos são.
pub async fn entrants(
    conn: &mut PgConnection,
    tournament: TournamentId,
) -> Result<Vec<Entrant>, sqlx::Error> {
    sqlx::query_as!(
        Entrant,
        r#"select t.id, t.seed, bool_and(p.checked_in_at is not null) as "checked_in!",
                  count(*) as "players!"
           from teams t
           join participants p on p.tournament_id = t.tournament_id and p.team_id = t.id
           where t.tournament_id = $1
           group by t.id
           order by t.created_at, t.id"#,
        tournament.get()
    )
    .fetch_all(conn)
    .await
}

/// Grava o seeding: `order` (ids de times) recebe 1..N, na ordem, e os
/// outros times ficam sem seed.
pub async fn save_seeds(
    conn: &mut PgConnection,
    tournament: TournamentId,
    order: &[i64],
) -> Result<(), sqlx::Error> {
    // Limpa antes, para a troca de posições não esbarrar no unique.
    sqlx::query!(
        "update teams set seed = null where tournament_id = $1 and seed is not null",
        tournament.get()
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        r#"update teams t set seed = o.seed::integer
           from unnest($2::bigint[]) with ordinality as o(id, seed)
           where t.tournament_id = $1 and t.id = o.id"#,
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

/// Marca a desistência do time `team` no torneio em andamento (`by`: o
/// organizador que marcou). O time que já tinha desistido fica como estava.
/// `None` se o time não é do torneio; `Some(false)` se é, mas ficou fora da
/// chave (sem seed).
pub async fn withdraw(
    conn: &mut PgConnection,
    tournament: TournamentId,
    team: i64,
    by: Option<i64>,
) -> Result<Option<bool>, sqlx::Error> {
    let seed = sqlx::query_scalar!(
        "select seed from teams where tournament_id = $1 and id = $2",
        tournament.get(),
        team
    )
    .fetch_optional(&mut *conn)
    .await?;
    match seed {
        None => Ok(None),
        Some(None) => Ok(Some(false)),
        Some(Some(_)) => {
            sqlx::query!(
                r#"update teams set withdrawn_at = now(), withdrawn_by = $3
                   where tournament_id = $1 and id = $2 and withdrawn_at is null"#,
                tournament.get(),
                team,
                by
            )
            .execute(conn)
            .await?;
            Ok(Some(true))
        }
    }
}

/// `user_id` está saindo da comunidade (`by`: quem removeu; `None`, saiu
/// sozinho): o time dele desiste se está na chave e nenhum outro jogador dele
/// continua na comunidade (no duelo, sempre); se ele era o capitão e alguém
/// fica, a capitania passa adiante. Devolve se o time dele está na chave.
pub async fn withdraw_departing(
    conn: &mut PgConnection,
    tournament: TournamentId,
    user_id: i64,
    by: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let team = sqlx::query!(
        r#"select t.id, t.seed is not null as "seeded!",
                  not exists(select 1 from participants o
                             join tenant_members m
                               on m.tenant_id = o.tenant_id and m.user_id = o.user_id
                             where o.tournament_id = t.tournament_id and o.team_id = t.id
                               and o.user_id <> $2 and m.left_at is null) as "last!"
           from participants p
           join teams t on t.tournament_id = p.tournament_id and t.id = p.team_id
           where p.tournament_id = $1 and p.user_id = $2"#,
        tournament.get(),
        user_id
    )
    .fetch_optional(&mut *conn)
    .await?;
    let Some(team) = team.filter(|t| t.seeded) else {
        return Ok(false);
    };
    // Nos times, o capitão que sai passa a capitania ao jogador do time
    // inscrito há mais tempo que continua na comunidade.
    hand_over(conn, tournament, team.id, user_id, true).await?;
    if team.last {
        withdraw(conn, tournament, team.id, by).await?;
    }
    Ok(true)
}

/// O time de um usuário num torneio (`of_user_in`).
pub struct UserTeam {
    /// O time dele, com os jogadores; `None` sem inscrição ou sem time.
    pub team: Option<StoredTeam>,
    /// O torneio já tem algum time (no TDM antes do início: quem está fora
    /// de todos fica de reserva).
    pub any_formed: bool,
}

/// O time de `user_id` em cada torneio de `tournaments` e se o torneio já
/// tem algum time, numa query só (a lista dos torneios de quem vê e o perfil
/// do membro), na ordem de `tournaments`. Os ids entram por `(select $1)`
/// para o Postgres reusar o plano genérico (`stage_rows` em `matches` diz
/// por quê).
pub async fn of_user_in(
    conn: &mut PgConnection,
    tournaments: &[TournamentId],
    user_id: i64,
) -> Result<Vec<UserTeam>, sqlx::Error> {
    let ids: Vec<i64> = tournaments.iter().map(|t| t.get()).collect();
    let rows = sqlx::query!(
        r#"select q.id as "tournament_id!",
                  exists(select 1 from teams x where x.tournament_id = q.id) as "formed!",
                  t.id as "id?", t.name as "name?", t.seed as "seed?", t.captain as "captain?",
                  t.withdrawn_at is not null as "withdrawn?",
                  p.user_id as "user_id?", m.nickname as "nickname?", coalesce(u.avatar_key, u.avatar_url) as "avatar_url?",
                  m.left_at is not null as "left?"
           from unnest((select $1::bigint[])) as q(id)
           left join participants me on me.tournament_id = q.id and me.user_id = $2
           left join teams t on t.tournament_id = q.id and t.id = me.team_id
           left join participants p on p.tournament_id = t.tournament_id and p.team_id = t.id
           left join tenant_members m on m.tenant_id = p.tenant_id and m.user_id = p.user_id
           left join users u on u.id = p.user_id
           order by q.id, p.user_id <> t.captain, p.registered_at, p.user_id"#,
        &ids,
        user_id
    )
    .fetch_all(conn)
    .await?;
    let mut formed: HashMap<i64, bool> = HashMap::new();
    let mut members: HashMap<i64, Vec<Row>> = HashMap::new();
    for r in rows {
        formed.insert(r.tournament_id, r.formed);
        let (Some(id), Some(captain), Some(user_id), Some(nickname)) =
            (r.id, r.captain, r.user_id, r.nickname)
        else {
            continue;
        };
        members.entry(r.tournament_id).or_default().push(Row {
            id,
            name: r.name,
            seed: r.seed,
            captain,
            withdrawn: r.withdrawn.unwrap_or_default(),
            user_id,
            nickname,
            avatar_url: r.avatar_url,
            left: r.left.unwrap_or_default(),
        });
    }
    Ok(ids
        .iter()
        .map(|id| UserTeam {
            team: group(members.remove(id).unwrap_or_default()).pop(),
            any_formed: formed.get(id).copied().unwrap_or_default(),
        })
        .collect())
}
