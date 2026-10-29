//! O time depois de formado: nome e capitão (o organizador, e o capitão no
//! nome, até o torneio encerrar) e a substituição de um jogador pelo
//! organizador com o torneio em andamento. Quem joga cada mapa é quem está
//! no time e na comunidade quando o mapa é enviado (o banco grava), então a
//! substituição vale a partir do próximo mapa enviado; no veto, o elenco da
//! escolha simultânea muda na hora.

use axum::{Json, extract::State};

use super::{
    db,
    formation::formation,
    types::{Substitution, TeamFormation, TeamPath, UpdateTeam},
};
use crate::{
    AppState,
    shared::{ApiError, ErrorBody, JsonBody, PathParams, text::clean_text},
    tenants::{Organizer, Role, TenantMember},
    tournaments::{self, TournamentStatus},
};

/// Tamanho do nome do time (o mesmo check está na tabela).
const MAX_NAME: usize = 40;

const NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");
const TEAM_NOT_FOUND: ApiError = ApiError::not_found("team.not_found");

#[utoipa::path(
    patch,
    path = "/tenants/{slug}/tournaments/{tournament}/teams/{team}",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("team" = i64, Path, description = "Id do time"),
    ),
    request_body = UpdateTeam,
    responses(
        (status = OK, body = TeamFormation, description = "Nome e capitão gravados (o mesmo valor não muda nada)"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "team.not_captain (o nome é do organizador e do capitão), tenant.forbidden (o capitão é só do organizador)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, team.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_tdm, tournament.finished, team.name_taken"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "team.name_invalid {max}, team.captain_invalid (não é jogador do time), request.invalid_body"),
    )
)]
pub async fn update_team(
    State(state): State<AppState>,
    member: TenantMember,
    PathParams(path): PathParams<TeamPath>,
    JsonBody(body): JsonBody<UpdateTeam>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament =
        tournaments::find_visible_for_update(&mut tx, &member, &path.tournament).await?;
    if !tournament.setup.teams() {
        return Err(ApiError::conflict("tournament.not_tdm"));
    }
    let team = db::lock_team(&mut tx, tournament.id, path.team)
        .await?
        .ok_or(TEAM_NOT_FOUND)?;
    let organizer = member.role >= Role::Organizer;
    if !organizer && team.captain != member.user.id {
        return Err(ApiError::forbidden("team.not_captain"));
    }
    if body.captain.is_some() && !organizer {
        return Err(ApiError::forbidden("tenant.forbidden"));
    }
    if !tournament.status.texts_editable() {
        return Err(ApiError::conflict("tournament.finished"));
    }
    if let Some(name) = &body.name {
        let name = match name.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(raw) => Some(
                clean_text(raw, MAX_NAME, "team.name_invalid")
                    .map_err(|e| e.with_params([("max", MAX_NAME as i64)]))?,
            ),
        };
        db::set_name(&mut tx, tournament.id, team.id, name.as_deref()).await?;
    }
    if let Some(captain) = body.captain
        && captain != team.captain
    {
        if !team.members.contains(&captain) {
            return Err(ApiError::invalid("team.captain_invalid"));
        }
        db::set_captain(&mut tx, tournament.id, team.id, captain).await?;
    }
    tournaments::changed(&mut tx, tournament.id).await?;
    let formation = formation(&mut tx, &tournament).await?;
    tx.commit().await?;
    Ok(Json(formation))
}

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/teams/{team}/substitution",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("team" = i64, Path, description = "Id do time"),
    ),
    request_body = Substitution,
    responses(
        (status = OK, body = TeamFormation, description = "Com o torneio em andamento, `out` sai do time (fica inscrito, sem time) e `in` entra: um reserva ou outro membro ativo da comunidade, que fica inscrito já no time. Vale a partir do próximo mapa enviado (a escalação de cada mapa é gravada no envio) e, no veto em andamento, na escolha simultânea de campeões. Se `out` era o capitão, a capitania passa ao jogador do time inscrito há mais tempo"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, team.not_found, member.not_found (quem entra não é membro ativo)"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_tdm, tournament.not_in_progress, team.not_in_bracket, team.withdrawn (desistiu), team.player_in_team (quem entra já joga num time)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "team.member_invalid (quem sai não é do time, ou é o mesmo que entra), request.invalid_body"),
    )
)]
pub async fn substitute(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TeamPath>,
    JsonBody(body): JsonBody<Substitution>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = tournaments::find_for_update(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(NOT_FOUND)?;
    if !tournament.setup.teams() {
        return Err(ApiError::conflict("tournament.not_tdm"));
    }
    if tournament.status != TournamentStatus::InProgress {
        return Err(ApiError::conflict("tournament.not_in_progress"));
    }
    let team = db::lock_team(&mut tx, tournament.id, path.team)
        .await?
        .ok_or(TEAM_NOT_FOUND)?;
    if team.seed.is_none() {
        return Err(ApiError::conflict("team.not_in_bracket"));
    }
    if team.withdrawn {
        return Err(ApiError::conflict("team.withdrawn"));
    }
    if !team.members.contains(&body.out) || body.out == body.sub {
        return Err(ApiError::invalid("team.member_invalid"));
    }
    let players = db::players(&mut tx, tournament.id).await?;
    match players.iter().find(|p| p.user_id == body.sub) {
        Some(p) if p.team_id.is_some() => {
            return Err(ApiError::conflict("team.player_in_team"));
        }
        Some(p) if p.left => return Err(ApiError::not_found("member.not_found")),
        Some(_) => {
            db::set_team(&mut tx, tournament.id, body.sub, Some(team.id)).await?;
        }
        None => {
            let inserted = db::insert_substitute(
                &mut tx,
                member.tenant,
                tournament.id,
                body.sub,
                team.id,
                member.user.id,
            )
            .await?;
            if !inserted {
                return Err(ApiError::not_found("member.not_found"));
            }
        }
    }
    db::hand_over(&mut tx, tournament.id, team.id, body.out, true).await?;
    db::set_team(&mut tx, tournament.id, body.out, None).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    let formation = formation(&mut tx, &tournament).await?;
    tx.commit().await?;
    Ok(Json(formation))
}
