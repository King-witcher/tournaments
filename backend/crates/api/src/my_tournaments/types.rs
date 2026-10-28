use serde::Serialize;
use utoipa::ToSchema;

use crate::{matches::NextMatch, tournaments::TournamentSummary};

/// Um torneio em que quem vê está inscrito.
#[derive(Serialize, ToSchema)]
pub struct MyTournament {
    pub tenant_slug: String,
    pub tenant_name: String,
    pub tournament: TournamentSummary,
    pub situation: Situation,
    /// Colocação de quem foi eliminado, desistiu ou venceu, quando já se
    /// definiu.
    pub placement: Option<MyPlacement>,
    /// Só de quem ainda joga, com o torneio em andamento.
    pub next_match: Option<NextMatch>,
}

/// A situação de um inscrito no torneio: quem vê, em `/me/tournaments`; o
/// membro, no perfil dele na comunidade.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Situation {
    /// Inscrito, antes do check-in (ou num torneio sem check-in).
    Registered,
    /// Check-in aberto e não feito: sem ele, fica fora da chave.
    CheckInPending,
    CheckedIn,
    /// O torneio começou sem ele na chave (não fez check-in).
    MissedCheckIn,
    /// Na chave e ainda vivo.
    Playing,
    Eliminated,
    /// Desistiu (marcado pela organização).
    Withdrawn,
    Champion,
}

/// Faixa de colocação: `rank` 5 com `tied` 4 é 5º–8º.
#[derive(Serialize, ToSchema)]
pub struct MyPlacement {
    pub rank: u16,
    pub tied: u16,
}
