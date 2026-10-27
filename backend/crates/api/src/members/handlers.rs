use axum::{Json, extract::State};

use super::{
    db,
    types::{Member, MemberPath, SetRole, UpdateMyMembership},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{MyTenant, Owner, Role, TenantMember, validate_nickname},
};

#[utoipa::path(
    patch,
    path = "/tenants/{slug}/me",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    request_body = UpdateMyMembership,
    responses(
        (status = OK, body = MyTenant, description = "Apelido atualizado"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "member.nickname_invalid, request.invalid_body"),
    )
)]
pub async fn update_my_membership(
    State(state): State<AppState>,
    member: TenantMember,
    JsonBody(body): JsonBody<UpdateMyMembership>,
) -> Result<Json<MyTenant>, ApiError> {
    let nickname = validate_nickname(&body.nickname)?;
    db::set_nickname(&state.db, member.tenant, member.user.id, &nickname).await?;
    Ok(Json(MyTenant {
        nickname,
        ..member.into()
    }))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Vec<Member>, description = "Membros: owner, organizers e players, cada grupo por apelido"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn list_members(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<Json<Vec<Member>>, ApiError> {
    db::list(&state.db, member.tenant, None).await.map(Json)
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/members/{user_id}/role",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    request_body = SetRole,
    responses(
        (status = OK, body = Member, description = "Papel trocado (só o owner pode)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "member.is_owner (o papel do owner não muda)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn set_member_role(
    State(state): State<AppState>,
    owner: TenantMember<Owner>,
    PathParams(path): PathParams<MemberPath>,
    JsonBody(body): JsonBody<SetRole>,
) -> Result<Json<Member>, ApiError> {
    let user_id = path.user_id;
    let current = db::role_of(&state.db, owner.tenant, user_id)
        .await?
        .ok_or(ApiError::not_found("member.not_found"))?;
    if current == Role::Owner {
        return Err(ApiError::conflict("member.is_owner"));
    }

    db::set_role(&state.db, owner.tenant, user_id, body.role.into()).await?;

    db::list(&state.db, owner.tenant, Some(user_id))
        .await?
        .pop()
        .map(Json)
        .ok_or(ApiError::not_found("member.not_found"))
}
