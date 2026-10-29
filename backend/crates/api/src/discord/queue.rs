//! A fila de envio. O despachante do outbox só põe o aviso na fila
//! (`enqueue`, na transação dele, sem esperar o Discord); uma tarefa própria
//! (`spawn`) a esvazia em ordem (`deliver_due`), uma mensagem por vez, no
//! ritmo que o Discord manda: o webhook que pediu para esperar (429 ou a
//! janela de ~5 mensagens por 2 s esgotada) fica em espera, e os das outras
//! comunidades seguem. Pelo menos uma vez, como o Web Push: a mensagem só sai
//! da fila depois do 2xx, e a reserva (`LEASE`) cobre quem cair no meio.

use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use sqlx::PgConnection;

use super::{
    catalog::{self, To},
    db::{self, Job, Queued},
    render,
    sender::Outcome,
};
use crate::{AppState, outbox::Event, shared::ApiError};

/// Tentativas de uma mensagem antes de desistir (falha de rede, 5xx).
const MAX_ATTEMPTS: i16 = 5;
/// Primeira espera depois de uma falha; dobra a cada tentativa.
const BACKOFF: Duration = Duration::from_secs(30);
/// Quanto a mensagem fica reservada para quem a está enviando.
const LEASE: Duration = Duration::from_secs(2 * 60);
/// Depois disso o aviso já não serve.
const MAX_AGE: Duration = Duration::from_secs(60 * 60);
/// De quanto em quanto tempo a tarefa olha a fila sem ser acordada (esperas
/// e novas tentativas vencem sozinhas).
const IDLE: Duration = Duration::from_secs(15);
/// Mensagens por chamada de `deliver_due`: uma enchente não prende a tarefa.
const BATCH: usize = 200;

/// Põe na fila o que o evento do outbox anuncia, se a comunidade tem webhook
/// com o aviso ligado. Roda na transação do despachante. Devolve se alguma
/// mensagem entrou (para acordar a tarefa depois do commit).
pub async fn enqueue(conn: &mut PgConnection, event: &Event) -> Result<bool, sqlx::Error> {
    let (value, users) = match event {
        Event::Notice { users, notice } => (serde_json::to_value(notice), users.as_slice()),
        Event::Announce { announce } => (serde_json::to_value(announce), [].as_slice()),
        _ => return Ok(false),
    };
    let Ok(serde_json::Value::Object(mut notice)) = value else {
        return Ok(false);
    };
    let (Some(code), Some(path)) = (
        notice.remove("code").and_then(into_string),
        notice.remove("path").and_then(into_string),
    ) else {
        return Ok(false);
    };
    let params = notice
        .remove("params")
        .unwrap_or_else(|| serde_json::json!({}));
    let (Some(spec), Some(tenant)) = (catalog::find(&code), path.split('/').nth(1)) else {
        return Ok(false);
    };
    let users: &[i64] = if spec.to == To::Nobody { &[] } else { users };
    let merge_key = spec.merge.then(|| format!("{code}:{path}"));
    let message = Queued {
        code: &code,
        params: &params,
        path: &path,
    };
    db::enqueue(conn, tenant, &message, users, merge_key.as_deref()).await
}

fn into_string(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s),
        _ => None,
    }
}

/// Envia o que pode sair em `now`, em ordem, até a fila ficar sem nada
/// disponível (ou `BATCH` mensagens). Devolve quantas foram tratadas.
pub async fn deliver_due(state: &AppState, now: DateTime<Utc>) -> Result<usize, ApiError> {
    db::drop_stale(&state.db, now, MAX_AGE).await?;
    let mut handled = 0;
    while handled < BATCH {
        let Some(job) = db::claim(&state.db, now, LEASE).await? else {
            break;
        };
        send(state, &job, now).await?;
        handled += 1;
    }
    Ok(handled)
}

async fn send(state: &AppState, job: &Job, now: DateTime<Utc>) -> Result<(), ApiError> {
    let body = match render::message(state, job).await {
        Ok(Some(body)) => body,
        // O código saiu do catálogo depois de entrar na fila.
        Ok(None) => return Ok(db::done(&state.db, job.id).await?),
        Err(e) => {
            tracing::error!(error = %e, id = job.id, "mensagem do Discord não montou");
            return Ok(
                db::retry(&state.db, job.id, after(now, backoff(job.attempts)), true).await?,
            );
        }
    };
    let outcome = state
        .webhooks
        .post(&job.webhook_id, &job.token, &body)
        .await;
    match outcome {
        Outcome::Sent { wait } => {
            db::done(&state.db, job.id).await?;
            if let Some(wait) = wait {
                db::hold(&state.db, job.tenant, after(now, wait)).await?;
            }
        }
        Outcome::RateLimited { retry_after } => {
            let at = after(now, retry_after);
            db::hold(&state.db, job.tenant, at).await?;
            db::retry(&state.db, job.id, at, false).await?;
        }
        Outcome::Gone => {
            tracing::warn!(
                webhook = job.webhook_id,
                "o Discord não conhece mais o webhook: configuração marcada como quebrada"
            );
            db::mark_broken(&state.db, job.tenant, now).await?;
        }
        Outcome::Rejected(status) => {
            tracing::warn!(%status, code = job.code, "o Discord recusou a mensagem");
            db::done(&state.db, job.id).await?;
        }
        Outcome::Failed(error) if job.attempts >= MAX_ATTEMPTS => {
            tracing::warn!(%error, code = job.code, "mensagem do Discord descartada");
            db::done(&state.db, job.id).await?;
        }
        Outcome::Failed(error) => {
            tracing::warn!(%error, code = job.code, attempt = job.attempts, "envio ao Discord falhou");
            db::retry(&state.db, job.id, after(now, backoff(job.attempts)), true).await?;
        }
    }
    Ok(())
}

/// 30 s, 1 min, 2 min, 4 min...
fn backoff(attempts: i16) -> Duration {
    BACKOFF
        * 2u32.pow(
            u32::try_from(attempts.saturating_sub(1))
                .unwrap_or(0)
                .min(8),
        )
}

fn after(now: DateTime<Utc>, wait: Duration) -> DateTime<Utc> {
    now + TimeDelta::from_std(wait).unwrap_or(TimeDelta::MAX)
}

/// Sobe a tarefa que esvazia a fila: acorda com `Webhooks::wake` (o
/// despachante, depois de enfileirar) e, parada, a cada `IDLE`.
pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = deliver_due(&state, Utc::now()).await {
                tracing::error!(code = e.code, "fila do Discord falhou");
            }
            state.webhooks.woken(IDLE).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_doubles() {
        let secs: Vec<u64> = (1..=5).map(|n| backoff(n).as_secs()).collect();
        assert_eq!(secs, [30, 60, 120, 240, 480]);
    }
}
