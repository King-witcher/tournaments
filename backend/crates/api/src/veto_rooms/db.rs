//! SQL da sala de veto. A sala vem por um token (de gestão, de espectador ou de
//! um lugar) numa query só, com os lugares, o pool e os campeões agregados: o
//! clique não faz uma ida para cada um.

use chrono::{DateTime, Utc};
use engine::core::{ChampionId, MapId};
use serde::Deserialize;
use sqlx::{PgConnection, PgPool, types::Json};

use crate::matches::{
    Side,
    veto::{Kind, NewVetoEvent, VetoRow},
};

/// A sala como vem do banco, pelo link `kind` (e `seat_id`, o lugar dele).
pub(super) struct RoomRow {
    pub id: i64,
    pub manage_token: String,
    pub spectator_token: String,
    pub team_size: i16,
    pub best_of: i16,
    pub map_mode: String,
    pub veto_order: String,
    pub pick_order: String,
    pub who_starts: String,
    pub champion_phase: bool,
    pub bans_per_series: i16,
    pub mirror: bool,
    pub same_team: bool,
    pub no_repeat: Option<String>,
    pub step_seconds: i32,
    pub team_pick_seconds: i32,
    pub expires_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub kind: String,
    pub seat_id: Option<i64>,
    pub seats: Json<Vec<SeatRow>>,
    pub maps: Json<Vec<MapRow>>,
    pub champions: Json<Vec<ChampionRow>>,
}

#[derive(Deserialize)]
pub(super) struct SeatRow {
    pub id: i64,
    pub side: Side,
    pub position: i16,
    pub token: String,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct MapRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
}

#[derive(Deserialize)]
pub(super) struct ChampionRow {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub active: bool,
}

/// A query da sala pelo token, a mesma para ler e para travar. Vencida, não
/// vem: como se não existisse, mesmo antes de a rotina apagá-la.
macro_rules! room_by_token {
    ($token:expr, $lock:literal) => {
        sqlx::query_as!(
            RoomRow,
            r#"with link as (
                   select room_id, 'player' as kind, id as seat_id
                   from veto_room_seats where token = $1
                   union all
                   select id, 'manager', null::bigint from veto_rooms where manage_token = $1
                   union all
                   select id, 'spectator', null::bigint from veto_rooms where spectator_token = $1
               )
               select r.id, r.manage_token, r.spectator_token, r.team_size, r.best_of, r.map_mode,
                      r.veto_order, r.pick_order, r.who_starts, r.champion_phase,
                      r.bans_per_series, r.mirror, r.same_team,
                      r.no_repeat, r.step_seconds, r.team_pick_seconds, r.expires_at, r.due_at,
                      link.kind as "kind!", link.seat_id as "seat_id?",
                      (select coalesce(jsonb_agg(jsonb_build_object(
                                  'id', s.id, 'side', s.side, 'position', s.position,
                                  'token', s.token, 'name', s.name)
                                  order by s.side, s.position), '[]'::jsonb)
                       from veto_room_seats s where s.room_id = r.id)
                          as "seats!: Json<Vec<SeatRow>>",
                      (select coalesce(jsonb_agg(jsonb_build_object(
                                  'id', m.id, 'slug', m.slug, 'name', m.name)
                                  order by p.position), '[]'::jsonb)
                       from veto_room_maps p join maps m on m.id = p.map_id
                       where p.room_id = r.id)
                          as "maps!: Json<Vec<MapRow>>",
                      (select coalesce(jsonb_agg(jsonb_build_object(
                                  'id', c.id, 'slug', c.slug, 'name', c.name, 'active', c.active)
                                  order by c.id), '[]'::jsonb)
                       from champions c where r.champion_phase)
                          as "champions!: Json<Vec<ChampionRow>>"
               from link join veto_rooms r on r.id = link.room_id
               where r.expires_at > now()"#
                + $lock,
            $token
        )
    };
}

/// A sala do link `token`, se existe e não venceu.
pub(super) async fn find(
    conn: &mut PgConnection,
    token: &str,
) -> Result<Option<RoomRow>, sqlx::Error> {
    room_by_token!(token, "").fetch_optional(conn).await
}

/// `find` com a sala travada até o fim da transação: a sala é o escopo, e nem
/// dois lados nem um clique e um sorteio andam ao mesmo tempo.
pub(super) async fn lock(
    conn: &mut PgConnection,
    token: &str,
) -> Result<Option<RoomRow>, sqlx::Error> {
    room_by_token!(token, " for update of r")
        .fetch_optional(conn)
        .await
}

/// O histórico da sala, na ordem, sem os nomes.
pub(super) async fn events(
    conn: &mut PgConnection,
    room: i64,
) -> Result<Vec<VetoRow>, crate::shared::ApiError> {
    let rows = sqlx::query!(
        "select id, game, kind, side, map_id, champion_id, seed, timeout, actor, player_id,
                pick_game, gave_way, created_at
         from veto_room_events where room_id = $1 order by id",
        room
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(VetoRow {
                id: r.id,
                game: r.game,
                kind: Kind::from_db(&r.kind)?,
                side: r.side.as_deref().map(Side::from_db).transpose()?,
                map: r.map_id.map(MapId),
                played_map: None,
                champion: r.champion_id.map(ChampionId),
                player: r.player_id,
                player_name: None,
                pick_game: r.pick_game,
                gave_way: r.gave_way,
                seed: r.seed,
                timeout: r.timeout,
                actor: r.actor,
                actor_name: None,
                created_at: r.created_at,
            })
        })
        .collect()
}

/// Grava um evento. Devolve o id e a hora gravada.
pub(super) async fn insert_event(
    conn: &mut PgConnection,
    room: i64,
    event: &NewVetoEvent,
) -> Result<(i64, DateTime<Utc>), sqlx::Error> {
    let row = sqlx::query!(
        "insert into veto_room_events
             (room_id, kind, game, side, map_id, champion_id, seed, timeout, actor, player_id,
              pick_game, gave_way, created_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, coalesce($13, now()))
         returning id, created_at",
        room,
        event.kind.as_str(),
        event.game,
        event.side.map(Side::as_str),
        event.map.map(|m| m.0),
        event.champion.map(|c| c.0),
        event.seed,
        event.timeout,
        event.actor,
        event.player,
        event.pick_game,
        event.gave_way,
        event.at
    )
    .fetch_one(conn)
    .await?;
    Ok((row.id, row.created_at))
}

/// O próximo sorteio da sala (`veto_rooms.due_at`).
pub(super) async fn set_due(
    conn: &mut PgConnection,
    room: i64,
    due: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    sqlx::query!("update veto_rooms set due_at = $2 where id = $1", room, due)
        .execute(conn)
        .await?;
    Ok(())
}

/// A sala passa a valer `hours` horas a partir de agora. Devolve até quando.
pub(super) async fn extend(
    conn: &mut PgConnection,
    room: i64,
    hours: i32,
) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar!(
        "update veto_rooms set expires_at = now() + make_interval(hours => $2)
         where id = $1 returning expires_at",
        room,
        hours
    )
    .fetch_one(conn)
    .await
}

/// O nome de um lugar, que só é definido uma vez.
pub(super) async fn set_name(
    conn: &mut PgConnection,
    seat: i64,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update veto_room_seats set name = $2 where id = $1",
        seat,
        name
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Quantas salas sem login estão valendo.
pub(super) async fn anonymous_active(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select count(*) as "count!" from veto_rooms
           where created_by is null and expires_at > now()"#
    )
    .fetch_one(conn)
    .await
}

/// Uma sala a criar: a configuração, já conferida, e os tokens.
pub(super) struct NewRoom<'a> {
    pub created_by: Option<i64>,
    pub manage_token: &'a str,
    pub spectator_token: &'a str,
    pub team_size: i16,
    pub best_of: i16,
    pub map_mode: &'a str,
    pub veto_order: &'a str,
    pub pick_order: &'a str,
    pub who_starts: &'a str,
    pub champion_phase: bool,
    pub bans_per_series: i16,
    pub mirror: bool,
    pub same_team: bool,
    pub no_repeat: Option<&'a str>,
    pub step_seconds: i32,
    pub team_pick_seconds: i32,
    /// Horas até vencer se ninguém começar.
    pub valid_hours: i32,
    /// O pool, na ordem.
    pub pool: &'a [MapId],
    /// Os lugares: lado e posição, com o token de cada um.
    pub seats: Vec<(Side, i16, String)>,
}

/// Grava a sala, os lugares e o pool. Quem chama faz o commit.
pub(super) async fn create(conn: &mut PgConnection, new: &NewRoom<'_>) -> Result<(), sqlx::Error> {
    let room = sqlx::query_scalar!(
        "insert into veto_rooms
             (created_by, manage_token, spectator_token, team_size, best_of, map_mode, veto_order,
              champion_phase, bans_per_series, mirror, same_team, no_repeat, step_seconds,
              team_pick_seconds, expires_at, pick_order, who_starts)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                 now() + make_interval(hours => $15), $16, $17)
         returning id",
        new.created_by,
        new.manage_token,
        new.spectator_token,
        new.team_size,
        new.best_of,
        new.map_mode,
        new.veto_order,
        new.champion_phase,
        new.bans_per_series,
        new.mirror,
        new.same_team,
        new.no_repeat,
        new.step_seconds,
        new.team_pick_seconds,
        new.valid_hours,
        new.pick_order,
        new.who_starts
    )
    .fetch_one(&mut *conn)
    .await?;
    let sides: Vec<String> = new.seats.iter().map(|s| s.0.as_str().to_owned()).collect();
    let positions: Vec<i16> = new.seats.iter().map(|s| s.1).collect();
    let tokens: Vec<String> = new.seats.iter().map(|s| s.2.clone()).collect();
    sqlx::query!(
        "insert into veto_room_seats (room_id, side, position, token)
         select $1, * from unnest($2::text[], $3::smallint[], $4::text[])",
        room,
        &sides,
        &positions,
        &tokens
    )
    .execute(&mut *conn)
    .await?;
    let maps: Vec<i64> = new.pool.iter().map(|m| m.0).collect();
    sqlx::query!(
        "insert into veto_room_maps (room_id, map_id, position)
         select $1, map_id, position::smallint
         from unnest($2::bigint[]) with ordinality as pool (map_id, position)",
        room,
        &maps
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O que o socket precisa saber de um link.
#[derive(Clone, Copy, Debug)]
pub struct RoomAccess {
    pub room: i64,
    pub spectator: bool,
    pub expires_at: DateTime<Utc>,
}

/// O link `token`, se a sala existe e não venceu.
pub async fn access(db: &PgPool, token: &str) -> Result<Option<RoomAccess>, sqlx::Error> {
    let row = sqlx::query!(
        r#"with link as (
               select room_id, false as spectator from veto_room_seats where token = $1
               union all
               select id, false from veto_rooms where manage_token = $1
               union all
               select id, true from veto_rooms where spectator_token = $1
           )
           select r.id, link.spectator as "spectator!", r.expires_at
           from link join veto_rooms r on r.id = link.room_id
           where r.expires_at > now()"#,
        token
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| RoomAccess {
        room: r.id,
        spectator: r.spectator,
        expires_at: r.expires_at,
    }))
}

/// O próximo sorteio de todas as salas valendo em `now`.
pub(super) async fn next_due(
    db: &PgPool,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        "select min(due_at) from veto_rooms where due_at is not null and expires_at > $1",
        now
    )
    .fetch_one(db)
    .await
}

/// As salas com sorteio vencido em `now`, pelo token de gestão, as mais
/// atrasadas primeiro; até `limit`, o resto fica para a passada seguinte.
pub(super) async fn due_tokens(
    db: &PgPool,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "select manage_token from veto_rooms
         where due_at <= $1 and expires_at > $1 order by due_at limit $2",
        now,
        limit
    )
    .fetch_all(db)
    .await
}

/// Apaga as salas vencidas em `now`, com os lugares, o pool e o histórico.
pub(super) async fn delete_expired(db: &PgPool, now: DateTime<Utc>) -> Result<u64, sqlx::Error> {
    Ok(
        sqlx::query!("delete from veto_rooms where expires_at <= $1", now)
            .execute(db)
            .await?
            .rows_affected(),
    )
}
