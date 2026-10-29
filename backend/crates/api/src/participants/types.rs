use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Inscrito num torneio.
#[derive(Serialize, ToSchema)]
pub struct Participant {
    pub user_id: i64,
    /// Apelido no tenant.
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub registered_at: DateTime<Utc>,
    /// Fez check-in. Com check-in no torneio, só o time em que todos fizeram
    /// entra no seeding e na chave.
    pub checked_in: bool,
    /// O time do inscrito (no duelo, o time de um jogador, criado com a
    /// inscrição); `null` sem time.
    pub team_id: Option<i64>,
    /// Posição do time dele no seeding (1 = melhor); `null` sem seed. Depois
    /// de iniciar, 1..N entre os times da chave.
    pub seed: Option<i32>,
    /// O time dele desistiu do torneio em andamento: as partidas dele viram
    /// W.O.
    pub withdrawn: bool,
    /// Saiu da comunidade (ou foi removido). Só fica inscrito assim em
    /// torneio iniciado, como histórico; no em andamento, o time dele também
    /// desistiu, se ninguém mais dele ficou.
    pub left: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct AddParticipant {
    /// Membro do tenant a inscrever.
    pub user_id: i64,
}

/// `{tournament}` e `{user_id}` de
/// `/tenants/{slug}/tournaments/{tournament}/participants/{user_id}` (o slug
/// do tenant fica com `TenantMember`).
#[derive(Deserialize)]
pub struct ParticipantPath {
    pub tournament: String,
    pub user_id: i64,
}
