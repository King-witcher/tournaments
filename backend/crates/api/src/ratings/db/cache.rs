//! SQL do recálculo: as marcas de "desatualizado", o lock por tenant, o
//! histórico que o engine reprocessa e o cache (`ratings`, `rating_history`).

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::{
    ratings::history::{Cache, HistoryRow, RatingRow, Source, SourceKind},
    tenants::TenantId,
};

/// Base das chaves do advisory lock do recálculo ("rati" e o id do tenant);
/// longe da chave do agendador.
const LOCK_BASE: i64 = 0x7261_7469_0000_0000;

/// Espera o recálculo de outra transação no mesmo tenant e trava até o fim
/// desta: dois recálculos nunca gravam o cache ao mesmo tempo, e o que vem
/// depois lê o histórico com o que o primeiro já viu.
pub async fn lock(conn: &mut PgConnection, tenant: TenantId) -> Result<(), sqlx::Error> {
    sqlx::query_scalar!(
        r#"select true as "locked!" from pg_advisory_xact_lock($1)"#,
        LOCK_BASE + tenant.get()
    )
    .fetch_one(conn)
    .await?;
    Ok(())
}

/// Se o tenant tem marca de rating desatualizado.
pub async fn is_dirty(db: &PgPool, tenant: TenantId) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from rating_dirty where tenant_id = $1) as "dirty!""#,
        tenant.get()
    )
    .fetch_one(db)
    .await
}

/// Apaga as marcas do tenant que esta transação vê; quantas eram.
pub async fn consume_dirty(conn: &mut PgConnection, tenant: TenantId) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        "delete from rating_dirty where tenant_id = $1",
        tenant.get()
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected())
}

/// Os tenants com marca, para o agendador.
pub async fn dirty_tenants(db: &PgPool) -> Result<Vec<TenantId>, sqlx::Error> {
    let ids = sqlx::query_scalar!("select distinct tenant_id from rating_dirty order by tenant_id")
        .fetch_all(db)
        .await?;
    Ok(ids.into_iter().map(TenantId::new).collect())
}

/// O histórico do tenant, sem ordem (`history::events` ordena): os mapas
/// confirmados dos torneios que contam, de todos os modos, com quem jogou por
/// cada lado, e os ajustes.
pub async fn sources(
    conn: &mut PgConnection,
    tenant: TenantId,
) -> Result<Vec<Source>, sqlx::Error> {
    let games = sqlx::query!(
        r#"select g.id, g.confirmed_at as "at!",
                  array_agg(p.user_id order by p.user_id)
                      filter (where (p.side = 'a') = (g.winner = g.side_a)) as "winners!",
                  array_agg(p.user_id order by p.user_id)
                      filter (where (p.side = 'a') <> (g.winner = g.side_a)) as "losers!"
           from match_games g
           join tournaments t on t.id = g.tournament_id
           join match_game_players p on p.game_id = g.id
           where t.tenant_id = $1 and t.counts_for_rating and g.status = 'confirmed'
           group by g.id"#,
        tenant.get()
    )
    .fetch_all(&mut *conn)
    .await?;
    let adjustments = sqlx::query!(
        "select id, user_id, delta, created_at from rating_adjustments where tenant_id = $1",
        tenant.get()
    )
    .fetch_all(&mut *conn)
    .await?;

    let games = games.into_iter().map(|g| Source {
        kind: SourceKind::Game {
            id: g.id,
            winners: g.winners,
            losers: g.losers,
        },
        at: g.at,
    });
    let adjustments = adjustments.into_iter().map(|a| Source {
        kind: SourceKind::Adjustment {
            id: a.id,
            user: a.user_id,
            delta: a.delta,
        },
        at: a.created_at,
    });
    Ok(games.chain(adjustments).collect())
}

/// O cache gravado, na ordem de `Cache`.
pub async fn read(conn: &mut PgConnection, tenant: TenantId) -> Result<Cache, sqlx::Error> {
    let ratings = sqlx::query_as!(
        RatingRow,
        r#"select user_id, rating, games, wins, losses, last_game_at from ratings
           where tenant_id = $1
           order by user_id"#,
        tenant.get()
    )
    .fetch_all(&mut *conn)
    .await?;
    let history = sqlx::query_as!(
        HistoryRow,
        r#"select user_id, seq, game_id, adjustment_id, won, delta, rating_after, at
           from rating_history
           where tenant_id = $1
           order by user_id, seq"#,
        tenant.get()
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(Cache { ratings, history })
}

/// Troca o cache gravado por `cache`.
pub async fn write(
    conn: &mut PgConnection,
    tenant: TenantId,
    cache: &Cache,
) -> Result<(), sqlx::Error> {
    let tenant = tenant.get();
    sqlx::query!("delete from rating_history where tenant_id = $1", tenant)
        .execute(&mut *conn)
        .await?;
    sqlx::query!("delete from ratings where tenant_id = $1", tenant)
        .execute(&mut *conn)
        .await?;

    let r = &cache.ratings;
    sqlx::query!(
        r#"insert into ratings
               (tenant_id, user_id, rating, games, wins, losses, last_game_at)
           select $1, * from unnest($2::bigint[], $3::int[], $4::int[], $5::int[], $6::int[],
                                    $7::timestamptz[])"#,
        tenant,
        &r.iter().map(|x| x.user_id).collect::<Vec<_>>(),
        &r.iter().map(|x| x.rating).collect::<Vec<_>>(),
        &r.iter().map(|x| x.games).collect::<Vec<_>>(),
        &r.iter().map(|x| x.wins).collect::<Vec<_>>(),
        &r.iter().map(|x| x.losses).collect::<Vec<_>>(),
        &r.iter().map(|x| x.last_game_at).collect::<Vec<_>>() as &[Option<DateTime<Utc>>],
    )
    .execute(&mut *conn)
    .await?;

    let h = &cache.history;
    sqlx::query!(
        r#"insert into rating_history
               (tenant_id, user_id, seq, game_id, adjustment_id, won, delta, rating_after, at)
           select $1, * from unnest($2::bigint[], $3::int[], $4::bigint[], $5::bigint[],
                                    $6::boolean[], $7::int[], $8::int[], $9::timestamptz[])"#,
        tenant,
        &h.iter().map(|x| x.user_id).collect::<Vec<_>>(),
        &h.iter().map(|x| x.seq).collect::<Vec<_>>(),
        &h.iter().map(|x| x.game_id).collect::<Vec<_>>() as &[Option<i64>],
        &h.iter().map(|x| x.adjustment_id).collect::<Vec<_>>() as &[Option<i64>],
        &h.iter().map(|x| x.won).collect::<Vec<_>>() as &[Option<bool>],
        &h.iter().map(|x| x.delta).collect::<Vec<_>>(),
        &h.iter().map(|x| x.rating_after).collect::<Vec<_>>(),
        &h.iter().map(|x| x.at).collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
