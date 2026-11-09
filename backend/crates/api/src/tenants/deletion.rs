//! Excluir a comunidade, digitando o slug (só o owner). Sem torneio iniciado
//! ou encerrado, ela sai na hora. Com histórico, agenda para daqui a 7 dias, e
//! o owner cancela até lá; nesse meio-tempo a comunidade é só leitura (o
//! `TenantMember` recusa toda escrita, menos o que recebe `DuringDeletion`:
//! cancelar e sair). Não agenda com torneio em andamento (a regra é do banco).
//! Na véspera, o owner recebe um aviso (`tenant.deletion_due`). Na data, o
//! agendador apaga o tenant, e a cascata leva tudo o que está dentro; os
//! membros perdem o acesso na hora (`access_lost`).

use axum::{Json, extract::State};
use chrono::{DateTime, TimeDelta, Utc};
use sqlx::{PgConnection, PgPool};

use super::{
    DuringDeletion, MyTenant, Owner, TenantId, TenantMember, changes, db,
    types::{
        RequestTenantDeletion, TenantDeletionEffect, TenantDeletionOutcome, TenantDeletionPreview,
    },
};
use crate::{
    AppState,
    outbox::{self, Event, Notice, NoticeKind},
    shared::{ApiError, ErrorBody, JsonBody},
};

/// Dias entre o pedido e a exclusão da comunidade.
const DELETION_DAYS: i32 = 7;

/// Antecedência do aviso ao owner, antes da exclusão.
const REMINDER_HOURS: i64 = 24;

#[utoipa::path(
    get,
    path = "/tenants/{slug}/deletion",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = TenantDeletionPreview, description = "O que `POST` faria agora, para a tela dizer antes de confirmar. Quem decide é o `POST`, na hora dele"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn preview_tenant_deletion(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
) -> Result<Json<TenantDeletionPreview>, ApiError> {
    let history = db::tournament_history(&state.db, owner.tenant).await?;
    let effect = match (history.in_progress, history.started) {
        (true, _) => TenantDeletionEffect::Blocked,
        (false, true) => TenantDeletionEffect::Scheduled,
        (false, false) => TenantDeletionEffect::Immediate,
    };
    Ok(Json(TenantDeletionPreview { effect }))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/deletion",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = RequestTenantDeletion,
    responses(
        (status = OK, body = TenantDeletionOutcome, description = "`deleted`: nenhum torneio começou, e a comunidade saiu agora, com tudo o que tem dentro. `scheduled`: há histórico, e ela sai em 7 dias (`deletion_scheduled_for`); até lá é só leitura: toda escrita responde 409 tenant.deletion_scheduled, menos cancelar a exclusão e sair; o check-in não abre sozinho, o aviso de hora de iniciar não sai, e os links de convite e de inscrição não valem"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled (já agendada), tenant.tournament_in_progress (o owner encerra o torneio ou desfaz o início antes)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tenant.confirmation_mismatch (o slug digitado não confere), request.invalid_body"),
    )
)]
pub async fn request_tenant_deletion(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<RequestTenantDeletion>,
) -> Result<Json<TenantDeletionOutcome>, ApiError> {
    if body.confirm_slug.trim() != owner.slug {
        return Err(ApiError::invalid("tenant.confirmation_mismatch"));
    }
    let mut tx = state.db.begin().await?;
    let Some(started) = db::lock_for_deletion(&mut tx, owner.tenant).await? else {
        return Err(ApiError::not_found("tenant.not_found"));
    };
    let outcome = if started {
        let due =
            db::schedule_deletion(&mut tx, owner.tenant, owner.user.id, DELETION_DAYS).await?;
        changes::changed(&mut tx, owner.tenant).await?;
        TenantDeletionOutcome::Scheduled {
            deletion_scheduled_for: due,
        }
    } else {
        remove(&mut tx, owner.tenant).await?;
        TenantDeletionOutcome::Deleted
    };
    tx.commit().await?;
    Ok(Json(outcome))
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

/// Avisa o owner de cada tenant cuja exclusão vence nas próximas
/// `REMINDER_HOURS` (o sininho e o Web Push; os membros não recebem nada), cada
/// um na própria transação, e devolve quantos avisou.
pub async fn remind_due(db: &PgPool, now: DateTime<Utc>) -> Result<usize, sqlx::Error> {
    let until = now + TimeDelta::hours(REMINDER_HOURS);
    let mut reminded = 0;
    for tenant in db::deletions_to_remind(db, now, until).await? {
        let remind = async {
            let mut tx = db.begin().await?;
            let Some(due) = db::mark_deletion_reminded(&mut tx, tenant, now, until).await? else {
                return Ok(false);
            };
            let notice = Notice {
                kind: NoticeKind::TenantDeletionDue {
                    tenant: due.name,
                    deletes_at: due.due,
                },
                path: format!("/{}/settings", due.slug),
            };
            outbox::emit(&mut tx, Event::notice([due.owner], notice)).await?;
            tx.commit().await?;
            Ok::<_, sqlx::Error>(true)
        };
        match remind.await {
            Ok(true) => reminded += 1,
            Ok(false) => {}
            Err(e) => {
                tracing::error!(id = tenant.get(), %e, "aviso de exclusão de comunidade falhou")
            }
        }
    }
    Ok(reminded)
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
/// busca).
async fn delete_tenant(
    db: &PgPool,
    tenant: TenantId,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    if !db::lock_due_deletion(&mut tx, tenant, now).await? {
        return Ok(false);
    }
    remove(&mut tx, tenant).await?;
    tx.commit().await?;
    Ok(true)
}

/// Apaga o tenant travado. Os avisos de acesso perdido saem antes, enquanto o
/// slug existe.
async fn remove(tx: &mut PgConnection, tenant: TenantId) -> Result<(), sqlx::Error> {
    for user in db::active_members(tx, tenant).await? {
        changes::access_lost(tx, tenant, user).await?;
    }
    db::delete(tx, tenant).await
}
