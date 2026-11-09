use axum::{Json, extract::State, http::HeaderMap};
use sqlx::PgConnection;

use super::{
    db::{self, StoredInvite},
    limit,
    types::{
        AcceptedInvite, InviteLink, InviteMembership, InvitePreview, InviteViewer,
        InvitedTournament, InvitedTranslation,
    },
};
use crate::{
    AppState,
    auth::CurrentUser,
    participants::{self, Entry},
    shared::{ApiError, ClientIp, ErrorBody, PathParams, public_url},
    tenants::{self, Organizer, TenantId, TenantMember, default_nickname},
    tournaments::{self, StoredTournament},
};

#[utoipa::path(
    get,
    path = "/tenants/{slug}/invite",
    tag = "invites",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = InviteLink, description = "O link de convite da comunidade: multiuso e sem validade; quem entra por ele vira player"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn get_invite_link(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
) -> Result<Json<InviteLink>, ApiError> {
    let token = db::tenant_token(&state.db, member.tenant).await?;
    Ok(Json(InviteLink { token }))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/invite/rotate",
    tag = "invites",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = InviteLink, description = "O link novo. O antigo deixa de valer (responde invite.not_found); quem já entrou continua"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled"),
    )
)]
pub async fn rotate_invite_link(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
) -> Result<Json<InviteLink>, ApiError> {
    let mut tx = state.db.begin().await?;
    let token = tenants::free_invite_code(&mut tx).await?;
    db::set_tenant_token(&mut tx, member.tenant, &token).await?;
    tenants::changed(&mut tx, member.tenant).await?;
    tx.commit().await?;
    Ok(Json(InviteLink { token }))
}

#[utoipa::path(
    get,
    path = "/invites/{token}",
    tag = "invites",
    params(("token" = String, Path, description = "Token do link de convite")),
    responses(
        (status = OK, body = InvitePreview, description = "Convite que vale (não exige login). No link de inscrição, o torneio vem com status, vagas e início: só dá para aceitar com a inscrição aberta e vaga sobrando. Com login, `viewer` diz se quem abre já é membro, se saiu (aceitar o traz de volta com o apelido e o histórico) e se já está inscrito no torneio do link"),
        (status = NOT_FOUND, body = ErrorBody, description = "invite.not_found (também o link que foi trocado por outro)"),
        (status = CONFLICT, body = ErrorBody, description = "tenant.deletion_scheduled (a comunidade vai ser excluída)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "invite.rate_limited {minutes} (limite por IP)"),
    )
)]
pub async fn preview_invite(
    State(state): State<AppState>,
    ip: ClientIp,
    headers: HeaderMap,
    user: Option<CurrentUser>,
    PathParams(token): PathParams<String>,
) -> Result<Json<InvitePreview>, ApiError> {
    limit::hit(&state, &headers, &ip)?;
    let mut conn = state.db.acquire().await?;
    let invite = find_invite(&mut conn, &token).await?;
    let user_id = user.as_ref().map(|u| u.id);
    let found = match &invite.tournament {
        Some(slug) => Some(tournaments::find(&mut conn, invite.tenant, slug, user_id).await?),
        None => None,
    };
    let registered = found
        .as_ref()
        .is_some_and(|t| t.as_ref().is_some_and(|t| t.registered));
    let viewer = match user_id {
        Some(id) => Some(viewer(&state, invite.tenant, id, registered).await?),
        None => None,
    };
    Ok(Json(InvitePreview {
        tenant_slug: invite.tenant_slug,
        tenant_name: invite.tenant_name,
        tenant_description: invite.tenant_description,
        tenant_logo_url: invite.tenant_logo_key.as_deref().and_then(public_url),
        tenant_cover_url: invite.tenant_cover_key.as_deref().and_then(public_url),
        owner_name: invite.owner_name,
        member_count: invite.member_count,
        tournament: found.map(invited).transpose()?,
        viewer,
    }))
}

/// Quem abre o link, logado: membro, ex-membro ou de fora.
async fn viewer(
    state: &AppState,
    tenant: TenantId,
    user_id: i64,
    registered: bool,
) -> Result<InviteViewer, ApiError> {
    let standing = tenants::standing(&state.db, tenant, user_id).await?;
    Ok(InviteViewer {
        membership: match &standing {
            None => InviteMembership::None,
            Some(s) if s.active => InviteMembership::Member,
            Some(_) => InviteMembership::Former,
        },
        nickname: standing.map(|s| s.nickname),
        registered,
    })
}

#[utoipa::path(
    post,
    path = "/invites/{token}/accept",
    tag = "invites",
    params(("token" = String, Path, description = "Token do link de convite")),
    responses(
        (status = OK, body = AcceptedInvite, description = "Entrou como player (quem já era membro mantém papel e apelido; quem tinha saído volta como player, com o apelido e o histórico de antes) e, no link de inscrição, ficou inscrito no torneio. Se a inscrição falha, também não entra no tenant"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "invite.not_found (também o link que foi trocado por outro)"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.registration_closed, tournament.full {max}, tenant.deletion_scheduled (a comunidade vai ser excluída)"),
        (status = TOO_MANY_REQUESTS, body = ErrorBody, description = "invite.rate_limited {minutes} (limite por IP)"),
    )
)]
pub async fn accept_invite(
    State(state): State<AppState>,
    ip: ClientIp,
    headers: HeaderMap,
    user: CurrentUser,
    PathParams(token): PathParams<String>,
) -> Result<Json<AcceptedInvite>, ApiError> {
    limit::hit(&state, &headers, &ip)?;
    let mut tx = state.db.begin().await?;
    let invite = find_invite(&mut tx, &token).await?;
    // Trava o torneio antes de entrar no tenant: a vaga conferida vale até o commit.
    let tournament = match &invite.tournament {
        Some(slug) => tournaments::find_for_update(&mut tx, invite.tenant, slug, Some(user.id))
            .await?
            .map(Some)
            .ok_or_else(ApiError::internal)?,
        None => None,
    };
    let nickname = default_nickname(&user.display_name);
    db::join_as_player(&mut tx, invite.tenant, user.id, &nickname).await?;
    tenants::changed(&mut tx, invite.tenant).await?;
    if let Some(t) = &tournament {
        participants::register(&mut tx, invite.tenant, t, user.id, Entry::Invite).await?;
    }
    tx.commit().await?;

    Ok(Json(AcceptedInvite {
        tenant: tenants::my_tenant(&state.db, invite.tenant, user.id).await?,
        tournament: tournament.map(|t| t.slug),
    }))
}

/// Convite pelo token: inexistente ou trocado por outro → 404, de comunidade
/// com a exclusão agendada → 409.
async fn find_invite(conn: &mut PgConnection, token: &str) -> Result<StoredInvite, ApiError> {
    let invite = db::find_by_token(conn, token)
        .await?
        .ok_or(ApiError::not_found("invite.not_found"))?;
    // A comunidade é só leitura até ser excluída.
    if invite.tenant_deleting {
        return Err(ApiError::conflict("tenant.deletion_scheduled"));
    }
    Ok(invite)
}

/// O torneio do link de inscrição (que existe: apagar o torneio apaga o link).
fn invited(found: Option<StoredTournament>) -> Result<InvitedTournament, ApiError> {
    let t = found.ok_or_else(ApiError::internal)?;
    Ok(InvitedTournament {
        slug: t.slug,
        locale: t.locale,
        name: t.texts.name,
        description: t.texts.description,
        rules: t.texts.rules,
        mode: t.setup.mode,
        team_size: t.setup.team_size,
        translations: t
            .translations
            .into_iter()
            .map(|tr| InvitedTranslation {
                locale: tr.locale,
                name: tr.name,
                description: tr.description,
                rules: tr.rules,
            })
            .collect(),
        status: t.status,
        max_participants: t.setup.max_participants,
        participants: t.participants,
        starts_at: t.schedule.starts_at,
        cover_url: t.cover_key.as_deref().and_then(public_url),
    })
}
