//! Confirmação por tempo: resultado pendente sem resposta confirma sozinho
//! ao fim da janela de contestação. Sem job em segundo plano: a API aplica
//! ao ler a chave ou uma partida e antes de cada ação sobre ela (como a
//! abertura automática do check-in). `apply_due_confirmations` é a peça
//! isolada que um agendador (tempo real) também pode chamar.

use chrono::{DateTime, Utc};
use sqlx::{Acquire, PgConnection};

use super::{db, stage};
use crate::{
    shared::ApiError,
    tenants::TenantMember,
    tournaments::{StoredTournament, find_visible, find_visible_for_update},
};

/// Confirma os resultados com a janela vencida em `now` e, se algum
/// confirmou, encerra o torneio se a chave terminou. Quem chama trava o
/// torneio (`find_for_update`). Devolve quantos confirmou.
pub async fn apply_due_confirmations(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    now: DateTime<Utc>,
) -> Result<u64, ApiError> {
    let confirmed = db::confirm_due(conn, tournament.id, now).await?;
    if confirmed > 0 {
        let stage = stage::load(conn, tournament).await?;
        stage::sync_status(conn, tournament, &stage).await?;
    }
    Ok(confirmed)
}

/// O torneio `slug` como `member` o vê, depois de aplicar as confirmações
/// vencidas: só abre transação (e trava o torneio) quando há alguma.
pub async fn find_current<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let tournament = find_visible(conn, member, slug).await?;
    if !db::has_due(conn, tournament.id, Utc::now()).await? {
        return Ok(tournament);
    }
    let mut tx = conn.begin().await?;
    let locked = find_visible_for_update(&mut tx, member, slug).await?;
    apply_due_confirmations(&mut tx, &locked, Utc::now()).await?;
    let current = find_visible(&mut tx, member, slug).await?;
    tx.commit().await?;
    Ok(current)
}

/// Para as ações: trava o torneio `slug` (como `member` o vê), aplica as
/// confirmações vencidas e devolve o torneio como ficou.
pub async fn lock_current<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let locked = find_visible_for_update(conn, member, slug).await?;
    if apply_due_confirmations(conn, &locked, Utc::now()).await? == 0 {
        return Ok(locked);
    }
    find_visible(conn, member, slug).await
}
