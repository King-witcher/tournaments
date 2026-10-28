//! Detalhe por mapa de um resultado (`match_games`).

use sqlx::PgConnection;

use crate::matches::outcome::NewGame;

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

/// Troca o detalhe do resultado `result` por `games` (vazio: sem detalhe).
pub async fn replace_games(
    conn: &mut PgConnection,
    result: i64,
    games: &[NewGame],
) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from match_games where result_id = $1", result)
        .execute(&mut *conn)
        .await?;
    let maps: Vec<i64> = games.iter().map(|g| g.map.0).collect();
    let a: Vec<i16> = games.iter().map(|g| g.a_frags).collect();
    let b: Vec<i16> = games.iter().map(|g| g.b_frags).collect();
    sqlx::query!(
        r#"insert into match_games (result_id, number, map_id, a_frags, b_frags)
           select $1, g.n::smallint, g.map, g.a, g.b
           from unnest($2::bigint[], $3::smallint[], $4::smallint[])
                with ordinality as g(map, a, b, n)"#,
        result,
        &maps,
        &a,
        &b
    )
    .execute(conn)
    .await?;
    Ok(())
}
