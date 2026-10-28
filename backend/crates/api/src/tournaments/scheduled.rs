//! O que o agendador do tempo real faz nos torneios sem ninguém pedir, cada
//! torneio na própria transação: abrir o check-in cuja janela abriu (pelo
//! mesmo caminho da abertura ao ler, que continua valendo para quem chega
//! antes) e avisar organizers e owner quando o início marcado chega sem o
//! torneio iniciar, uma vez (adiar o início para depois do aviso arma de
//! novo).

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{changes, db, status::TournamentStatus};
use crate::shared::ApiError;

/// Abre o check-in dos torneios cuja janela abriu em `now`. Devolve quantos
/// abriu; o erro de um torneio fica no log e não para os outros.
pub async fn open_due_check_ins(db: &PgPool, now: DateTime<Utc>) -> Result<usize, ApiError> {
    let mut opened = 0;
    for due in db::due_check_ins(db, now).await? {
        let open = async {
            let mut tx = db.begin().await?;
            let status = db::current_status(
                &mut tx,
                due.id,
                due.status,
                &due.schedule,
                due.reopened_at,
                now,
            )
            .await?;
            tx.commit().await?;
            Ok::<_, sqlx::Error>(status == TournamentStatus::CheckIn)
        };
        match open.await {
            Ok(true) => opened += 1,
            Ok(false) => {}
            Err(e) => tracing::error!(id = due.id.get(), %e, "abertura do check-in falhou"),
        }
    }
    Ok(opened)
}

/// Avisa a hora de iniciar dos torneios cujo início chegou em `now`. Devolve
/// quantos avisou.
pub async fn remind_due_starts(db: &PgPool, now: DateTime<Utc>) -> Result<usize, ApiError> {
    let mut reminded = 0;
    for id in db::starts_to_remind(db, now).await? {
        let remind = async {
            let mut tx = db.begin().await?;
            let Some(tenant) = db::mark_start_reminded(&mut tx, id, now).await? else {
                return Ok(false);
            };
            changes::start_due(&mut tx, tenant, id).await?;
            tx.commit().await?;
            Ok::<_, sqlx::Error>(true)
        };
        match remind.await {
            Ok(true) => reminded += 1,
            Ok(false) => {}
            Err(e) => tracing::error!(id = id.get(), %e, "aviso de hora de iniciar falhou"),
        }
    }
    Ok(reminded)
}
