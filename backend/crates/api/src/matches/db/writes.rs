//! Escritas em `match_results`: enviar, confirmar, contestar, o organizador
//! definir, o W.O. automático e anular. Quem chama trava o torneio: as
//! posições do replay são
//! `max + 1`.

use engine::core::{Outcome, ParticipantId};
use sqlx::PgConnection;

use super::MatchRef;
use crate::matches::result::{Confirmation, Side};

/// Resultado novo de uma partida pronta.
pub struct NewResult {
    pub best_of: u8,
    pub sides: (ParticipantId, ParticipantId),
    pub outcome: Outcome,
    /// Mapas de `a` e de `b`; `None` no W.O.
    pub score: Option<(u8, u8)>,
    /// Quem enviou ou lançou.
    pub by: i64,
}

fn wins(score: Option<(u8, u8)>) -> (Option<i16>, Option<i16>) {
    match score {
        Some((a, b)) => (Some(i16::from(a)), Some(i16::from(b))),
        None => (None, None),
    }
}

/// O placar que um jogador do lado `side` enviou: pendente até o adversário
/// responder ou acabar a janela (`contest_minutes`). Devolve o id.
pub async fn insert_report(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    new: &NewResult,
    side: Side,
    contest_minutes: i32,
) -> Result<i64, sqlx::Error> {
    let (a_wins, b_wins) = wins(new.score);
    sqlx::query_scalar!(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, a_wins, b_wins, reported_side, reported_by, confirm_by)
           values ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, $9, $10, $11, $12,
                   now() + make_interval(mins => $13))
           returning id"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        i16::from(new.best_of),
        new.sides.0.0,
        new.sides.1.0,
        new.outcome.winner().0,
        a_wins,
        b_wins,
        side.as_str(),
        new.by,
        contest_minutes
    )
    .fetch_one(conn)
    .await
}

/// Confirma o resultado pendente ou contestado `id` (`via` o adversário ou o
/// organizador `by`): passa a valer na chave, no fim do replay.
pub async fn confirm(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    id: i64,
    via: Confirmation,
    by: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update match_results
           set status = 'confirmed', confirmed_via = $3, confirmed_by = $4, confirmed_at = now(),
               contested_by = null, contested_at = null, contest_comment = null,
               position = (select coalesce(max(position), 0) + 1
                           from match_results where tournament_id = $2)
           where id = $1 and status in ('pending', 'contested')"#,
        id,
        at.tournament.get(),
        via.as_str(),
        by
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// O adversário `by` contesta o pendente `id`: sai da janela e espera o
/// organizador.
pub async fn contest(
    conn: &mut PgConnection,
    id: i64,
    by: i64,
    comment: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update match_results
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

/// O organizador define o resultado da partida: já confirmado. Substitui o
/// pendente ou contestado (que entra no fim do replay) e, na correção de um
/// confirmado, mantém a posição dele. Devolve o id.
pub async fn put_by_organizer(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    new: &NewResult,
) -> Result<i64, sqlx::Error> {
    let (a_wins, b_wins) = wins(new.score);
    sqlx::query_scalar!(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, a_wins, b_wins, reported_by, confirmed_via, confirmed_by, confirmed_at,
                position)
           values ($1, $2, $3, $4, $5, $6, $7, 'confirmed', $8, $9, $10, $11, 'organizer', $11,
                   now(), (select coalesce(max(position), 0) + 1
                           from match_results where tournament_id = $2))
           on conflict (tournament_id, stage, match_key) do update
           set side_a = excluded.side_a, side_b = excluded.side_b, status = 'confirmed',
               winner = excluded.winner, a_wins = excluded.a_wins, b_wins = excluded.b_wins,
               reported_side = null, reported_by = excluded.reported_by, reported_at = now(),
               confirm_by = null, contested_by = null, contested_at = null,
               contest_comment = null, confirmed_via = 'organizer',
               confirmed_by = excluded.confirmed_by, confirmed_at = now(),
               position = coalesce(match_results.position, excluded.position)
           returning id"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        i16::from(new.best_of),
        new.sides.0.0,
        new.sides.1.0,
        new.outcome.winner().0,
        a_wins,
        b_wins,
        new.by
    )
    .fetch_one(conn)
    .await
}

/// W.O. automático para `winner` (o outro lado desistiu ou saiu da
/// comunidade): já confirmado, sem autor. Substitui o pendente ou contestado
/// da partida e entra no fim do replay. Devolve o id.
pub async fn put_walkover(
    conn: &mut PgConnection,
    at: &MatchRef<'_>,
    best_of: u8,
    (a, b): (ParticipantId, ParticipantId),
    winner: ParticipantId,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"insert into match_results
               (tenant_id, tournament_id, stage, match_key, best_of, side_a, side_b, status,
                winner, confirmed_via, confirmed_at, position)
           values ($1, $2, $3, $4, $5, $6, $7, 'confirmed', $8, 'withdrawal', now(),
                   (select coalesce(max(position), 0) + 1
                    from match_results where tournament_id = $2))
           on conflict (tournament_id, stage, match_key) do update
           set side_a = excluded.side_a, side_b = excluded.side_b, status = 'confirmed',
               winner = excluded.winner, a_wins = null, b_wins = null, reported_side = null,
               reported_by = null, reported_at = now(), confirm_by = null, contested_by = null,
               contested_at = null, contest_comment = null, confirmed_via = 'withdrawal',
               confirmed_by = null, confirmed_at = now(), position = excluded.position
           where match_results.status <> 'confirmed'
           returning id"#,
        at.tenant.get(),
        at.tournament.get(),
        at.stage,
        at.key,
        i16::from(best_of),
        a.0,
        b.0,
        winner.0
    )
    .fetch_one(conn)
    .await
}

/// Apaga o resultado `id` (pendente ou contestado que uma correção deixou sem
/// partida).
pub async fn void(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "delete from match_results where id = $1 and status <> 'confirmed'",
        id
    )
    .execute(conn)
    .await?;
    Ok(())
}
