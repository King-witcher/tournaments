use axum::{Json, extract::State, http::StatusCode};

use super::{
    db, departure, history,
    types::{Member, MemberPath, MemberProfile, SetRole, UpdateMyMembership},
};
use crate::{
    AppState,
    outbox::{self, Event},
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{self, MyTenant, Organizer, Owner, Role, TenantMember, validate_nickname},
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
    let mut tx = state.db.begin().await?;
    db::set_nickname(&mut tx, member.tenant, member.user.id, &nickname).await?;
    tenants::changed(&mut tx, member.tenant).await?;
    tx.commit().await?;
    Ok(Json(MyTenant {
        nickname,
        ..member.into()
    }))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/me",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = NO_CONTENT, description = "Saiu da comunidade: perde o acesso, sai das inscrições dos torneios que não começaram e os convites que criou são revogados. Nos torneios iniciados, o nome continua na chave; nos em andamento, desiste: cada partida pronta com ele, agora ou depois, vira W.O. para o adversário. Volta por convite, como player, com o apelido de antes"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "member.owner_must_transfer (o owner transfere a posse antes)"),
    )
)]
pub async fn leave_tenant(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    departure::depart(
        &mut tx,
        member.tenant,
        member.user.id,
        None,
        departure::check_leave,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/members/{user_id}",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    responses(
        (status = NO_CONTENT, description = "Membro removido (o owner remove qualquer um; o organizer, só players): fica como quem saiu sozinho"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (player, ou organizer removendo organizer)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "member.is_owner (ninguém remove o owner), member.remove_self (para sair, DELETE /tenants/{slug}/me)"),
    )
)]
pub async fn remove_member(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<MemberPath>,
) -> Result<StatusCode, ApiError> {
    if path.user_id == member.user.id {
        return Err(ApiError::conflict("member.remove_self"));
    }
    let mut tx = state.db.begin().await?;
    let by = member.role;
    departure::depart(
        &mut tx,
        member.tenant,
        path.user_id,
        Some(member.user.id),
        |target| departure::check_removal(by, target),
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members",
    tag = "tenants",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Vec<Member>, description = "Membros: owner, organizers e players, cada grupo por apelido. `joined_via` só vem para organizer e owner"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn list_members(
    State(state): State<AppState>,
    member: TenantMember,
) -> Result<Json<Vec<Member>>, ApiError> {
    db::list(
        &state.db,
        member.tenant,
        None,
        member.role >= Role::Organizer,
    )
    .await
    .map(Json)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/members/{user_id}",
    tag = "tenants",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("user_id" = i64, Path, description = "Id do usuário membro"),
    ),
    responses(
        (status = OK, body = MemberProfile, description = "O membro na comunidade, com os torneios dela em que se inscreveu e que quem vê vê na lista da comunidade (player não vê rascunho nem privado em que não está inscrito), do início mais recente ao mais antigo (sem data, só rascunhos, primeiro; no empate, o criado por último), com a situação e a colocação dele. As confirmações por tempo vencidas são aplicadas antes"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, member.not_found (não é membro ativo: nunca foi, saiu ou foi removido)"),
    )
)]
pub async fn get_member(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<MemberPath>,
) -> Result<Json<MemberProfile>, ApiError> {
    let Member {
        user_id,
        nickname,
        display_name,
        avatar_url,
        role,
        joined_at,
        joined_via: _,
    } = db::list(&state.db, member.tenant, Some(path.user_id), false)
        .await?
        .pop()
        .ok_or(ApiError::not_found("member.not_found"))?;
    let tournaments = history::of(&state.db, &member, user_id).await?;
    Ok(Json(MemberProfile {
        user_id,
        nickname,
        display_name,
        avatar_url,
        role,
        joined_at,
        tournaments,
    }))
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

    let mut tx = state.db.begin().await?;
    db::set_role(&mut tx, owner.tenant, user_id, body.role.into()).await?;
    tenants::changed(&mut tx, owner.tenant).await?;
    // Organizer vê rascunhos: "Seus torneios" dele pode mudar.
    outbox::emit(&mut tx, Event::me([user_id])).await?;
    tx.commit().await?;

    db::list(&state.db, owner.tenant, Some(user_id), true)
        .await?
        .pop()
        .map(Json)
        .ok_or(ApiError::not_found("member.not_found"))
}
