//! O agendador: a cada 10 s, numa instância só (advisory lock do Postgres),
//! abre o check-in que chegou na hora, avisa a hora de iniciar e confirma os
//! mapas com a janela vencida, cada torneio na própria transação e emitindo
//! pelo outbox, e recalcula o rating das comunidades marcadas (depois das
//! confirmações, para pegar as desta passada); depois acorda o despachante. Aplicar ao ler e agir continua,
//! para quem chega antes do agendador ver o estado certo.

use std::time::Duration;

use chrono::{DateTime, Utc};

use super::db;
use crate::{AppState, matches, ratings, shared::ApiError, tournaments};

pub const EVERY: Duration = Duration::from_secs(10);

/// O que uma passada fez.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tick {
    /// Check-ins abertos.
    pub opened: usize,
    /// Torneios avisados da hora de iniciar.
    pub reminded: usize,
    /// Mapas confirmados por tempo.
    pub confirmed: u64,
    /// Comunidades com o rating recalculado e diferente.
    pub rated: usize,
}

/// Uma passada em `now`. `None` se outra instância está numa passada.
pub async fn tick(state: &AppState, now: DateTime<Utc>) -> Result<Option<Tick>, ApiError> {
    let mut lock = state.db.begin().await?;
    if !db::try_lock_scheduler(&mut lock).await? {
        return Ok(None);
    }
    let pass = async {
        Ok::<_, ApiError>(Tick {
            opened: tournaments::open_due_check_ins(&state.db, now).await?,
            reminded: tournaments::remind_due_starts(&state.db, now).await?,
            confirmed: matches::confirm_all_due(&state.db, now).await?,
            rated: ratings::refresh_stale(&state.db).await?,
        })
    }
    .await;
    lock.rollback().await?;
    state.realtime.wake();
    pass.map(Some)
}
