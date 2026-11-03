use axum::{Json, extract::State, http::StatusCode};

use super::{
    MyTenant, Owner, Role, TenantMember, changes, db,
    nickname::default_nickname,
    slug::validate_slug,
    types::{CreateTenant, TransferOwnership, UpdateTenant},
};
use crate::{
    AppState,
    auth::CurrentUser,
    outbox::{self, Event},
    shared::{
        ApiError, ErrorBody, JsonBody,
        text::{clean_long_text, clean_text},
    },
};

/// Quantos tenants um usuário pode ter como owner (ser membro de outros não conta).
const MAX_OWNED_TENANTS: i64 = 3;
const NAME_MAX: usize = 64;
/// O mesmo limite da descrição do torneio e do check da tabela.
const DESCRIPTION_MAX: usize = 2000;

fn clean_name(raw: &str) -> Result<String, ApiError> {
    clean_text(raw, NAME_MAX, "tenant.name_invalid")
}

#[utoipa::path(
    post,
    path = "/tenants",
    tag = "tenants",
    request_body = CreateTenant,
    responses(
        (status = CREATED, body = MyTenant, description = "Tenant criado; quem criou é o owner"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.limit_reached, tenant.slug_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tenant.slug_invalid, tenant.slug_reserved, tenant.name_invalid, request.invalid_body"),
    )
)]
pub async fn create_tenant(
    State(state): State<AppState>,
    user: CurrentUser,
    JsonBody(body): JsonBody<CreateTenant>,
) -> Result<(StatusCode, Json<MyTenant>), ApiError> {
    validate_slug(&body.slug)?;
    let name = clean_name(&body.name)?;
    let nickname = default_nickname(&user.display_name);

    db::create_with_owner(
        &state.db,
        user.id,
        &body.slug,
        &name,
        &nickname,
        MAX_OWNED_TENANTS,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(MyTenant {
            slug: body.slug,
            name,
            description: None,
            role: Role::Owner,
            nickname,
            deletion_scheduled_for: None,
            logo_url: None,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/tenants",
    tag = "tenants",
    responses(
        (status = OK, body = Vec<MyTenant>, description = "Tenants de que o usuário participa, na ordem em que entrou"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
    )
)]
pub async fn list_my_tenants(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<MyTenant>>, ApiError> {
    db::list_for_user(&state.db, user.id).await.map(Json)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = MyTenant, description = "O tenant e o papel do usuário nele"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found (não existe ou o usuário não é membro)"),
    )
)]
pub async fn get_tenant(member: TenantMember) -> Json<MyTenant> {
    Json(member.into())
}

#[utoipa::path(
    patch,
    path = "/tenants/{slug}",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = UpdateTenant,
    responses(
        (status = OK, body = MyTenant, description = "Nome e descrição gravados. O slug não muda"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tenant.name_invalid, tenant.description_invalid {max}, request.invalid_body"),
    )
)]
pub async fn update_tenant(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<UpdateTenant>,
) -> Result<Json<MyTenant>, ApiError> {
    let name = match &body.name {
        Some(raw) => clean_name(raw)?,
        None => owner.name.clone(),
    };
    let description = match &body.description {
        Some(raw) => {
            let text = clean_long_text(raw, DESCRIPTION_MAX, "tenant.description_invalid")?;
            (!text.is_empty()).then_some(text)
        }
        None => owner.description.clone(),
    };
    let mut tx = state.db.begin().await?;
    db::update_details(&mut tx, owner.tenant, &name, description.as_deref()).await?;
    changes::changed(&mut tx, owner.tenant).await?;
    tx.commit().await?;
    Ok(Json(MyTenant {
        name,
        description,
        ..owner.into()
    }))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/transfer-ownership",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = TransferOwnership,
    responses(
        (status = OK, body = MyTenant, description = "Posse transferida: o membro escolhido vira owner (dentro do limite de 3 tenants como owner) e quem era owner vira organizer. Devolve o tenant visto por quem transferiu"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só o owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo)"),
        (status = CONFLICT, body = ErrorBody, description = "member.is_owner (já é o owner), member.owner_limit_reached {max}, member.deletion_scheduled (o membro pediu a exclusão da conta), tenant.deletion_scheduled"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "tenant.confirmation_mismatch (o slug digitado não confere), request.invalid_body"),
    )
)]
pub async fn transfer_ownership(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    JsonBody(body): JsonBody<TransferOwnership>,
) -> Result<Json<MyTenant>, ApiError> {
    if body.confirm_slug.trim() != owner.slug {
        return Err(ApiError::invalid("tenant.confirmation_mismatch"));
    }
    if body.user_id == owner.user.id {
        return Err(ApiError::conflict("member.is_owner"));
    }
    let mut tx = state.db.begin().await?;
    db::transfer_ownership(
        &mut tx,
        owner.tenant,
        owner.user.id,
        body.user_id,
        MAX_OWNED_TENANTS,
    )
    .await?;
    changes::changed(&mut tx, owner.tenant).await?;
    // A home do novo owner muda: o papel em "Suas comunidades" e, se era
    // player, o que espera a organização.
    outbox::emit(&mut tx, Event::me([body.user_id])).await?;
    tx.commit().await?;
    Ok(Json(MyTenant {
        role: Role::Organizer,
        ..owner.into()
    }))
}
