//! Escritas em `match_results`, a série decidida: pelos mapas, pelo W.O. do
//! organizador, pelo W.O. automático, e a correção que a reabre. Quem chama
//! trava o torneio: as posições do replay são `max + 1`.

use chrono::{DateTime, Utc};
use engine::core::ParticipantId;
use sqlx::PgConnection;

use super::MatchRef;

/// A série que os mapas confirmados decidiram para `winner`, com os mapas de
/// `a` e de `b`, confirmada em `at_time`. Entra no fim do replay; na correção
/// de uma série já decidida (pelos mapas ou por W.O.), fica com a posição
/// dela.
pub async fn put_decided(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    best_of: u8,
    (a, b): (ParticipantId, ParticipantId),
    winner: ParticipantId,
    (a_wins, b_wins): (u8, u8),
    at_time: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, a_wins, b_wins, reported_at, confirmed_via, confirmed_at, position)
           values ($1, $2, $3, $4, $5, $6, $7, 'confirmed', $8, $9, $10, $11, 'maps', $11,
                   (select coalesce(max(position), 0) + 1
                    from match_results where tournament_id = $2))
           on conflict (tournament_id, stage, match_key) do update
           set side_a = excluded.side_a, side_b = excluded.side_b, winner = excluded.winner,
               a_wins = excluded.a_wins, b_wins = excluded.b_wins, reported_side = null,
               reported_by = null, reported_at = excluded.reported_at, confirm_by = null,
               contested_by = null, contested_at = null, contest_comment = null,
               confirmed_via = 'maps', confirmed_by = null, confirmed_at = excluded.confirmed_at,
               position = match_results.position"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        i16::from(best_of),
        a.0,
        b.0,
        winner.0,
        i16::from(a_wins),
        i16::from(b_wins),
        at_time
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// W.O. do organizador `by` para `winner`: já confirmado. Entra no fim do
/// replay; na correção de uma série decidida, fica com a posição dela.
pub async fn put_by_organizer(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    best_of: u8,
    (a, b): (ParticipantId, ParticipantId),
    winner: ParticipantId,
    by: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, reported_by, confirmed_via, confirmed_by, confirmed_at, position)
           values ($1, $2, $3, $4, $5, $6, $7, 'confirmed', $8, $9, 'organizer', $9, now(),
                   (select coalesce(max(position), 0) + 1
                    from match_results where tournament_id = $2))
           on conflict (tournament_id, stage, match_key) do update
           set side_a = excluded.side_a, side_b = excluded.side_b, winner = excluded.winner,
               a_wins = null, b_wins = null, reported_side = null,
               reported_by = excluded.reported_by, reported_at = now(), confirm_by = null,
               contested_by = null, contested_at = null, contest_comment = null,
               confirmed_via = 'organizer', confirmed_by = excluded.confirmed_by,
               confirmed_at = now(), position = match_results.position"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        i16::from(best_of),
        a.0,
        b.0,
        winner.0,
        by
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// W.O. automático para `winner` (o outro lado desistiu ou saiu da
/// comunidade): já confirmado, sem autor, no fim do replay.
pub async fn put_walkover(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    best_of: u8,
    (a, b): (ParticipantId, ParticipantId),
    winner: ParticipantId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, confirmed_via, confirmed_at, position)
           values ($1, $2, $3, $4, $5, $6, $7, 'confirmed', $8, 'withdrawal', now(),
                   (select coalesce(max(position), 0) + 1
                    from match_results where tournament_id = $2))"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        i16::from(best_of),
        a.0,
        b.0,
        winner.0
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Apaga a série decidida `id`: uma correção de mapa a deixou sem decisão.
pub async fn remove_result(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("delete from match_results where id = $1", id)
        .execute(conn)
        .await?;
    Ok(())
}
