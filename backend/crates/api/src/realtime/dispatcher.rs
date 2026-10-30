//! O despachante: lê o outbox em ordem, publica cada evento no canal dos
//! sockets, guarda os avisos no sininho de cada destinatário, manda os Web
//! Push e apaga o que saiu, tudo numa transação (outra instância pula as
//! linhas travadas). Os avisos que a comunidade quer no Discord entram na
//! fila dele nessa transação (o envio é da tarefa do `discord`). Roda quando
//! acordado: depois de cada requisição que emitiu e a cada passada do
//! agendador.

use crate::{
    AppState, discord, notifications,
    outbox::{self, Event},
    push,
    shared::ApiError,
};

/// Eventos por passada; com mais, o despachante passa de novo.
const BATCH: i64 = 200;

/// Uma passada: publica e apaga até `BATCH` eventos. Devolve quantos saíram
/// (os ilegíveis também saem, com log).
pub async fn dispatch_once(state: &AppState) -> Result<usize, ApiError> {
    let mut queued = false;
    let mut tx = state.db.begin().await?;
    let batch = outbox::claim(&mut tx, BATCH).await?;
    for stored in &batch {
        let Some(event) = &stored.event else {
            continue;
        };
        queued |= enqueue_discord(&mut tx, event).await?;
        if let Event::Notice { users, notice } = event {
            notifications::store(&mut tx, users, notice).await?;
            if let Some(sender) = &state.push {
                push::deliver(&mut tx, sender.as_ref(), users, notice).await?;
            }
        }
        state.realtime.publish(event.clone());
    }
    let ids: Vec<i64> = batch.iter().map(|s| s.id).collect();
    outbox::remove(&mut tx, &ids).await?;
    tx.commit().await?;
    if queued {
        state.webhooks.wake();
    }
    Ok(batch.len())
}

/// Até esvaziar o outbox. Um erro fica no log e espera o próximo despertar.
pub(super) async fn drain(state: &AppState) {
    loop {
        match dispatch_once(state).await {
            Ok(n) if n as i64 == BATCH => {}
            Ok(_) => return,
            Err(e) => {
                tracing::error!(code = e.code, "despachante do outbox falhou");
                return;
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
