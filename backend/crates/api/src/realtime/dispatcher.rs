//! O despachante: lê o outbox em ordem, publica cada evento no canal dos
//! sockets, manda os Web Push dos avisos e apaga o que saiu, tudo numa
//! transação (outra instância pula as linhas travadas). Roda quando acordado:
//! depois de cada requisição que emitiu e a cada passada do agendador.

use crate::{
    AppState,
    outbox::{self, Event},
    push,
    shared::ApiError,
};

/// Eventos por passada; com mais, o despachante passa de novo.
const BATCH: i64 = 200;

/// Uma passada: publica e apaga até `BATCH` eventos. Devolve quantos saíram
/// (os ilegíveis também saem, com log).
pub async fn dispatch_once(state: &AppState) -> Result<usize, ApiError> {
    let mut tx = state.db.begin().await?;
    let batch = outbox::claim(&mut tx, BATCH).await?;
    for stored in &batch {
        let Some(event) = &stored.event else {
            continue;
        };
        if let (Event::Notice { users, notice }, Some(sender)) = (event, &state.push) {
            push::deliver(&mut tx, sender.as_ref(), users, notice).await?;
        }
        state.realtime.publish(event.clone());
    }
    let ids: Vec<i64> = batch.iter().map(|s| s.id).collect();
    outbox::remove(&mut tx, &ids).await?;
    tx.commit().await?;
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
