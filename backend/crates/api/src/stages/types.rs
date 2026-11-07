use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{teams::Team, tournaments::StageStatus};

/// Uma fase da 2 em diante: a configuração da entrada nela, onde ela está e
/// os candidatos (quem jogou a anterior).
#[derive(Serialize, ToSchema)]
pub struct StageEntry {
    /// A posição da fase ("Fase N"), de 2 em diante.
    pub number: u16,
    pub status: StageStatus,
    /// Só entra quem confirmar presença no check-in da fase.
    pub check_in: bool,
    /// Quantos da fase anterior entram; `null`: todos.
    pub advance: Option<u16>,
    pub started_at: Option<DateTime<Utc>>,
    pub check_in_opened_at: Option<DateTime<Utc>>,
    /// Quem jogou a fase anterior, na ordem da classificação dela (a de
    /// agora, antes de ela terminar); vazio antes de ela começar.
    pub candidates: Vec<StageCandidate>,
    /// A organização pode iniciar a fase agora (`start_blocker` nulo).
    pub can_start: bool,
    /// Por que a fase não pode começar agora: `stage.already_started`,
    /// `tournament.not_in_progress`, `stage.previous_not_finished` (séries por
    /// decidir ou empate do fim esperando a organização),
    /// `stage.check_in_not_open` (a fase pede check-in e ele não abriu),
    /// `stage.not_enough_entrants` (menos de 2 entrariam). `null`: pode.
    pub start_blocker: Option<String>,
}

/// Um candidato à fase: um time que jogou a anterior.
#[derive(Serialize, ToSchema)]
pub struct StageCandidate {
    pub team: Team,
    /// A posição na classificação da fase anterior (a de agora, antes de ela
    /// terminar).
    pub previous_rank: u16,
    /// Desistiu: fica fora.
    pub withdrawn: bool,
    /// Confirmou presença no check-in da fase.
    pub checked_in: bool,
    pub checked_in_at: Option<DateTime<Utc>>,
    /// Antes do início, o seed com que entraria agora (com o check-in aberto,
    /// só quem confirmou; antes de abrir, todos que não desistiram); depois,
    /// o gravado. `null`: suplente (abaixo do corte) ou fora.
    pub projected_seed: Option<u16>,
    /// O seed gravado no início: entrou na fase. `null` antes do início e para
    /// quem ficou de fora.
    pub seed: Option<u16>,
}

/// `{tournament}` e `{stage}` de
/// `/tenants/{slug}/tournaments/{tournament}/stages/{stage}`.
#[derive(Deserialize)]
pub struct StagePath {
    pub tournament: String,
    pub stage: i16,
}

/// `{tournament}`, `{stage}` e `{team}` de
/// `/tenants/{slug}/tournaments/{tournament}/stages/{stage}/teams/{team}/check-in`.
#[derive(Deserialize)]
pub struct StageTeamPath {
    pub tournament: String,
    pub stage: i16,
    pub team: i64,
}
