//! A formação dos times do TDM antes do início, pelo organizador: ver os
//! times e os reservas e salvar o arranjo inteiro de uma vez (o sorteio, que
//! precisa do rating, fica em `lifecycle`). Os times se refazem à vontade até
//! o início; depois, só nome e capitão (`roster::update_team`) e a
//! substituição (`roster::substitute`).

use std::collections::{HashMap, HashSet};

use axum::{Json, extract::State};
use sqlx::PgConnection;

use super::{
    db,
    types::{FormationDraft, FormationPlayer, TeamFormation},
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
        registration_order: players.iter().map(|p| p.user_id).collect(),
        reserves: players
            .into_iter()
            .filter(|p| p.team_id.is_none())
            .map(|p| FormationPlayer {
                user_id: p.user_id,
                nickname: p.nickname,
                avatar_url: p.avatar_url,
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

/// Confere a formação enviada contra os times e os inscritos do torneio e
/// devolve os times que ficam, sem os vazios.
fn checked(
    draft: FormationDraft,
    current: &HashMap<i64, i64>,
    players: &[db::Player],
    team_size: i32,
) -> Result<Vec<db::ArrangedTeam>, ApiError> {
    let registered: HashSet<i64> = players.iter().map(|p| p.user_id).collect();
    let mut seen_teams = HashSet::new();
    let mut seen_players = HashSet::new();
    let mut teams = Vec::new();
    for team in draft.teams {
        if let Some(id) = team.id {
            if !current.contains_key(&id) {
                return Err(ApiError::not_found("team.not_found"));
            }
            if !seen_teams.insert(id) {
                return Err(ApiError::invalid("request.invalid_body"));
            }
        }
        if team.members.len() > team_size as usize {
            return Err(
                ApiError::conflict("team.full").with_params([("max", i64::from(team_size))])
            );
        }
        for user in &team.members {
            if !registered.contains(user) {
                return Err(ApiError::not_found("participant.not_found"));
            }
            if !seen_players.insert(*user) {
                return Err(ApiError::conflict("team.player_in_team"));
            }
        }
        if team.members.is_empty() {
            continue;
        }
        let captain = team
            .captain
            .filter(|c| team.members.contains(c))
            .ok_or(ApiError::invalid("team.captain_invalid"))?;
        teams.push(db::ArrangedTeam {
            id: team.id,
            members: team.members,
            captain,
        });
    }
    Ok(teams)
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
    put,
    path = "/tenants/{slug}/tournaments/{tournament}/teams/formation",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = FormationDraft,
    responses(
        (status = OK, body = TeamFormation, description = "A formação inteira gravada numa transação: cada time com seus jogadores e capitão (time existente guarda o nome e o seed; sem id, nasce um novo), quem não aparece fica de reserva e o time que não aparece, ou vem vazio, some. Time incompleto é aceito (iniciar é que exige completo)"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, team.not_found, participant.not_found (não está inscrito)"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_tdm, tournament.participants_locked (já começou), team.full {max}, team.player_in_team (jogador em dois times)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "team.captain_invalid (não é jogador do time), request.invalid_body"),
    )
)]
pub async fn save_team_formation(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<FormationDraft>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = editable(&mut tx, &member, &path.tournament).await?;
    let current = db::captains(&mut tx, tournament.id).await?;
    let players = db::players(&mut tx, tournament.id).await?;
    let teams = checked(body, &current, &players, tournament.setup.team_size)?;
    db::arrange(&mut tx, member.tenant, tournament.id, &current, &teams).await?;
    let formation = respond(&mut tx, &tournament).await?;
    tx.commit().await?;
    Ok(Json(formation))
}
