//! O agendador: a cada 10 s, numa instância só (advisory lock do Postgres),
//! confirma os mapas com a janela vencida, avisa a organização das contestações
//! que ninguém resolveu a tempo, avisa na véspera as exclusões de
//! comunidade (ao owner) e de conta (por email) e executa as que venceram,
//! cada torneio, comunidade ou conta na própria
//! transação e emitindo pelo outbox; uma vez por dia roda as rotinas dos
//! screenshots e das imagens públicas (depois das exclusões, para já levar dos
//! buckets os arquivos das comunidades excluídas), recalcula o rating das comunidades marcadas
//! (depois das confirmações e das desistências desta passada) e apaga as
//! notificações com mais de 30 dias. Se a passada emitiu, acorda o despachante
//! e, para os torneios alcançados, o relógio do veto (`clock`, que sorteia os
//! passos vencidos e tem a própria rede de segurança). Aplicar ao ler e agir continua, para quem chega
//! antes do agendador ver o estado certo. Também sorteia os passos vencidos das
//! salas de veto e apaga as vencidas.

use std::time::Duration;

use chrono::{DateTime, Utc};

use super::db;
use crate::{
    AppState, auth, images, matches, members, notifications, outbox, ratings, screenshots,
    shared::ApiError, tenants, veto_rooms,
};

pub const EVERY: Duration = Duration::from_secs(10);

/// O que uma passada fez.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tick {
    /// Mapas confirmados por tempo.
    pub confirmed: u64,
    /// Contestações que a organização foi avisada de resolver.
    pub contest_notices: u64,
    /// Owners avisados da exclusão da comunidade na véspera.
    pub tenant_reminders: usize,
    /// Lembretes por email da exclusão da conta que saíram.
    pub account_reminders: usize,
    /// Comunidades excluídas.
    pub deleted_tenants: usize,
    /// Contas excluídas (anonimizadas).
    pub deleted_accounts: usize,
    /// A rotina diária dos screenshots rodou nesta passada.
    pub swept: bool,
    /// A rotina diária das imagens públicas rodou nesta passada.
    pub swept_images: bool,
    /// Comunidades com o rating recalculado e diferente.
    pub rated: usize,
    /// Notificações que passaram do prazo e saíram.
    pub expired_notifications: u64,
    /// Passos de sala de veto sorteados pelo fim do tempo.
    pub veto_room_steps: u64,
    /// Salas de veto vencidas que saíram.
    pub expired_veto_rooms: u64,
}

/// Uma passada em `now`. `None` se outra instância está numa passada.
pub async fn tick(state: &AppState, now: DateTime<Utc>) -> Result<Option<Tick>, ApiError> {
    // O lock é de transação, para soltar sozinho até se a tarefa morrer; por
    // isso esta conexão fica presa à passada, e as etapas usam outras do pool.
    let mut lock = state.db.begin().await?;
    if !db::try_lock_scheduler(&mut lock).await? {
        return Ok(None);
    }
    let pass = async {
        Ok::<_, ApiError>(Tick {
            confirmed: matches::confirm_all_due(&state.db, now).await?,
            contest_notices: matches::notify_contested(&state.db, now).await?,
            tenant_reminders: tenants::remind_due_deletions(&state.db, now).await?,
            account_reminders: auth::remind_due_account_deletions(state, now).await?,
            deleted_tenants: tenants::delete_due(&state.db, now).await?,
            deleted_accounts: members::delete_due_accounts(state, now).await?,
            swept: sweep_screenshots(state, now).await,
            swept_images: sweep_images(state, now).await,
            rated: ratings::refresh_stale(&state.db).await?,
            expired_notifications: notifications::delete_expired(&state.db, now).await?,
            veto_room_steps: veto_rooms::clock(state, now).await?.0,
            expired_veto_rooms: veto_rooms::delete_expired(state, now).await?,
        })
    };
    let (pass, touched) = outbox::track(None, pass).await;
    state.realtime.emitted(touched);
    lock.rollback().await?;
    pass.map(Some)
}

/// A rotina diária das imagens públicas, se é a hora; o erro fica no log e não
/// para o resto da passada.
async fn sweep_images(state: &AppState, now: DateTime<Utc>) -> bool {
    match images::sweep_due(state, now).await {
        Ok(done) => done.is_some(),
        Err(e) => {
            tracing::error!(code = e.code, "rotina das imagens públicas falhou");
            false
        }
    }
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
