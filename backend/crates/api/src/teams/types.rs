use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Um time do torneio: quem ocupa um lado da chave. No duelo, o time de um
/// jogador, sem nome (a tela mostra o apelido dele).
#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct Team {
    pub id: i64,
    /// Nome do time; `null` no duelo e nos times sem nome (a tela mostra o
    /// apelido do único jogador, ou "Time de {capitão}").
    pub name: Option<String>,
    /// Posição no seeding (1 = melhor); `null` sem seed. Depois de iniciar,
    /// 1..N entre os times da chave.
    pub seed: Option<i32>,
    /// `user_id` do capitão, um dos `members` (no duelo, o único).
    pub captain: i64,
    /// Desistiu do torneio em andamento: as partidas prontas dele viram W.O.
    /// para o adversário.
    pub withdrawn: bool,
    /// Os jogadores: o capitão primeiro, depois na ordem de inscrição.
    pub members: Vec<TeamMember>,
}

/// Um jogador de um time.
#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct TeamMember {
    pub user_id: i64,
    /// Apelido no tenant; vazio quando a conta foi excluída (a tela mostra
    /// "Conta excluída").
    pub nickname: String,
    pub avatar_url: Option<String>,
    /// Saiu da comunidade (ou foi removido); continua no time como histórico.
    /// O time sem nenhum jogador na comunidade está fora do torneio.
    pub left: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct SeedingOrder {
    /// Todos os times elegíveis, cada um uma vez, do seed 1 em diante.
    pub team_ids: Vec<i64>,
}

/// `{tournament}` e `{team}` de
/// `/tenants/{slug}/tournaments/{tournament}/teams/{team}` (o slug do tenant
/// fica com `TenantMember`).
#[derive(Deserialize)]
pub struct TeamPath {
    pub tournament: String,
    /// Id do time.
    pub team: i64,
}
