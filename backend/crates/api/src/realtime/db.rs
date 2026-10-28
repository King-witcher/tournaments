//! SQL do agendador: o advisory lock que garante uma passada por vez entre
//! as instâncias da API.

use sqlx::PgConnection;

/// Chave do advisory lock do agendador (qualquer número fixo do projeto).
const SCHEDULER_LOCK: i64 = 0x746f_7572_6e65_7973;

/// Tenta o lock até o fim da transação de `conn`. `false`: outra instância
/// está numa passada.
pub async fn try_lock_scheduler(conn: &mut PgConnection) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"select pg_try_advisory_xact_lock($1) as "locked!""#,
        SCHEDULER_LOCK
    )
    .fetch_one(conn)
    .await
}
