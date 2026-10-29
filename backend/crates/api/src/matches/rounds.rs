//! As rodadas do suíço: quando todas as séries das rodadas já pareadas estão
//! decididas, o `engine` gera o pareamento da seguinte (`next_round`) e ele
//! fica gravado (`stage_pairings`). O replay da fase usa os gravados, então
//! corrigir um resultado antigo muda a tabela e nunca refaz uma rodada; a
//! correção que reabre uma série segura a rodada seguinte até ela ser
//! decidida de novo. A primeira rodada nasce no início (`first`); as outras,
//! no `settle` (`pair_next`), que refaz a fase e repete. Nos outros formatos,
//! nada acontece aqui.

use engine::{
    core::ParticipantId,
    formats::{self as ef, StageBracket, StageHistory},
};
use sqlx::PgConnection;

use super::{db, stage::Stage};
use crate::{
    shared::ApiError,
    tenants::TenantId,
    tournaments::{StoredTournament, TournamentId},
};

/// A fase nova com `seeded` (em ordem de seed) e, no suíço, a primeira rodada
/// pareada e gravada (`at`: tenant, torneio e fase). Erros: os de montar a
/// fase.
pub async fn first(
    conn: &mut PgConnection,
    at: (TenantId, TournamentId, i16),
    format: ef::StageFormat,
    seeded: &[ParticipantId],
) -> Result<StageBracket, ApiError> {
    let mut history = StageHistory::default();
    let mut bracket = StageBracket::new(format, seeded.to_vec())?;
    while let Some(pairing) = bracket.next_round() {
        db::insert_round(conn, at, number(history.rounds.len())?, &pairing).await?;
        history.rounds.push(pairing);
        bracket = StageBracket::replay(format, seeded.to_vec(), history.clone())?;
    }
    Ok(bracket)
}

/// Grava o pareamento da rodada seguinte do suíço de `stage`, se é hora (as
/// séries das rodadas pareadas decididas, rodada por parear e pelo menos 2
/// times que continuam). Devolve se gravou: quem chama refaz a fase, e as
/// partidas novas saem prontas no `changes` como as outras.
pub async fn pair_next(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<bool, ApiError> {
    let Some(pairing) = stage.bracket.next_round() else {
        return Ok(false);
    };
    let at = (tournament.tenant, tournament.id, stage.number);
    db::insert_round(conn, at, number(stage.rounds.len())?, &pairing).await?;
    Ok(true)
}

/// O número da rodada depois de `paired` rodadas.
fn number(paired: usize) -> Result<u16, ApiError> {
    u16::try_from(paired + 1).map_err(|_| ApiError::internal())
}
