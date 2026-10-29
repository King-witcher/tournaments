//! O sorteio dos times do TDM antes do início: refaz a formação inteira com
//! os elegíveis (check-in feito; sem check-in no torneio, todos), na ordem de
//! inscrição, pelo `engine` (`teams::random` ou `teams::balanced`, com a seed
//! gravada no torneio). Os últimos inscritos que não completam um time ficam
//! de reserva. Fica aqui, e não em `teams`, porque o balanceado lê o rating
//! (`ratings`, que já depende de `teams`).

use axum::{Json, extract::State};
use engine::{core::PlayerId, teams as et};

use crate::{
    AppState, ratings,
    shared::{ApiError, ErrorBody, JsonBody, PathParams},
    teams::{self, DrawMethod, DrawTeams, TeamFormation},
    tenants::{Organizer, TenantMember},
    tournaments::{self, TournamentPath},
};

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/teams/draw",
    tag = "teams",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
    ),
    request_body = DrawTeams,
    responses(
        (status = OK, body = TeamFormation, description = "Times refeitos por sorteio com os elegíveis (check-in feito; sem check-in no torneio, todos), na ordem de inscrição: os últimos que não completam um time ficam de reserva, e quem não fez check-in fica sem time. `random`: embaralhados, capitão sorteado; `balanced`: zigue-zague pelo rating da comunidade (inicial para quem não tem; empates pela seed), capitão o de maior rating. A seed fica no torneio; os seeds da chave somem com os times antigos"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_tdm, tournament.participants_locked (já começou)"),
        (status = UNPROCESSABLE_ENTITY, body = ErrorBody, description = "request.invalid_body"),
    )
)]
pub async fn draw_teams(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TournamentPath>,
    JsonBody(body): JsonBody<DrawTeams>,
) -> Result<Json<TeamFormation>, ApiError> {
    let mut tx = state.db.begin().await?;
    let tournament = teams::formation_editable(&mut tx, &member, &path.tournament).await?;
    let has_check_in = tournament.schedule.has_check_in();
    let eligible: Vec<i64> = teams::players(&mut tx, tournament.id)
        .await?
        .into_iter()
        .filter(|p| p.checked_in || !has_check_in)
        .map(|p| p.user_id)
        .collect();
    let size = usize::try_from(tournament.setup.team_size).unwrap_or(1);
    let seed: u64 = rand::random();
    let formation = match body.method {
        DrawMethod::Random => {
            let players: Vec<PlayerId> = eligible.iter().copied().map(PlayerId).collect();
            et::random(&players, size, seed)
        }
        DrawMethod::Balanced => {
            let ratings = ratings::current(&mut tx, member.tenant, &eligible).await?;
            let players: Vec<(PlayerId, i32)> = eligible
                .iter()
                .map(|u| (PlayerId(*u), ratings.get(u).copied().unwrap_or_default()))
                .collect();
            et::balanced(&players, size, seed)
        }
    }
    .map_err(|e| {
        // Os elegíveis vêm do banco, um por inscrição, e o tamanho já foi
        // validado: erro nosso.
        tracing::error!(%e, "sorteio dos times recusado");
        ApiError::internal()
    })?;
    teams::clear_teams(&mut tx, tournament.id).await?;
    for team in &formation.teams {
        let ids: Vec<i64> = team.players.iter().map(|p| p.0).collect();
        teams::create_team(&mut tx, member.tenant, tournament.id, &ids).await?;
    }
    teams::save_teams_seed(&mut tx, tournament.id, seed).await?;
    tournaments::changed(&mut tx, tournament.id).await?;
    let view = teams::formation(&mut tx, &tournament).await?;
    tx.commit().await?;
    Ok(Json(view))
}
