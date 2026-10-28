//! Peças comuns às ações sobre uma partida (`report`, `manage`): as
//! checagens com código de erro e a resposta depois de gravar.

use engine::core::{Match, MatchState, ParticipantId};
use sqlx::PgConnection;

use super::{
    db::MatchRef,
    detail,
    result::Side,
    stage::{self, Stage},
    types::{GameInput, MatchDetail},
    view::Viewer,
    withdrawals,
};
use crate::{
    shared::{ApiError, text::clean_text},
    tenants::TenantMember,
    tournaments::{StoredTournament, TournamentStatus},
};

/// Comentário de contestação ou nota do organizador.
const MAX_COMMENT: usize = 280;

/// 409 `tournament.not_in_progress`: jogador só age com o torneio em
/// andamento.
pub fn in_progress(tournament: &StoredTournament) -> Result<(), ApiError> {
    if tournament.status != TournamentStatus::InProgress {
        return Err(ApiError::conflict("tournament.not_in_progress"));
    }
    Ok(())
}

/// Os lados da partida pronta. 409 `match.already_reported` se ela já tem
/// resultado confirmado e `match.not_ready` se ainda não dá para jogar (lado
/// por definir, bye, reset que não precisou).
pub fn ready(m: &Match) -> Result<(ParticipantId, ParticipantId), ApiError> {
    match m.state {
        MatchState::Ready { a, b } => Ok((a, b)),
        MatchState::Completed { .. } => Err(ApiError::conflict("match.already_reported")),
        _ => Err(ApiError::conflict("match.not_ready")),
    }
}

/// O lado de quem age entre `sides`, se ele joga a partida.
pub fn side_of<R>(
    stage: &Stage,
    member: &TenantMember<R>,
    (a, b): (ParticipantId, ParticipantId),
) -> Option<Side> {
    let me = stage.participant_of(member.user.id)?;
    if me == a {
        Some(Side::A)
    } else if me == b {
        Some(Side::B)
    } else {
        None
    }
}

/// 422 `match.comment_invalid` fora de 1 a 280 caracteres numa linha.
pub fn comment(raw: &str) -> Result<String, ApiError> {
    clean_text(raw, MAX_COMMENT, "match.comment_invalid")
}

/// Slugs dos mapas do detalhe, para buscar os ids.
pub fn map_slugs(games: &[GameInput]) -> Vec<String> {
    games.iter().map(|g| g.map.clone()).collect()
}

pub fn at<'a, R>(
    member: &TenantMember<R>,
    tournament: &StoredTournament,
    stage: &Stage,
    key: &'a str,
) -> MatchRef<'a> {
    MatchRef {
        tenant: member.tenant,
        tournament: tournament.id,
        stage: stage.number,
        key,
    }
}

/// Depois de gravar: refaz a chave, dá os W.O. automáticos que a mudança
/// deixou prontos, encerra ou reabre o torneio conforme a chave e devolve a
/// partida `key` como ficou.
pub async fn respond<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    mut tournament: StoredTournament,
    key: &str,
) -> Result<MatchDetail, ApiError> {
    let stage = withdrawals::apply(conn, &tournament).await?;
    tournament.status = stage::sync_status(conn, &tournament, &stage).await?;
    let viewer = Viewer::new(member, &stage);
    let m = detail::find(&stage, key)?;
    detail::load(conn, &tournament, &stage, &viewer, m).await
}
