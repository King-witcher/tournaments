//! A rotina diária do bucket público, que o agendador do tempo real chama a
//! cada passada: uma vez por dia (a última fica em `image_sweeps`), apaga os
//! uploads pedidos e nunca registrados e as imagens das regras registradas e
//! nunca usadas em texto nenhum, e depois esvazia a fila de exclusão
//! (`image_deletions`, que o banco preenche a cada objeto que deixa de ser
//! usado) apagando os objetos do bucket, até mil por requisição. Objeto cuja
//! URL de upload ainda vale fica para a passada do dia seguinte: apagado
//! antes, o mesmo PUT o gravaria de novo, fora do banco e da fila.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use super::db;
use crate::{
    AppState,
    shared::{ApiError, Storage, storage::DELETE_BATCH},
    tournaments,
};

/// De quanto em quanto tempo a rotina roda.
const EVERY: Duration = Duration::days(1);
/// Upload pedido e não registrado depois disso não vai mais ser (a URL vale
/// 10 minutos).
const PENDING_FOR: Duration = Duration::days(1);
/// Imagem das regras registrada e sem texto depois disso não vai mais ser
/// usada (quem envia põe no texto e salva em seguida).
const UNUSED_FOR: Duration = Duration::days(1);
/// Folga depois do vencimento da URL de upload antes de apagar o objeto: o
/// bucket confere a validade pelo relógio dele.
const CLOCK_MARGIN: Duration = Duration::minutes(5);
/// Requisições de exclusão por passada: com mais na fila, a próxima passada
/// (10 s depois) continua, sem segurar o agendador.
const BATCHES_PER_PASS: usize = 5;

/// O que uma passada fez.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sweep {
    /// Uploads nunca registrados.
    pub abandoned: u64,
    /// Imagens das regras registradas e nunca usadas.
    pub unused: u64,
    /// Objetos apagados do bucket.
    pub deleted: usize,
    /// A fila ficou sem nada que já desse para apagar (senão, a passada
    /// seguinte continua).
    pub drained: bool,
}

/// A rotina, se a última passada tem mais de um dia (ou nunca houve). `None`
/// sem o bucket público ou fora da hora.
pub async fn sweep_due(state: &AppState, now: DateTime<Utc>) -> Result<Option<Sweep>, ApiError> {
    let Some(storage) = &state.public_storage else {
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
/// quando o bucket falha (o erro fica no log e a fila espera o dia seguinte);
/// só não marca quando sobrou fila por causa do limite de requisições.
pub async fn sweep(db: &PgPool, storage: &Storage, now: DateTime<Utc>) -> Result<Sweep, ApiError> {
    let mut done = Sweep {
        abandoned: db::delete_pending_before(db, now - PENDING_FOR).await?,
        unused: tournaments::delete_unused_rules_images(db, now - UNUSED_FOR).await?,
        ..Sweep::default()
    };
    let mut finished = false;
    for _ in 0..BATCHES_PER_PASS {
        let keys = db::queued(db, DELETE_BATCH, now - CLOCK_MARGIN).await?;
        if keys.is_empty() {
            done.drained = true;
            finished = true;
            break;
        }
        let failed = match storage.delete(&keys).await {
            Ok(failed) => failed,
            Err(e) => {
                tracing::error!(error = %e, "imagens: exclusão no bucket falhou");
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
                "imagens: o bucket não apagou alguns objetos"
            );
            finished = true;
            break;
        }
    }
    if finished {
        db::mark_swept(db, now).await?;
    }
    if done != Sweep::default() {
        tracing::info!(?done, "imagens: rotina diária");
    }
    Ok(done)
}
