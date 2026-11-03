//! O tempo da sala de veto, sem depender de ninguém ler: os passos vencidos são
//! sorteados pelo relógio do `realtime`, que dorme até o próximo prazo, e pelo
//! agendador, que passa a cada 10 s como rede de segurança e apaga as vencidas.
//! O prazo vem da coluna `due_at`, para não refazer todas as salas a cada passada.

use chrono::{DateTime, Utc};

use super::{db, room::Room};
use crate::{AppState, shared::ApiError};

/// Salas por passada; o resto fica para a seguinte.
const BATCH: i64 = 200;

/// Aplica os passos vencidos em `now` e devolve quantos gravou e o próximo prazo.
/// Sem nada vencido, é uma query só. O erro de uma sala fica no log e não para as
/// outras.
pub async fn clock(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<(u64, Option<DateTime<Utc>>), ApiError> {
    let next = db::next_due(&state.db, now).await?;
    if next.is_none_or(|at| at > now) {
        return Ok((0, next));
    }
    let mut written = 0;
    for token in db::due_tokens(&state.db, now, BATCH).await? {
        match settle(state, &token, now).await {
            Ok(n) => written += n as u64,
            Err(e) => tracing::error!(code = e.code, "tempo da sala de veto falhou"),
        }
    }
    Ok((written, db::next_due(&state.db, now).await?))
}

/// Uma sala: trava, sorteia e avisa.
async fn settle(state: &AppState, token: &str, now: DateTime<Utc>) -> Result<usize, ApiError> {
    let mut tx = state.db.begin().await?;
    let mut room = match Room::lock(&mut tx, token).await {
        Ok(room) => room,
        // Venceu entre uma query e a outra: não há o que fazer.
        Err(e) if e.code == super::room::NOT_FOUND.code => return Ok(0),
        Err(e) => return Err(e),
    };
    let written = room.apply_due(&mut tx, now).await?;
    tx.commit().await?;
    if room.changed {
        state.veto_rooms.changed(room.id, None);
    }
    Ok(written)
}

/// Apaga as salas vencidas em `now`. Devolve quantas.
pub async fn delete_expired(state: &AppState, now: DateTime<Utc>) -> Result<u64, ApiError> {
    Ok(db::delete_expired(&state.db, now).await?)
}
