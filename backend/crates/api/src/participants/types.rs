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
