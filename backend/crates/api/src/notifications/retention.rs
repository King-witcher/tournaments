//! O prazo das notificações: o agendador do tempo real apaga, a cada passada,
//! as que passaram dele (uma busca no índice de `created_at`, quase sempre sem
//! nada a apagar).

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use super::db;

/// Quanto uma notificação fica no sininho.
const KEEP_FOR: Duration = Duration::days(30);

/// Apaga as notificações com mais de `KEEP_FOR` em `now` e devolve quantas.
pub async fn delete_expired(pool: &PgPool, now: DateTime<Utc>) -> Result<u64, sqlx::Error> {
    db::delete_before(pool, now - KEEP_FOR).await
}
