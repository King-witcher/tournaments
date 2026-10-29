//! A rotina diária dos screenshots, que o agendador do tempo real chama a cada
//! passada: uma vez por dia (a última fica em `screenshot_sweeps`), apaga do
//! banco os screenshots dos torneios encerrados há mais de 30 dias e os uploads
//! pedidos e nunca registrados, e depois esvazia a fila de exclusão
//! (`screenshot_deletions`, que o banco preenche a cada screenshot apagado, por
//! qualquer caminho) apagando os objetos do bucket, até mil por requisição.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use super::db;
use crate::{
    AppState,
    shared::{ApiError, Storage, storage::DELETE_BATCH},
};

/// De quanto em quanto tempo a rotina roda.
const EVERY: Duration = Duration::days(1);
/// Quanto os screenshots ficam depois do fim do torneio.
const KEEP_AFTER_FINISH: Duration = Duration::days(30);
/// Upload pedido e não registrado depois disso não vai mais ser (a URL vale
/// 10 minutos).
const PENDING_FOR: Duration = Duration::days(1);
/// Requisições de exclusão por passada: com mais na fila, a próxima passada
/// (10 s depois) continua, sem segurar o agendador.
const BATCHES_PER_PASS: usize = 5;

/// O que uma passada fez.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sweep {
    /// Screenshots de torneios encerrados há mais de 30 dias.
    pub expired: u64,
    /// Uploads nunca registrados.
    pub abandoned: u64,
    /// Objetos apagados do bucket.
    pub deleted: usize,
    /// A fila ficou vazia (senão, a passada seguinte continua).
    pub drained: bool,
}

/// A rotina, se a última passada tem mais de um dia (ou nunca houve). `None`
/// sem o bucket ou fora da hora.
pub async fn sweep_due(state: &AppState, now: DateTime<Utc>) -> Result<Option<Sweep>, ApiError> {
    let Some(storage) = &state.storage else {
        return Ok(None);
    };
    if db::last_sweep(&state.db)
        .await?
        .is_some_and(|last| now - last < EVERY)
    {
        return Ok(None);
    }
    sweep(&state.db, storage, now).await.map(Some)
}

/// Uma passada em `now`. Marca a passada quando a fila esvazia e também
/// quando o bucket falha (o erro fica no log e a fila espera o dia seguinte,
/// em vez de bater no bucket a cada 10 s); só não marca quando sobrou fila
/// por causa do limite de requisições.
pub async fn sweep(db: &PgPool, storage: &Storage, now: DateTime<Utc>) -> Result<Sweep, ApiError> {
    let mut done = Sweep {
        expired: db::delete_finished_before(db, now - KEEP_AFTER_FINISH).await?,
        abandoned: db::delete_pending_before(db, now - PENDING_FOR).await?,
        ..Sweep::default()
    };
    let mut finished = false;
    for _ in 0..BATCHES_PER_PASS {
        let keys = db::queued(db, DELETE_BATCH).await?;
        if keys.is_empty() {
            done.drained = true;
            finished = true;
            break;
        }
        let failed = match storage.delete(&keys).await {
            Ok(failed) => failed,
            Err(e) => {
                tracing::error!(error = %e, "screenshots: exclusão no bucket falhou");
                finished = true;
                break;
            }
        };
        let gone: Vec<String> = keys.into_iter().filter(|k| !failed.contains(k)).collect();
        db::dequeue(db, &gone).await?;
        done.deleted += gone.len();
        if !failed.is_empty() {
            tracing::warn!(
                failed = failed.len(),
                "screenshots: o bucket não apagou alguns objetos"
            );
            finished = true;
            break;
        }
    }
    if finished {
        db::mark_swept(db, now).await?;
    }
    if done != Sweep::default() {
        tracing::info!(?done, "screenshots: rotina diária");
    }
    Ok(done)
}
