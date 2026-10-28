//! Detalhe por mapa de um resultado (`match_games`).

use sqlx::PgConnection;

/// Um mapa do detalhe, com o mapa já resolvido.
pub struct GameRow {
    pub number: i16,
    pub map_slug: String,
    pub map_name: String,
    pub a_frags: i16,
    pub b_frags: i16,
}

/// Os mapas do resultado `result`, na ordem jogada.
pub async fn games(conn: &mut PgConnection, result: i64) -> Result<Vec<GameRow>, sqlx::Error> {
    sqlx::query_as!(
        GameRow,
        r#"select g.number, m.slug as map_slug, m.name as map_name, g.a_frags, g.b_frags
           from match_games g
           join maps m on m.id = g.map_id
           where g.result_id = $1
           order by g.number"#,
        result
    )
    .fetch_all(conn)
    .await
}
