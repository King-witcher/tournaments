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
    /// Fez check-in. Com check-in no torneio, só quem fez entra no seeding e
    /// na chave.
    pub checked_in: bool,
    /// Posição no seeding (1 = melhor); `null` sem seed. Depois de iniciar,
    /// 1..N entre os elegíveis.
    pub seed: Option<i32>,
    /// Saiu da comunidade (ou foi removido). Só fica inscrito assim em
    /// torneio iniciado, como histórico.
    pub left: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct AddParticipant {
    /// Membro do tenant a inscrever.
    pub user_id: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct SeedingOrder {
    /// Todos os elegíveis, cada um uma vez, do seed 1 em diante.
    pub user_ids: Vec<i64>,
}

/// `{tournament}` e `{user_id}` de
/// `/tenants/{slug}/tournaments/{tournament}/participants/{user_id}` (o slug
/// do tenant fica com `TenantMember`).
#[derive(Deserialize)]
pub struct ParticipantPath {
    pub tournament: String,
    pub user_id: i64,
}
