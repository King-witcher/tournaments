use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::tenants::Role;

#[derive(Serialize, ToSchema)]
pub struct Member {
    pub user_id: i64,
    pub nickname: String,
    /// Nome global do usuário (do Discord).
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateMyMembership {
    /// 1 a 32 caracteres; espaços nas pontas são removidos.
    pub nickname: String,
}

/// Papéis que o owner pode dar. A posse do tenant não muda por aqui.
#[derive(Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AssignableRole {
    Organizer,
    Player,
}

impl From<AssignableRole> for Role {
    fn from(role: AssignableRole) -> Self {
        match role {
            AssignableRole::Organizer => Self::Organizer,
            AssignableRole::Player => Self::Player,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct SetRole {
    pub role: AssignableRole,
}

/// `{user_id}` de `/tenants/{slug}/members/{user_id}/...` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct MemberPath {
    pub user_id: i64,
}
