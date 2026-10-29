//! Excluir a comunidade. O owner agenda para daqui a 7 dias, digitando o
//! slug, e cancela até lá; nesse meio-tempo a comunidade é só leitura (o
//! `TenantMember` recusa toda escrita, menos o que recebe `DuringDeletion`:
//! cancelar e sair). Não agenda com torneio em andamento (a regra é do banco).
//! Na data, o agendador apaga o tenant, e a cascata leva tudo o que está
//! dentro; os membros perdem o acesso na hora (`access_lost`).

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{
    DuringDeletion, MyTenant, Owner, TenantId, TenantMember, changes, db,
    types::ScheduleTenantDeletion,
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody},
};

/// Dias entre o pedido e a exclusão da comunidade.
const DELETION_DAYS: i32 = 7;

#[utoipa::path(
    post,
    path = "/tenants/{slug}/deletion",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = ScheduleTenantDeletion,
    responses(
        (status = OK, body = MyTenant, description = "Exclusão agendada para daqui a 7 dias (`deletion_scheduled_for`). Até lá, a comunidade é só leitura: toda escrita responde 409 tenant.deletion_scheduled, menos cancelar a exclusão e sair; o check-in não abre sozinho, o aviso de hora de iniciar não sai, e os convites e links de inscrição não valem. Na data, a comunidade é apagada com tudo o que tem dentro"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled (já agendada), tenant.tournament_in_progress (o owner encerra o torneio ou desfaz o início antes)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tenant.confirmation_mismatch (o slug digitado não confere), request.invalid_body"),
    )
)]
pub async fn schedule_tenant_deletion(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<ScheduleTenantDeletion>,
) -> Result<Json<MyTenant>, ApiError> {
    if body.confirm_slug.trim() != owner.slug {
        return Err(ApiError::invalid("tenant.confirmation_mismatch"));
    }
    let mut tx = state.db.begin().await?;
    let due = db::schedule_deletion(&mut tx, owner.tenant, owner.user.id, DELETION_DAYS).await?;
    changes::changed(&mut tx, owner.tenant).await?;
    tx.commit().await?;
    Ok(Json(MyTenant {
        deletion_scheduled_for: Some(due),
        ..owner.into()
    }))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/deletion",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = MyTenant, description = "Exclusão cancelada: a comunidade volta a aceitar escritas"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_not_scheduled"),
    )
)]
pub async fn cancel_tenant_deletion(
    State(state): State<AppState>,
    DuringDeletion(owner): DuringDeletion<Owner>,
) -> Result<Json<MyTenant>, ApiError> {
    let mut tx = state.db.begin().await?;
    if !db::cancel_deletion(&mut tx, owner.tenant).await? {
        return Err(ApiError::conflict("tenant.deletion_not_scheduled"));
    }
    changes::changed(&mut tx, owner.tenant).await?;
    tx.commit().await?;
    Ok(Json(MyTenant {
        deletion_scheduled_for: None,
        ..owner.into()
    }))
}

/// Apaga os tenants cuja exclusão venceu em `now`, cada um na própria
/// transação, e devolve quantos apagou; o erro de um fica no log e não para
/// os outros.
pub async fn delete_due(db: &PgPool, now: DateTime<Utc>) -> Result<usize, ApiError> {
    let mut deleted = 0;
    for tenant in db::due_deletions(db, now).await? {
        match delete_tenant(db, tenant, now).await {
            Ok(true) => deleted += 1,
            Ok(false) => {}
            Err(e) => tracing::error!(id = tenant.get(), %e, "exclusão de comunidade falhou"),
        }
    }
    Ok(deleted)
}

/// Apaga o tenant, se a exclusão ainda vence (o owner não cancelou desde a
/// busca). Os avisos de acesso perdido saem antes, enquanto o slug existe.
async fn delete_tenant(
    db: &PgPool,
    tenant: TenantId,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    if !db::lock_due_deletion(&mut tx, tenant, now).await? {
        return Ok(false);
    }
    for user in db::active_members(&mut tx, tenant).await? {
        changes::access_lost(&mut tx, tenant, user).await?;
    }
    db::delete(&mut tx, tenant).await?;
    tx.commit().await?;
    Ok(true)
}
