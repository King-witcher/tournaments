//! O recálculo do rating de um tenant: o histórico inteiro de novo pelo
//! engine, sempre que há marca de "desatualizado" (o trigger da fonte grava
//! a marca na transação de quem mudou um mapa confirmado, um ajuste, a
//! configuração ou o "conta para o rating"). Quem recalcula: o agendador, a
//! cada passada (`refresh_stale`), a leitura do ranking e do rating de um
//! membro antes de ler (`fresh`), e as rotas de ajuste e configuração, na
//! própria transação (`refresh`).

use std::collections::HashMap;

use engine::rating::{self as er, Elo};
use sqlx::{PgConnection, PgPool};

use super::{
    db,
    history::{self, Cache},
};
use crate::{shared::ApiError, tenants, tenants::TenantId};

/// Recalcula o tenant se ele tem marca, na transação de `conn`; `true` se o
/// cache mudou (e então a comunidade emite a mudança no outbox).
///
/// O lock por tenant vem antes de tudo: com ele, o histórico lido inclui
/// tudo o que gravou as marcas que esta transação apaga (as de transações que
/// ainda não confirmaram ficam e fazem recalcular de novo depois).
pub async fn refresh(conn: &mut PgConnection, tenant: TenantId) -> Result<bool, ApiError> {
    db::lock(conn, tenant).await?;
    if db::consume_dirty(conn, tenant).await? == 0 {
        return Ok(false);
    }
    let elo = elo(conn, tenant).await?;
    let mut sources = db::sources(conn, tenant).await?;
    let events = history::events(&mut sources);
    let replay = er::replay(&elo, &events).map_err(|e| {
        tracing::error!(tenant = tenant.get(), error = %e, "histórico de rating inválido");
        ApiError::internal()
    })?;
    let cache = Cache::build(&sources, &replay);
    if db::read(conn, tenant).await? == cache {
        return Ok(false);
    }
    db::write(conn, tenant, &cache).await?;
    tenants::changed(conn, tenant).await?;
    Ok(true)
}

/// O Elo do tenant (o padrão sem configuração gravada).
pub async fn elo(conn: &mut PgConnection, tenant: TenantId) -> Result<Elo, ApiError> {
    let config = db::settings(&mut *conn, tenant).await?.unwrap_or_default();
    // A tabela tem as mesmas faixas do engine.
    Elo::new(config).map_err(|e| {
        tracing::error!(tenant = tenant.get(), error = %e, "configuração de rating inválida");
        ApiError::internal()
    })
}

/// Antes de ler o cache: recalcula numa transação própria se o tenant tem
/// marca (sem marca, só uma consulta).
pub async fn fresh(db: &PgPool, tenant: TenantId) -> Result<(), ApiError> {
    if !db::is_dirty(db, tenant).await? {
        return Ok(());
    }
    let mut tx = db.begin().await?;
    refresh(&mut tx, tenant).await?;
    tx.commit().await?;
    Ok(())
}

/// O que o agendador faz: recalcula cada tenant com marca, cada um na
/// própria transação (o erro de um fica no log e não para os outros).
/// Quantos mudaram.
pub async fn refresh_stale(db: &PgPool) -> Result<usize, ApiError> {
    let mut changed = 0;
    for tenant in db::dirty_tenants(db).await? {
        let run = async {
            let mut tx = db.begin().await?;
            let did = refresh(&mut tx, tenant).await?;
            tx.commit().await?;
            Ok::<_, ApiError>(did)
        };
        match run.await {
            Ok(did) => changed += usize::from(did),
            Err(e) => tracing::error!(
                tenant = tenant.get(),
                code = e.code,
                "recálculo do rating falhou"
            ),
        }
    }
    Ok(changed)
}

/// O rating de cada um de `users` na comunidade agora (o inicial do Elo para
/// quem ainda não tem), recalculando antes, na transação de `conn`, se há
/// marca. Para o sorteio balanceado dos times.
pub async fn current(
    conn: &mut PgConnection,
    tenant: TenantId,
    users: &[i64],
) -> Result<HashMap<i64, i32>, ApiError> {
    refresh(conn, tenant).await?;
    let initial = elo(conn, tenant).await?.config().initial;
    let rated = db::ratings_of(conn, tenant, users).await?;
    Ok(users
        .iter()
        .map(|u| (*u, rated.get(u).copied().unwrap_or(initial)))
        .collect())
}
