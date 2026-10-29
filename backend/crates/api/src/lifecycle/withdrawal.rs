//! Quem deixa um torneio no meio do caminho. Num torneio em andamento, o time
//! desiste (o organizador marca, ou o último jogador dele sai da comunidade;
//! no duelo, o próprio jogador): continua na chave e cada partida pronta com
//! ele, agora ou depois, vira W.O. para o adversário
//! (`matches::settle_withdrawals`). O membro que sai da comunidade também
//! sai das inscrições dos torneios que não começaram; nos encerrados, a
//! inscrição fica como está.

use axum::{extract::State, http::StatusCode};
use chrono::Utc;
use sqlx::PgConnection;

use crate::{
    AppState, matches, participants,
    shared::{ApiError, ErrorBody, PathParams},
    teams::{self, TeamPath},
    tenants::{Organizer, TenantId, TenantMember},
    tournaments::{self, StoredTournament, TournamentStatus},
};

#[utoipa::path(
    post,
    path = "/tenants/{slug}/tournaments/{tournament}/teams/{team}/withdrawal",
    tag = "lifecycle",
    params(
        ("slug" = String, Path, description = "Slug do tenant"),
        ("tournament" = String, Path, description = "Slug do torneio"),
        ("team" = i64, Path, description = "Id do time (no duelo, o `team_id` do inscrito)"),
    ),
    responses(
        (status = NO_CONTENT, description = "Desistência do time marcada pelo organizador (também se já estava): o time continua na chave e os jogadores na comunidade, e cada partida pronta com ele, agora ou depois, vira W.O. automático para o adversário (com os dois lados fora, para o seed melhor). Envio pendente ou contestado dessas partidas dá lugar ao W.O. Não se desfaz; o organizador ainda pode corrigir resultados"),
        (status = BAD_REQUEST, body = ErrorBody, description = "request.invalid_path"),
        (status = UNAUTHORIZED, body = ErrorBody, description = "auth.unauthenticated"),
        (status = FORBIDDEN, body = ErrorBody, description = "tenant.forbidden (só organizer ou owner)"),
        (status = NOT_FOUND, body = ErrorBody, description = "tenant.not_found, tournament.not_found, team.not_found"),
        (status = CONFLICT, body = ErrorBody, description = "tournament.not_in_progress, team.not_in_bracket (time que ficou fora da chave)"),
    )
)]
pub async fn withdraw_team(
    State(state): State<AppState>,
    member: TenantMember<Organizer>,
    PathParams(path): PathParams<TeamPath>,
) -> Result<StatusCode, ApiError> {
    let mut tx = state.db.begin().await?;
    let locked = tournaments::find_for_update(&mut tx, member.tenant, &path.tournament, None)
        .await?
        .ok_or(ApiError::not_found("tournament.not_found"))?;
    let t = after_due(&mut tx, locked, None).await?;
    if t.status != TournamentStatus::InProgress {
        return Err(ApiError::conflict("tournament.not_in_progress"));
    }
    match teams::withdraw(&mut tx, t.id, path.team, Some(member.user.id)).await? {
        None => return Err(ApiError::not_found("team.not_found")),
        Some(false) => return Err(ApiError::conflict("team.not_in_bracket")),
        Some(true) => {}
    }
    matches::settle_withdrawals(&mut tx, &t).await?;
    tournaments::changed(&mut tx, t.id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `user_id` deixa o tenant (`by`: quem removeu; `None`, saiu sozinho): sai
/// das inscrições dos torneios que não começaram e, nos em andamento em que o
/// time dele está na chave, o time desiste se ninguém mais dele fica na
/// comunidade (no duelo, sempre). Trava, na ordem dos ids, todo torneio que a saída
/// pode mudar (também os que não encerraram e em que ele não está, para uma
/// inscrição dele no mesmo instante esperar e ver que ele saiu).
pub async fn leave_tournaments(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
    by: Option<i64>,
) -> Result<(), ApiError> {
    for slug in tournaments::slugs_for_member(conn, tenant, user_id).await? {
        let viewer = Some(user_id);
        let Some(t) = tournaments::find_for_update(conn, tenant, &slug, viewer).await? else {
            continue;
        };
        if !t.registered {
            continue;
        }
        if t.status.participants_editable() {
            participants::unregister(conn, t.id, user_id).await?;
            tournaments::changed_with(conn, t.id, &[user_id]).await?;
            continue;
        }
        let t = after_due(conn, t, viewer).await?;
        if t.status == TournamentStatus::InProgress
            && teams::withdraw_departing(conn, t.id, user_id, by).await?
        {
            matches::settle_withdrawals(conn, &t).await?;
            tournaments::changed(conn, t.id).await?;
        }
    }
    Ok(())
}

/// O torneio travado `t` depois das confirmações por tempo vencidas, que
/// valem antes da desistência (e podem encerrá-lo).
async fn after_due(
    conn: &mut PgConnection,
    t: StoredTournament,
    viewer: Option<i64>,
) -> Result<StoredTournament, ApiError> {
    if !matches!(t.status, TournamentStatus::InProgress)
        || matches::apply_due_confirmations(conn, &t, Utc::now()).await? == 0
    {
        return Ok(t);
    }
    tournaments::find(conn, t.tenant, &t.slug, viewer)
        .await?
        .ok_or_else(ApiError::internal)
}
