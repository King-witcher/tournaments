use serde::Serialize;
use utoipa::ToSchema;

use crate::{matches::NextMatch, teams::Team, tournaments::TournamentSummary};

/// Os torneios de quem vê, em todas as comunidades de que participa.
#[derive(Serialize, ToSchema)]
pub struct MyTournaments {
    /// Em que está inscrito: primeiro os que não encerraram (em andamento,
    /// check-in, inscrição, rascunho; dentro de cada um, pelo início), depois
    /// os 5 encerrados de início mais recente.
    pub registered: Vec<MyTournament>,
    /// Em que pode se inscrever agora pelo botão, pelo início (os sem data
    /// por último).
    pub open: Vec<OpenTournament>,
}

/// Um torneio em que quem vê pode se inscrever agora pelo botão: inscrição
/// aberta a todos os membros e com vaga.
#[derive(Serialize, ToSchema)]
pub struct OpenTournament {
    pub tenant_slug: String,
    pub tenant_name: String,
    /// O logo da comunidade, no CDN (o lugar da capa, no torneio sem ela).
    pub tenant_logo_url: Option<String>,
    pub tournament: TournamentSummary,
}

/// Um torneio em que quem vê está inscrito.
#[derive(Serialize, ToSchema)]
pub struct MyTournament {
    pub tenant_slug: String,
    pub tenant_name: String,
    /// O logo da comunidade, no CDN (o lugar da capa, no torneio sem ela).
    pub tenant_logo_url: Option<String>,
    pub tournament: TournamentSummary,
    pub situation: Situation,
    /// A fase de que a situação fala: a que ele joga ou em que terminou, ou a
    /// seguinte, que ele espera (`awaiting_stage`, `stage_check_in_pending`,
    /// `substitute`). `null` antes da chave.
    pub stage: Option<u16>,
    /// Colocação de quem foi eliminado, desistiu ou venceu, quando já se
    /// definiu (na classificação final do torneio).
    pub placement: Option<MyPlacement>,
    /// Só de quem ainda joga, com o torneio em andamento.
    pub next_match: Option<NextMatch>,
    /// TDM: o time de quem vê (`null` no duelo e sem time).
    pub team: Option<Team>,
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
    /// TDM: fora de todo time. Antes do início, com algum time já formado
    /// (inscreveu-se ou fez o check-in depois); depois, o torneio começou com
    /// ele de reserva, ou ele foi substituído.
    Reserve,
    /// Na chave e ainda vivo (no round robin e no suíço, até a fase terminar).
    Playing,
    /// As séries da fase acabaram e ele está num empate que espera a decisão
    /// da organização: a colocação é a melhor posição do grupo, e `tied`,
    /// quantos estão nele.
    TiePending,
    /// Terminou, com a colocação: eliminado, ou na tabela final do round robin
    /// e do suíço.
    Eliminated,
    /// Desistiu (marcado pela organização).
    Withdrawn,
    Champion,
    /// Entre as fases (a anterior terminou, a seguinte, `stage`, não começou):
    /// está entre os que entram nela.
    AwaitingStage,
    /// Entre as fases, com o check-in da seguinte (`stage`) aberto: falta
    /// confirmar a presença do time; sem ela, fica fora.
    StageCheckInPending,
    /// Entre as fases: abaixo do corte da seguinte (`stage`); entra se alguém
    /// de cima não confirmar presença ou desistir. Quem fica de fora ganha a
    /// colocação quando ela começa.
    Substitute,
}

/// Faixa de colocação: `rank` 5 com `tied` 4 é 5º–8º.
#[derive(Serialize, ToSchema)]
pub struct MyPlacement {
    pub rank: u16,
    pub tied: u16,
}
