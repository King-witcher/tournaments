//! O que cada rota da fase confere e a vista dela (`StageEntry`).

use engine::stages::StageSetup;

use super::types::{StageCandidate, StageEntry};
use crate::{
    matches::NextStage,
    shared::ApiError,
    tournaments::{StageProgress, StoredTournament, TournamentStatus},
};

/// Mínimo de times para a fase começar.
const MIN_ENTRANTS: usize = 2;

/// A fase `number` de `tournament`, da 2 em diante. 404 `stage.not_found`
/// para a 1 (que nasce com o torneio) e para a que não existe.
pub fn stage_of(
    tournament: &StoredTournament,
    number: i16,
) -> Result<(&StageSetup, StageProgress), ApiError> {
    const NOT_FOUND: ApiError = ApiError::not_found("stage.not_found");
    if number < 2 {
        return Err(NOT_FOUND);
    }
    tournament.stage(number).ok_or(NOT_FOUND)
}

/// A fase anterior a `number` terminou: todas as séries decididas e nenhum
/// empate do fim esperando a organização.
fn previous_finished(tournament: &StoredTournament, number: i16, next: &NextStage) -> bool {
    tournament.stage_started(number - 1) && next.previous_finished
}

/// Por que a fase `number` não pode começar agora (o código do erro), ou
/// `None`.
pub fn blocker(
    tournament: &StoredTournament,
    number: i16,
    next: &NextStage,
) -> Option<&'static str> {
    let (setup, progress) = tournament.stage(number)?;
    if tournament.stage_started(number) {
        return Some("stage.already_started");
    }
    if tournament.status != TournamentStatus::InProgress {
        return Some("tournament.not_in_progress");
    }
    if !previous_finished(tournament, number, next) {
        return Some("stage.previous_not_finished");
    }
    if setup.check_in && progress.check_in_opened_at.is_none() {
        return Some("stage.check_in_not_open");
    }
    if next.entrants.len() < MIN_ENTRANTS {
        return Some("stage.not_enough_entrants");
    }
    None
}

/// Iniciar a fase `number`: o erro do que impede (409; `stage.not_enough_entrants`
/// com `{min, got}`).
pub fn check_start(
    tournament: &StoredTournament,
    number: i16,
    next: &NextStage,
) -> Result<(), ApiError> {
    match blocker(tournament, number, next) {
        None => Ok(()),
        Some("stage.not_enough_entrants") => Err(ApiError::conflict("stage.not_enough_entrants")
            .with_params([
                ("min", MIN_ENTRANTS as i64),
                ("got", next.entrants.len() as i64),
            ])),
        Some(code) => Err(ApiError::conflict(code)),
    }
}

/// Abrir o check-in da fase `number`: 409 `stage.check_in_disabled` (a fase
/// não pede), `stage.already_started`, `tournament.not_in_progress`,
/// `stage.previous_not_finished` e `stage.check_in_already_open`.
pub fn check_open(
    tournament: &StoredTournament,
    number: i16,
    next: &NextStage,
) -> Result<(), ApiError> {
    let (setup, progress) = stage_of(tournament, number)?;
    let code = if !setup.check_in {
        "stage.check_in_disabled"
    } else if tournament.stage_started(number) {
        "stage.already_started"
    } else if tournament.status != TournamentStatus::InProgress {
        "tournament.not_in_progress"
    } else if !previous_finished(tournament, number, next) {
        "stage.previous_not_finished"
    } else if progress.check_in_opened_at.is_some() {
        "stage.check_in_already_open"
    } else {
        return Ok(());
    };
    Err(ApiError::conflict(code))
}

/// Confirmar ou desfazer a presença na fase `number`: com o check-in dela
/// aberto e a fase por começar (409 `stage.check_in_not_open`).
pub fn check_in_open(tournament: &StoredTournament, number: i16) -> Result<(), ApiError> {
    let (_, progress) = stage_of(tournament, number)?;
    if progress.check_in_opened_at.is_none()
        || tournament.stage_started(number)
        || tournament.status != TournamentStatus::InProgress
    {
        return Err(ApiError::conflict("stage.check_in_not_open"));
    }
    Ok(())
}

/// A vista da fase `number`, com os candidatos de `next`. 404
/// `stage.not_found` como em `stage_of`.
pub fn view(
    tournament: &StoredTournament,
    number: i16,
    next: NextStage,
) -> Result<StageEntry, ApiError> {
    let blocker = blocker(tournament, number, &next);
    let (setup, progress) = stage_of(tournament, number)?;
    let candidates = next
        .candidates
        .into_iter()
        .map(|c| {
            let line = c.team.in_stage(number);
            StageCandidate {
                withdrawn: c.team.withdrawn,
                checked_in: line.is_some_and(|l| l.checked_in_at.is_some()),
                checked_in_at: line.and_then(|l| l.checked_in_at),
                seed: line
                    .and_then(|l| l.seed)
                    .and_then(|s| u16::try_from(s).ok()),
                projected_seed: c.projected_seed,
                previous_rank: c.previous_rank,
                team: c.team.view(),
            }
        })
        .collect();
    Ok(StageEntry {
        number: u16::try_from(number).unwrap_or(0),
        status: tournament.stage_status(number),
        check_in: setup.check_in,
        advance: setup.advance,
        started_at: progress.started_at,
        check_in_opened_at: progress.check_in_opened_at,
        candidates,
        can_start: blocker.is_none(),
        start_blocker: blocker.map(str::to_owned),
    })
}
