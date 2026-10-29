//! A exclusão de conta que venceu, pelo agendador. A conta sai de cada
//! comunidade como quem sai sozinho (`departure::depart`: desiste dos
//! torneios em andamento, com W.O. nas partidas dela, sai das inscrições dos
//! que não começaram e tem os convites revogados), os apelidos dela viram o
//! marcador de conta excluída (o vazio), as inscrições de push saem e `auth`
//! anonimiza o resto. Cada conta na própria transação; o erro de uma fica no
//! log e não para as outras.

use chrono::{DateTime, Utc};

use super::{db, departure};
use crate::{AppState, auth, push, shared::ApiError, tenants};

/// Executa as exclusões de conta vencidas em `now` e devolve quantas fez.
pub async fn delete_due_accounts(state: &AppState, now: DateTime<Utc>) -> Result<usize, ApiError> {
    let mut deleted = 0;
    for user_id in auth::due_account_deletions(&state.db, now).await? {
        match delete_account(state, user_id, now).await {
            Ok(true) => deleted += 1,
            Ok(false) => {}
            Err(e) => tracing::error!(user_id, code = e.code, "exclusão de conta falhou"),
        }
    }
    Ok(deleted)
}

/// Exclui a conta, se a exclusão ainda vence (ninguém entrou desde a busca).
async fn delete_account(
    state: &AppState,
    user_id: i64,
    now: DateTime<Utc>,
) -> Result<bool, ApiError> {
    let mut tx = state.db.begin().await?;
    if !auth::lock_due_account_deletion(&mut tx, user_id, now).await? {
        return Ok(false);
    }
    for tenant in db::active_tenants(&mut tx, user_id).await? {
        departure::depart(&mut tx, tenant, user_id, None, departure::check_leave).await?;
    }
    // Também onde já tinha saído: os torneios de lá mostram o nome.
    for tenant in db::forget_nicknames(&mut tx, user_id).await? {
        tenants::changed(&mut tx, tenant).await?;
    }
    push::forget_user(&mut tx, user_id).await?;
    auth::anonymize_account(&mut tx, user_id).await?;
    tx.commit().await?;
    // Ninguém entrou desde o pedido (entrar cancela), mas por garantia.
    auth::end_all_sessions(state, user_id).await?;
    Ok(true)
}
