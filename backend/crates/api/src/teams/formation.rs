//! A formação dos times do TDM antes do início, pelo organizador: ver os
//! times e os reservas, criar um time com um jogador, pôr e tirar jogadores
//! (o sorteio, que precisa do rating, fica em `lifecycle`). Os times se
//! refazem à vontade até o início; depois, só nome e capitão
//! (`roster::update_team`) e a substituição (`roster::substitute`).

use axum::{Json, extract::State};
use sqlx::PgConnection;

use super::{
    db,
    types::{FormationPlayer, NewTeam, TeamFormation, TeamMemberPath},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    tenants::{Organizer, TenantMember},
    tournaments::{self, StoredTournament, TournamentPath},
};

const NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");

/// A formação como a API mostra: os times (com os jogadores) e os inscritos
/// sem time.
pub async fn formation(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<TeamFormation, sqlx::Error> {
    let teams = db::list(conn, tournament.id, false).await?;
    let players = db::players(conn, tournament.id).await?;
    Ok(TeamFormation {
        team_size: tournament.setup.team_size,
        has_check_in: tournament.schedule.has_check_in(),
        teams: teams.iter().map(|t| t.view()).collect(),
        checked_in: players
            .iter()
            .filter(|p| p.checked_in)
            .map(|p| p.user_id)
            .collect(),
        reserves: players
            .into_iter()
            .filter(|p| p.team_id.is_none())
            .map(|p| FormationPlayer {
                user_id: p.user_id,
                nickname: p.nickname,
                avatar_url: p.avatar_url,
                registered_at: p.registered_at,
                left: p.left,
            })
            .collect(),
    })
}

/// O torneio TDM travado, antes de começar: o que toda mudança da formação
/// confere. 409 `tournament.participants_locked` depois de começar e
/// `tournament.not_tdm` no duelo.
pub async fn editable<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let tournament = tournaments::find_for_update(conn, member.tenant, slug, None)
        .await?
        .ok_or(NOT_FOUND)?;
    if !tournament.setup.teams() {
        return Err(ApiError::conflict("tournament.not_tdm"));
    }
    if !tournament.status.participants_editable() {
        return Err(ApiError::conflict("tournament.participants_locked"));
    }
    Ok(tournament)
}

/// `user_id` é inscrito no torneio. 404 `participant.not_found`.
async fn registered(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    user_id: i64,
) -> Result<db::Player, ApiError> {
    db::players(conn, tournament.id)
        .await?
        .into_iter()
        .find(|p| p.user_id == user_id)
        .ok_or(ApiError::not_found("participant.not_found"))
}

/// Grava, emite a mudança e devolve a formação.
async fn respond(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
) -> Result<TeamFormation, ApiError> {
    tournaments::changed(conn, tournament.id).await?;
    Ok(formation(conn, tournament).await?)
}

#[utoipa::path(
    get,
    path = "/tenants/{slug}/tournaments/{tournament}/teams",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    responses(
        (status = OK, body = TeamFormation, description = "Os times do torneio com os jogadores e os inscritos sem time (reservas). No duelo, cada inscrito é um time de um"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
    )
)]
pub async fn get_teams(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TournamentPath>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let tournament = tournaments::find_visible(&mut conn, &member, &path.tournament).await?;
    Ok(Json(formation(&mut conn, &tournament).await?))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/teams",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = NewTeam,
    responses(
        (status = OK, body = TeamFormation, description = "Time novo com o inscrito como capitão (ele sai do time em que estava; o time que fica vazio some)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, participant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_tdm, tournament.participants_locked (já começou)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn create_team(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<NewTeam>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = editable(&mut tx, &member, &path.tournament).await?;
    registered(&mut tx, &tournament, body.user_id).await?;
    db::leave_team(&mut tx, tournament.id, body.user_id).await?;
    db::create(&mut tx, member.tenant, tournament.id, &[body.user_id]).await?;
    let formation = respond(&mut tx, &tournament).await?;
    tx.commit().await?;
    Ok(Json(formation))
}

#[utoipa::path(
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/teams/{team}/members/{user_id}",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("team" = i64, Path, description = "Id do time"),
        ("user_id" = i64, Path, description = "O inscrito"),
    ),
    responses(
        (status = OK, body = TeamFormation, description = "O inscrito entra no time (sai do que estava; o capitão que sai passa a capitania ao jogador inscrito há mais tempo; o time que fica vazio some). Já estar no time não muda nada"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, team.not_found, participant.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_tdm, tournament.participants_locked, team.full {max}"),
    )
)]
pub async fn put_member(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TeamMemberPath>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = editable(&mut tx, &member, &path.tournament).await?;
    let player = registered(&mut tx, &tournament, path.user_id).await?;
    let team = db::lock_team(&mut tx, tournament.id, path.team)
        .await?
        .ok_or(ApiError::not_found("team.not_found"))?;
    if player.team_id != Some(team.id) {
        let max = tournament.setup.team_size;
        if team.members.len() >= max as usize {
            return Err(ApiError::conflict("team.full").with_params([("max", i64::from(max))]));
        }
        db::leave_team(&mut tx, tournament.id, path.user_id).await?;
        db::set_team(&mut tx, tournament.id, path.user_id, Some(team.id)).await?;
    }
    let formation = respond(&mut tx, &tournament).await?;
    tx.commit().await?;
    Ok(Json(formation))
}

#[utoipa::path(
    delete,
    path = "/tenants/{slug}/tournaments/{tournament}/teams/{team}/members/{user_id}",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("team" = i64, Path, description = "Id do time"),
        ("user_id" = i64, Path, description = "O jogador"),
    ),
    responses(
        (status = OK, body = TeamFormation, description = "O jogador sai do time e fica de reserva (o capitão passa a capitania; o time que fica vazio some)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, team.not_found, participant.not_found (não está nesse time)"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_tdm, tournament.participants_locked"),
    )
)]
pub async fn delete_member(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TeamMemberPath>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = editable(&mut tx, &member, &path.tournament).await?;
    let team = db::lock_team(&mut tx, tournament.id, path.team)
        .await?
        .ok_or(ApiError::not_found("team.not_found"))?;
    if !team.members.contains(&path.user_id) {
        return Err(ApiError::not_found("participant.not_found"));
    }
    db::leave_team(&mut tx, tournament.id, path.user_id).await?;
    let formation = respond(&mut tx, &tournament).await?;
    tx.commit().await?;
    Ok(Json(formation))
}
