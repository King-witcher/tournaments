//! Mapas da série (`match_games`): ler, enviar, confirmar, contestar, aceitar a
//! contestação, o organizador lançar e anular, e o aviso atrasado à organização.

use chrono::{DateTime, Utc};
use engine::{
    core::{MapId, ParticipantId},
    series,
};
use sqlx::{PgConnection, PgPool};

use super::MatchRef;
use crate::{
    matches::{
        games::{GameRow, StoredGame},
        result::{Confirmation, Side},
    },
    shared::ApiError,
    tenants::TenantId,
    tournaments::TournamentId,
};

/// Um mapa a gravar.
pub struct NewGame {
    pub number: i16,
    pub sides: (ParticipantId, ParticipantId),
    /// O lado que fez mais frags: o vencedor vem do placar, nunca do envio.
    pub winner: Side,
    pub map: MapId,
    /// Frags de `a` e de `b`.
    pub frags: (i16, i16),
}

impl NewGame {
    /// O participante que venceu.
    pub fn winner_id(&self) -> ParticipantId {
        self.winner.of(self.sides)
    }

    /// O mapa como a série o vê, com o estado que ele terá.
    pub fn series_game(&self, status: series::GameStatus) -> series::SeriesGame {
        series::SeriesGame {
            winner: self.winner.into(),
            status,
            map: Some(self.map),
        }
    }
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
                  g.reported_side, g.reported_by, g.reported_at, g.confirm_by, g.contested_by,
                  g.contested_at, g.contest_comment, g.confirmed_via, g.confirmed_at
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

/// Quanto dura o prazo de um envio: a partida liberada em `released` (ver
/// `release`) dura `game_minutes`, e a janela de contestação
/// (`contest_minutes`) começa no envio ou no fim dessa duração, o que vier
/// por último.
pub struct Deadline {
    pub released: Option<DateTime<Utc>>,
    pub game_minutes: i32,
    pub contest_minutes: i32,
}

/// A partida que um jogador do lado `side` enviou: pendente até o adversário
/// responder ou acabar o prazo (`deadline`). Devolve o prazo.
pub async fn insert_report(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    game: &NewGame,
    side: Side,
    by: i64,
    deadline: &Deadline,
) -> Result<DateTime<Utc>, sqlx::Error> {
    let (a_frags, b_frags) = game.frags;
    // `greatest` ignora o nulo: sem `released`, o prazo conta do envio.
    sqlx::query_scalar!(
        r#"insert into match_games
               (tenant_id, tournament_id, stage, match_key, number, side_a, side_b, winner,
                map_id, a_frags, b_frags, status, reported_side, reported_by, confirm_by)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending', $12, $13,
                   greatest(now(), $14::timestamptz + make_interval(mins => $15))
                       + make_interval(mins => $16))
           returning confirm_by as "confirm_by!""#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        game.number,
        game.sides.0.0,
        game.sides.1.0,
        game.winner_id().0,
        game.map.0,
        a_frags,
        b_frags,
        side.as_str(),
        by,
        deadline.released,
        deadline.game_minutes,
        deadline.contest_minutes
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
    let (a_frags, b_frags) = game.frags;
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
               contested_at = null, contest_comment = null, contest_notify_at = null,
               confirmed_via = 'organizer', confirmed_by = excluded.confirmed_by,
               confirmed_at = now()"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        game.number,
        game.sides.0.0,
        game.sides.1.0,
        game.winner_id().0,
        game.map.0,
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
               contested_by = null, contested_at = null, contest_comment = null,
               contest_notify_at = null
           where id = $1 and status in ('pending', 'contested')"#,
        id,
        via.as_str(),
        by
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Um jogador da série (`by`) contesta a partida `id`, pendente ou já
/// confirmada (a contestação tardia): sai do prazo, perde a confirmação e
/// espera o capitão adversário ou o organizador, que é avisado depois de
/// `notify_after` segundos.
pub async fn contest_game(
    conn: &mut PgConnection,
    id: i64,
    by: i64,
    comment: &str,
    notify_after: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update match_games
           set status = 'contested', contested_by = $2, contested_at = now(), contest_comment = $3,
               contest_notify_at = now() + make_interval(secs => $4),
               confirmed_via = null, confirmed_by = null, confirmed_at = null
           where id = $1 and status in ('pending', 'confirmed')"#,
        id,
        by,
        comment,
        notify_after
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O capitão `by`, do lado `side`, aceita a contestação da partida `id`: ela
/// passa a valer com o resultado que ele deu, confirmada como resposta do
/// adversário. O prazo gravado é o que uma partida enviada agora teria, para a
/// linha continuar dizendo que foi enviada por um jogador.
pub async fn accept_contest(
    conn: &mut PgConnection,
    id: i64,
    game: &NewGame,
    side: Side,
    by: i64,
    contest_minutes: i32,
) -> Result<(), sqlx::Error> {
    let (a_frags, b_frags) = game.frags;
    sqlx::query!(
        r#"update match_games
           set winner = $2, map_id = $3, a_frags = $4, b_frags = $5, reported_side = $6,
               reported_by = $7, reported_at = now(),
               confirm_by = now() + make_interval(mins => $8), contested_by = null,
               contested_at = null, contest_comment = null, contest_notify_at = null,
               status = 'confirmed', confirmed_via = 'opponent', confirmed_by = $7,
               confirmed_at = now()
           where id = $1 and status = 'contested'"#,
        id,
        game.winner_id().0,
        game.map.0,
        a_frags,
        b_frags,
        side.as_str(),
        by,
        contest_minutes
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Os torneios com partida contestada cujo aviso à organização venceu em `now`.
pub async fn contest_notices_due(
    db: &PgPool,
    now: DateTime<Utc>,
) -> Result<Vec<(TenantId, String)>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"select distinct t.id, t.tenant_id, t.slug
           from match_games g
           join tournaments t on t.id = g.tournament_id
           where g.contest_notify_at <= $1
           order by t.id"#,
        now
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| (TenantId::new(r.tenant_id), r.slug))
    .collect())
}

/// Uma partida contestada cujo aviso à organização venceu.
pub struct ContestNotice {
    pub game: i64,
    pub match_key: String,
    pub number: i16,
    /// Quem contestou; `None` se a conta foi excluída.
    pub contested_by: Option<i64>,
    /// O apelido de quem contestou.
    pub player: Option<String>,
}

/// Tira da fila os avisos à organização do torneio vencidos em `now` e os
/// devolve: cada um sai uma vez só.
pub async fn take_contest_notices(
    conn: &mut PgConnection,
    tournament: TournamentId,
    now: DateTime<Utc>,
) -> Result<Vec<ContestNotice>, sqlx::Error> {
    sqlx::query_as!(
        ContestNotice,
        r#"with due as (
               update match_games set contest_notify_at = null
               where tournament_id = $1 and contest_notify_at <= $2
               returning id, tenant_id, match_key, number, contested_by
           )
           select d.id as game, d.match_key, d.number, d.contested_by as "contested_by?",
                  m.nickname as "player?"
           from due d
           left join tenant_members m on m.tenant_id = d.tenant_id and m.user_id = d.contested_by
           order by d.match_key, d.number"#,
        tournament.get(),
        now
    )
    .fetch_all(conn)
    .await
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
