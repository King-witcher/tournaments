//! Histórico das partidas (`match_events`).

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use crate::tournaments::TournamentId;

/// Um evento do histórico, com o apelido de quem agiu.
pub struct EventRow {
    pub kind: String,
    pub actor: Option<i64>,
    /// Apelido no tenant (ou o nome exibido, se saiu do tenant).
    pub actor_name: Option<String>,
    pub actor_avatar: Option<String>,
    pub winner: Option<i64>,
    pub winner_maps: Option<i16>,
    pub loser_maps: Option<i16>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Histórico da partida `key` da fase `stage`, do mais antigo ao mais novo.
pub async fn events(
    conn: &mut PgConnection,
    tournament: TournamentId,
    stage: i16,
    key: &str,
) -> Result<Vec<EventRow>, sqlx::Error> {
    sqlx::query_as!(
        EventRow,
        r#"select e.kind, e.actor, coalesce(m.nickname, u.display_name) as actor_name,
                  u.avatar_url as actor_avatar, e.winner, e.winner_maps, e.loser_maps,
                  e.comment, e.created_at
           from match_events e
           left join users u on u.id = e.actor
           left join tenant_members m on m.tenant_id = e.tenant_id and m.user_id = e.actor
           where e.tournament_id = $1 and e.stage = $2 and e.match_key = $3
           order by e.created_at, e.id"#,
        tournament.get(),
        stage,
        key
    )
    .fetch_all(conn)
    .await
}
