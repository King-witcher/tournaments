//! Histórico das partidas (`match_events`).

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{PgConnection, types::Json};

use super::{LineupRow, MatchRef};
use crate::{matches::history::NewEvent, tournaments::TournamentId};

/// Um evento do histórico, com o apelido de quem agiu.
#[derive(Deserialize, Debug)]
pub struct EventRow {
    pub kind: String,
    pub actor: Option<i64>,
    /// Apelido no tenant (ou o nome exibido, se saiu do tenant).
    pub actor_name: Option<String>,
    pub actor_avatar: Option<String>,
    /// Mapa a que o evento se refere; `None`: a série inteira.
    pub game: Option<i16>,
    pub winner: Option<i64>,
    pub winner_maps: Option<i16>,
    pub loser_maps: Option<i16>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// O histórico da partida `key` da fase `stage`, do mais antigo ao mais novo,
/// e a escalação de cada mapa dela (quem jogou por lado), por mapa e lado,
/// numa query só (os dois agregados em JSON): toda resposta da partida lê os
/// dois.
pub async fn match_history(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
    key: &str,
) -> Result<(Vec<EventRow>, Vec<LineupRow>), sqlx::Error> {
    let row = sqlx::query!(
        r#"select
               (select json_agg(e order by e.created_at, e.id) from (
                    select e.id, e.kind, e.actor, coalesce(m.nickname, u.display_name) as actor_name,
                           u.avatar_url as actor_avatar, e.game, e.winner, e.winner_maps,
                           e.loser_maps, e.comment, e.created_at
                    from match_events e
                    left join users u on u.id = e.actor
                    left join tenant_members m on m.tenant_id = e.tenant_id and m.user_id = e.actor
                    where e.tournament_id = $1 and e.stage = $2 and e.match_key = $3) e)
                   as "events: Json<Vec<EventRow>>",
               (select json_agg(l order by l.game_id, l.side, l.nickname, l.user_id) from (
                    select gp.game_id, gp.side, gp.user_id, m.nickname, u.avatar_url,
                           m.left_at is not null as "left"
                    from match_game_players gp
                    join match_games g on g.id = gp.game_id
                    join tenant_members m on m.tenant_id = gp.tenant_id and m.user_id = gp.user_id
                    join users u on u.id = gp.user_id
                    where g.tournament_id = $1 and g.stage = $2 and g.match_key = $3) l)
                   as "lineups: Json<Vec<LineupRow>>""#,
        tournament.get(),
        stage,
        key
    )
    .fetch_one(conn)
    .await?;
    Ok((
        row.events.map(|j| j.0).unwrap_or_default(),
        row.lineups.map(|j| j.0).unwrap_or_default(),
    ))
}

/// Grava um evento da partida `at`.
pub async fn insert_event(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    event: &NewEvent<'_>,
) -> Result<(), sqlx::Error> {
    let (winner_maps, loser_maps) = event.maps.unzip();
    sqlx::query!(
        r#"insert into match_events
               (tenant_id, tournament_id, stage, match_key, kind, actor, game, winner,
                winner_maps, loser_maps, comment, created_at)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, coalesce($12, now()))"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        event.kind.as_str(),
        event.actor,
        event.game,
        event.winner.map(|w| w.0),
        winner_maps.map(i16::from),
        loser_maps.map(i16::from),
        event.comment,
        event.at
    )
    .execute(conn)
    .await?;
    Ok(())
}
