//! O agendador: a cada 10 s, numa instância só (advisory lock do Postgres),
//! abre o check-in que chegou na hora, avisa a hora de iniciar, confirma os
//! mapas com a janela vencida, sorteia os passos de veto vencidos (rede de
//! segurança do relógio preciso, `clock`) e executa as exclusões de comunidade
//! e de conta que venceram, cada torneio, comunidade ou conta na própria
//! transação e emitindo pelo outbox; uma vez por dia roda a rotina dos
//! screenshots (depois das exclusões, para já levar do bucket os screenshots
//! das comunidades excluídas), recalcula o rating das comunidades marcadas
//! (depois das confirmações e das desistências desta passada) e apaga as
//! notificações com mais de 30 dias; depois acorda o despachante. Aplicar ao ler e agir continua, para quem chega
//! antes do agendador ver o estado certo.

use std::time::Duration;

use chrono::{DateTime, Utc};

use super::db;
use crate::{
    AppState, matches, members, notifications, ratings, screenshots, shared::ApiError, tenants,
    tournaments,
};

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
    /// Passos de veto sorteados pelo fim do tempo.
    pub vetoed: u64,
    /// Comunidades excluídas.
    pub deleted_tenants: usize,
    /// Contas excluídas (anonimizadas).
    pub deleted_accounts: usize,
    /// A rotina diária dos screenshots rodou nesta passada.
    pub swept: bool,
    /// Comunidades com o rating recalculado e diferente.
    pub rated: usize,
    /// Notificações que passaram do prazo e saíram.
    pub expired_notifications: u64,
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
            vetoed: matches::veto_clock(&state.db, now).await?.0,
            deleted_tenants: tenants::delete_due(&state.db, now).await?,
            deleted_accounts: members::delete_due_accounts(state, now).await?,
            swept: sweep_screenshots(state, now).await,
            rated: ratings::refresh_stale(&state.db).await?,
            expired_notifications: notifications::delete_expired(&state.db, now).await?,
        })
    }
    .await;
    lock.rollback().await?;
    state.realtime.wake();
    pass.map(Some)
}

/// A rotina diária dos screenshots, se é a hora; o erro fica no log e não para
/// o resto da passada.
async fn sweep_screenshots(state: &AppState, now: DateTime<Utc>) -> bool {
    match screenshots::sweep_due(state, now).await {
        Ok(done) => done.is_some(),
        Err(e) => {
            tracing::error!(code = e.code, "rotina dos screenshots falhou");
            false
        }
    }
}
