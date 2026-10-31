//! Peças comuns às ações sobre uma partida (`report`, `manage`): as
//! checagens com código de erro e a resposta depois de gravar.

use std::collections::HashMap;

use engine::core::{EngineError, MapId, Match, MatchState, ParticipantId};
use sqlx::PgConnection;

use super::{
    changes::Snapshot,
    db::MatchRef,
    detail,
    games::StoredGame,
    result::Side,
    settle,
    stage::Stage,
    types::{GameInput, MatchDetail},
    view::Viewer,
};
use crate::{
    maps,
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

/// Os lados da partida pronta. 409 `match.series_decided` se a série já está
/// decidida e `match.not_ready` se ainda não dá para jogar (lado por
/// definir, bye, reset que não precisou).
pub fn ready(m: &Match) -> Result<(ParticipantId, ParticipantId), ApiError> {
    match m.state {
        MatchState::Ready { a, b } => Ok((a, b)),
        MatchState::Completed { .. } => Err(ApiError::conflict("match.series_decided")),
        _ => Err(ApiError::conflict("match.not_ready")),
    }
}

/// O lado de quem age entre `sides`, se ele joga a partida (é jogador do time
/// de um dos lados; ver `Stage::participant_of`). Nos times, qualquer jogador
/// do time age pelo lado.
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

/// O id do mapa do envio, se veio, entre os ativos.
pub async fn map_ids(
    conn: &mut PgConnection,
    input: &GameInput,
) -> Result<HashMap<String, MapId>, ApiError> {
    let slugs: Vec<String> = input.map.iter().cloned().collect();
    Ok(maps::active_ids(conn, &slugs).await?)
}

/// O mapa `number` da partida, se já foi enviado. 404 `match.game_not_found`.
pub fn game<'a>(games: &[&'a StoredGame], number: i16) -> Result<&'a StoredGame, ApiError> {
    games
        .iter()
        .find(|g| g.number == number)
        .copied()
        .ok_or(ApiError::not_found("match.game_not_found"))
}

/// O índice (a partir de 0) do mapa `number`. 409 `match.game_out_of_order`
/// `{next}` para número abaixo de 1 (acima do próximo, o `engine` recusa).
pub fn game_index(number: i16, len: usize) -> Result<usize, ApiError> {
    usize::try_from(number)
        .ok()
        .and_then(|n| n.checked_sub(1))
        .ok_or_else(|| EngineError::GameOutOfOrder { next: len + 1 }.into())
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

/// Depois de gravar: fecha as séries decididas, anula o que ficou depois da
/// decisão, dá os W.O. automáticos, encerra ou reabre o torneio conforme a
/// chave, emite o que mudou desde `before` (a fase lida antes de gravar) e
/// devolve a partida `key` como ficou.
pub async fn respond<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    mut tournament: StoredTournament,
    before: &Stage,
    key: &str,
) -> Result<MatchDetail, ApiError> {
    let before = Snapshot::of(before);
    let actor = Some(member.user.id);
    let (stage, status) = settle::settle(conn, &tournament, actor, &before).await?;
    tournament.status = status;
    let viewer = Viewer::new(member, &stage);
    let m = detail::find(&stage, key)?;
    detail::load(conn, &tournament, &stage, &viewer, m).await
}
