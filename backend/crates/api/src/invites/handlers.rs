use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgConnection;

use super::{
    db::{self, StoredInvite},
    types::{
        AcceptedInvite, CreatedInvite, Invite, InviteMembership, InvitePath, InvitePreview,
        InviteViewer, InvitedTournament,
    },
};
use crate::{
    AppState,
    auth::CurrentUser,
    participants::{self, Entry},
    shared::{ApiError, ErrorBody, PathParams, token},
    tenants::{self, Organizer, TenantId, TenantMember, default_nickname},
    tournaments::{self, NameTranslation, StoredTournament},
};

/// Validade de um link de convite do tenant (o de torneio vale enquanto a
/// inscrição estiver aberta).
const INVITE_DAYS: i32 = 7;

#[utoipa::path(
    post,
    path = "/tenants/{slug}/invites",
    tag = "invites",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = CREATED, body = CreatedInvite, description = "Link multiuso, válido por 7 dias; quem entra por ele vira player"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn create_invite(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
) -> Result<(StatusCode, Json<CreatedInvite>), ApiError> {
    let token = token::random_token();
    let mut tx = state.db.begin().await?;
    let (id, expires_at) =
        db::create(&mut tx, member.tenant, &token, member.user.id, INVITE_DAYS).await?;
    tenants::changed(&mut tx, member.tenant).await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedInvite {
            id,
            token,
            expires_at,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/invites",
    tag = "invites",
    params(("slug" = String, Path, description = "Slug do tenant")),
    responses(
        (status = OK, body = Vec<Invite>, description = "Convites ativos do tenant (os links de inscrição ficam em cada torneio), do mais novo ao mais antigo"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found"),
    )
)]
pub async fn list_invites(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
) -> Result<Json<Vec<Invite>>, ApiError> {
    Ok(Json(db::list_active(&state.db, member.tenant).await?))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/invites/{invite_id}",
    tag = "invites",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("invite_id" = i64, Path, description = "Id do convite (do tenant ou link de inscrição de um torneio)"),
    ),
    responses(
        (status = NO_CONTENT, description = "Convite revogado (também se já estava)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, invite.not_found"),
    )
)]
pub async fn revoke_invite(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<InvitePath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    if !db::revoke(&mut tx, member.tenant, path.invite_id).await? {
        return Err(ApiError::not_found("invite.not_found"));
    }
    // Também os links de inscrição: a aba do torneio recebe o aviso da comunidade.
    tenants::changed(&mut tx, member.tenant).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/invites/{token}",
    tag = "invites",
    params(("token" = String, Path, description = "Token do link de convite")),
    responses(
        (status = OK, body = InvitePreview, description = "Convite válido (não exige login). No link de inscrição, o torneio vem com status, vagas e início: só dá para aceitar com a inscrição aberta e vaga sobrando. Com login, `viewer` diz se quem abre já é membro, se saiu (aceitar o traz de volta com o apelido e o histórico) e se já está inscrito no torneio do link"),
        (status = NOT_FOUND, body = ErrorBody, description = "invite.not_found"),
        (status = GONE, body = ErrorBody, description = "invite.expired, invite.revoked"),
    )
)]
pub async fn preview_invite(
    State(state): State<AppState>,
    user: Option<CurrentUser>,
    PathParams(token): PathParams<String>,
) -> Result<Json<InvitePreview>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let invite = find_valid_invite(&mut conn, &token).await?;
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
        expires_at: invite.expires_at,
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
        (status = NOT_FOUND, body = ErrorBody, description = "invite.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.registration_closed, tournament.full {max}"),
        (status = GONE, body = ErrorBody, description = "invite.expired, invite.revoked"),
    )
)]
pub async fn accept_invite(
    State(state): State<AppState>,
    user: CurrentUser,
    PathParams(token): PathParams<String>,
) -> Result<Json<AcceptedInvite>, ApiError> {
    let mut tx = state.db.begin().await?;
    let invite = find_valid_invite(&mut tx, &token).await?;
    // Trava o torneio antes de entrar no tenant: a vaga conferida vale até o commit.
    let tournament = match &invite.tournament {
        Some(slug) => tournaments::find_for_update(&mut tx, invite.tenant, slug, Some(user.id))
            .await?
            .map(Some)
            .ok_or_else(ApiError::internal)?,
        None => None,
    };
    let nickname = default_nickname(&user.display_name);
    db::join_as_player(&mut tx, invite.tenant, user.id, &nickname, invite.id).await?;
    tenants::changed(&mut tx, invite.tenant).await?;
    if let Some(t) = &tournament {
        let entry = Entry::Invite(invite.id);
        participants::register(&mut tx, invite.tenant, t, user.id, entry).await?;
    }
    tx.commit().await?;

    Ok(Json(AcceptedInvite {
        tenant: tenants::my_tenant(&state.db, invite.tenant, user.id).await?,
        tournament: tournament.map(|t| t.slug),
    }))
}

/// Convite pelo token: inexistente → 404, revogado ou expirado → 410.
async fn find_valid_invite(conn: &mut PgConnection, token: &str) -> Result<StoredInvite, ApiError> {
    let invite = db::find_by_token_hash(conn, &token::hash_token(token))
        .await?
        .ok_or(ApiError::not_found("invite.not_found"))?;
    if invite.revoked {
        return Err(ApiError::gone("invite.revoked"));
    }
    if invite.expired {
        return Err(ApiError::gone("invite.expired"));
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
        translations: t
            .translations
            .into_iter()
            .map(|tr| NameTranslation {
                locale: tr.locale,
                name: tr.name,
            })
            .collect(),
        status: t.status,
        max_participants: t.setup.max_participants,
        participants: t.participants,
        starts_at: t.schedule.starts_at,
    })
}
