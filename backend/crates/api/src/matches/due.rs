//! Confirmação por tempo: mapa pendente sem resposta confirma sozinho
//! ao fim da janela de contestação. A API aplica ao ler a chave ou uma
//! partida e antes de cada ação sobre ela (como a abertura automática do
//! check-in), e o agendador do tempo real aplica nos torneios com mapa
//! vencido que ninguém leu (`confirm_all_due`), para a chave andar e os
//! jogadores serem avisados. Os passos de veto vencidos entram junto
//! (`current`, `lock_stage`).

use chrono::{DateTime, Utc};
use sqlx::{Acquire, PgConnection, PgPool};

use super::{
    changes::Snapshot,
    db, settle,
    stage::{self, Stage},
    veto,
};
use crate::{
    shared::ApiError,
    tenants::TenantMember,
    tournaments::{self, StoredTournament, find_visible, find_visible_for_update},
};

/// Confirma os mapas com a janela vencida em `now` e, se algum confirmou,
/// fecha as séries que eles decidiram, dá os W.O. automáticos que ficaram
/// prontos, encerra o torneio se a chave terminou e emite o que mudou
/// (`settle`). Quem chama trava o torneio (`find_for_update`). Devolve
/// quantos confirmou.
pub async fn apply_due_confirmations(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    now: DateTime<Utc>,
) -> Result<u64, ApiError> {
    if !db::has_due(conn, tournament.id, now).await? {
        return Ok(0);
    }
    let before = Snapshot::of(&stage::load(conn, tournament).await?);
    let confirmed = db::confirm_due(conn, tournament.id, now).await?;
    if confirmed > 0 {
        settle::settle(conn, tournament, None, &before).await?;
    }
    Ok(confirmed)
}

/// Para o agendador: aplica `apply_due_confirmations` em cada torneio com
/// mapa vencido em `now`, cada um na própria transação e com o torneio
/// travado. Devolve quantos mapas confirmou; o erro de um torneio fica no
/// log e não para os outros.
pub async fn confirm_all_due(db: &PgPool, now: DateTime<Utc>) -> Result<u64, ApiError> {
    let mut confirmed = 0;
    for (tenant, slug) in db::due_tournaments(db, now).await? {
        let confirm = async {
            let mut tx = db.begin().await?;
            let Some(t) = tournaments::find_for_update(&mut tx, tenant, &slug, None).await? else {
                return Ok(0);
            };
            let n = apply_due_confirmations(&mut tx, &t, now).await?;
            tx.commit().await?;
            Ok::<_, ApiError>(n)
        };
        match confirm.await {
            Ok(n) => confirmed += n,
            Err(e) => tracing::error!(slug, code = e.code, "confirmação por tempo falhou"),
        }
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

/// O torneio `slug` e a fase em jogo como `member` os vê, depois das
/// confirmações e dos passos de veto vencidos (só abre transação, e trava o
/// torneio, quando há algum).
pub async fn current<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<(StoredTournament, Stage), ApiError> {
    let tournament = find_current(conn, member, slug).await?;
    let stage = stage::load(conn, &tournament).await?;
    if !veto::any_due(tournament.status, &stage, Utc::now()) {
        return Ok((tournament, stage));
    }
    let mut tx = conn.begin().await?;
    let (tournament, stage) = lock_stage(&mut tx, member, slug).await?;
    tx.commit().await?;
    Ok((tournament, stage))
}

/// Para as ações: trava o torneio `slug` (como `member` o vê), aplica as
/// confirmações e os passos de veto vencidos e devolve o torneio e a fase
/// como ficaram.
pub async fn lock_stage<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<(StoredTournament, Stage), ApiError> {
    let tournament = lock_current(conn, member, slug).await?;
    let stage = stage::load(conn, &tournament).await?;
    let now = Utc::now();
    if !veto::any_due(tournament.status, &stage, now) {
        return Ok((tournament, stage));
    }
    let before = Snapshot::of(&stage);
    veto::apply_due(conn, &tournament, &stage, now).await?;
    let (stage, status) = settle::settle(conn, &tournament, None, &before).await?;
    let tournament = StoredTournament {
        status,
        ..tournament
    };
    Ok((tournament, stage))
}
