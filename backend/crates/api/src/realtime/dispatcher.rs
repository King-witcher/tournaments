//! O despachante: lê o outbox em ordem, guarda os avisos no sininho de cada
//! destinatário, manda os Web Push e apaga o que saiu, tudo numa transação
//! (outra instância pula as linhas travadas), e depois do commit publica cada
//! evento no canal dos sockets (o `notify` leva o id de uma notificação que
//! já existe). Os avisos que a comunidade quer no Discord entram na fila dele
//! nessa transação (o envio é da tarefa do `discord`). Roda quando acordado:
//! depois de cada requisição, passada do agendador ou do relógio que emitiu;
//! sem despertar, a cada `IDLE` (rede de segurança de quem emitiu sem acordar),
//! e `RETRY` depois de uma falha.

use std::time::Duration;

use super::hub::Published;
use crate::{
    AppState, discord, notifications,
    outbox::{self, Event},
    push,
    shared::ApiError,
};

/// Eventos por passada; com mais, o despachante passa de novo.
const BATCH: i64 = 200;
/// Sem despertar, quanto o despachante espera para olhar o outbox de novo.
pub(super) const IDLE: Duration = Duration::from_secs(60);
/// Depois de uma passada que falhou, quando tentar de novo.
pub(super) const RETRY: Duration = Duration::from_secs(10);

/// Uma passada: publica e apaga até `BATCH` eventos. Devolve quantos saíram
/// (os ilegíveis também saem, com log).
pub async fn dispatch_once(state: &AppState) -> Result<usize, ApiError> {
    let mut queued = false;
    let mut tx = state.db.begin().await?;
    let batch = outbox::claim(&mut tx, BATCH).await?;
    let mut published = Vec::with_capacity(batch.len());
    for stored in &batch {
        let Some(event) = &stored.event else {
            continue;
        };
        queued |= enqueue_discord(&mut tx, event).await?;
        let mut out = Published {
            origin: stored.origin.clone(),
            ..Published::from(event.clone())
        };
        if let Event::Notice { users, notice } = event {
            out.notifications = notifications::store(&mut tx, users, notice).await?;
            if let Some(sender) = &state.push {
                push::deliver(&mut tx, sender.as_ref(), users, notice).await?;
            }
        }
        published.push(out);
    }
    if batch.is_empty() {
        // Nada a apagar: a transação só travaria o que não existe.
        tx.rollback().await?;
        return Ok(0);
    }
    let ids: Vec<i64> = batch.iter().map(|s| s.id).collect();
    outbox::remove(&mut tx, &ids).await?;
    tx.commit().await?;
    for out in published {
        state.realtime.publish(out);
    }
    if queued {
        state.webhooks.wake();
    }
    Ok(batch.len())
}

/// Até esvaziar o outbox. Um erro fica no log e devolve `false` (tenta de
/// novo em `RETRY`).
pub(super) async fn drain(state: &AppState) -> bool {
    loop {
        match dispatch_once(state).await {
            Ok(n) if n as i64 == BATCH => {}
            Ok(_) => return true,
            Err(e) => {
                tracing::error!(code = e.code, "despachante do outbox falhou");
                return false;
            }
        }
    }
}

/// Põe o evento na fila do Discord, num savepoint: a falha dele fica no log e
/// não derruba a transação (e com ela o push e o tempo real do lote inteiro).
async fn enqueue_discord(tx: &mut sqlx::PgConnection, event: &Event) -> Result<bool, sqlx::Error> {
    let mut savepoint = sqlx::Connection::begin(tx).await?;
    match discord::enqueue(&mut savepoint, event).await {
        Ok(queued) => {
            savepoint.commit().await?;
            Ok(queued)
        }
        Err(e) => {
            tracing::error!(error = %e, "aviso do Discord não entrou na fila");
            savepoint.rollback().await?;
            Ok(false)
        }
    }
}
