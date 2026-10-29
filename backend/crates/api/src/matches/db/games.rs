//! Mapas da série (`match_games`): ler, enviar, confirmar, contestar, o
//! organizador lançar e anular.

use chrono::{DateTime, Utc};
use engine::core::{MapId, ParticipantId};
use sqlx::PgConnection;

use super::MatchRef;
use crate::{
    matches::{
        games::{GameRow, StoredGame},
        result::{Confirmation, Side},
    },
    shared::ApiError,
    tournaments::TournamentId,
};

/// Um mapa a gravar.
pub struct NewGame {
    pub number: i16,
    pub sides: (ParticipantId, ParticipantId),
    pub winner: ParticipantId,
    pub map: Option<MapId>,
    /// Frags de `a` e de `b`.
    pub frags: Option<(i16, i16)>,
}

/// Os mapas da fase `stage`, por partida e na ordem.
pub async fn games(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
) -> Result<Vec<StoredGame>, ApiError> {
    sqlx::query_as!(
        GameRow,
        r#"select g.id, g.match_key, g.number, g.side_a, g.side_b, g.winner, g.map_id,
                  m.slug as "map_slug?", m.name as "map_name?", g.a_frags, g.b_frags, g.status,
                  g.reported_side, g.reported_at, g.confirm_by, g.contested_at,
                  g.contest_comment, g.confirmed_via, g.confirmed_at
           from match_games g
           left join maps m on m.id = g.map_id
           where g.tournament_id = $1 and g.stage = $2
           order by g.match_key, g.number"#,
        tournament.get(),
        stage
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(StoredGame::try_from)
    .collect()
}

/// O mapa que um jogador do lado `side` enviou: pendente até o adversário
/// responder ou acabar a janela (`contest_minutes`). Devolve o prazo.
pub async fn insert_report(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    game: &NewGame,
    side: Side,
    by: i64,
    contest_minutes: i32,
) -> Result<DateTime<Utc>, sqlx::Error> {
    let (a_frags, b_frags) = game.frags.unzip();
    sqlx::query_scalar!(
        r#"insert into match_games
               (tenant_id, tournament_id, stage, match_key, number, side_a, side_b, winner,
                map_id, a_frags, b_frags, status, reported_side, reported_by, confirm_by)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending', $12, $13,
                   now() + make_interval(mins => $14))
           returning confirm_by as "confirm_by!""#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        game.number,
        game.sides.0.0,
        game.sides.1.0,
        game.winner.0,
        game.map.map(|m| m.0),
        a_frags,
        b_frags,
        side.as_str(),
        by,
        contest_minutes
    )
    .fetch_one(conn)
    .await
}

/// O organizador `by` lança o mapa, já confirmado: novo ou no lugar do que
/// havia com o mesmo número.
pub async fn put_game(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    game: &NewGame,
    by: i64,
) -> Result<(), sqlx::Error> {
    let (a_frags, b_frags) = game.frags.unzip();
    sqlx::query!(
        r#"insert into match_games
               (tenant_id, tournament_id, stage, match_key, number, side_a, side_b, winner,
                map_id, a_frags, b_frags, status, reported_by, confirmed_via, confirmed_by,
                confirmed_at)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'confirmed', $12,
                   'organizer', $12, now())
           on conflict (tournament_id, stage, match_key, number) do update
           set side_a = excluded.side_a, side_b = excluded.side_b, winner = excluded.winner,
               map_id = excluded.map_id, a_frags = excluded.a_frags, b_frags = excluded.b_frags,
               status = 'confirmed', reported_side = null, reported_by = excluded.reported_by,
               reported_at = now(), confirm_by = null, contested_by = null,
               contested_at = null, contest_comment = null, confirmed_via = 'organizer',
               confirmed_by = excluded.confirmed_by, confirmed_at = now()"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        game.number,
        game.sides.0.0,
        game.sides.1.0,
        game.winner.0,
        game.map.map(|m| m.0),
        a_frags,
        b_frags,
        by
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Confirma o mapa pendente ou contestado `id` (`via` o adversário ou o
/// organizador `by`), como foi enviado.
pub async fn confirm_game(
    conn: &mut PgConnection,
    id: i64,
    via: Confirmation,
    by: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update match_games
           set status = 'confirmed', confirmed_via = $2, confirmed_by = $3, confirmed_at = now(),
               contested_by = null, contested_at = null, contest_comment = null
           where id = $1 and status in ('pending', 'contested')"#,
        id,
        via.as_str(),
        by
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O adversário `by` contesta o mapa pendente `id`: sai da janela e espera o
/// organizador.
pub async fn contest_game(
    conn: &mut PgConnection,
    id: i64,
    by: i64,
    comment: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update match_games
           set status = 'contested', contested_by = $2, contested_at = now(), contest_comment = $3
           where id = $1 and status = 'pending'"#,
        id,
        by,
        comment
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Apaga os mapas `ids` (anulados).
pub async fn void_games(conn: &mut PgConnection, ids: &[i64]) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from match_games where id = any($1)", ids)
        .execute(conn)
        .await?;
    Ok(())
}

/// Um jogador da escalação de um mapa.
pub struct LineupRow {
    pub game_id: i64,
    pub side: String,
    pub user_id: i64,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub left: bool,
}

/// A escalação de cada mapa da partida `key` da fase `stage` (quem jogou por
/// lado), por mapa e lado.
pub async fn lineups(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
    key: &str,
) -> Result<Vec<LineupRow>, sqlx::Error> {
    sqlx::query_as!(
        LineupRow,
        r#"select gp.game_id, gp.side, gp.user_id, m.nickname, u.avatar_url,
                  m.left_at is not null as "left!"
           from match_game_players gp
           join match_games g on g.id = gp.game_id
           join tenant_members m on m.tenant_id = gp.tenant_id and m.user_id = gp.user_id
           join users u on u.id = gp.user_id
           where g.tournament_id = $1 and g.stage = $2 and g.match_key = $3
           order by gp.game_id, gp.side, m.nickname, gp.user_id"#,
        tournament.get(),
        stage,
        key
    )
    .fetch_all(conn)
    .await
}
