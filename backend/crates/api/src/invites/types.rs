use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CreatedInvite {
    pub id: i64,
    /// Token do link `/invite/{token}`. Só aparece nesta resposta: o banco guarda o hash.
    #[schema(example = "7Fq2xKpR9mWbTnVc3hJdYe")]
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Convite ativo (nem revogado nem expirado).
#[derive(Serialize, ToSchema)]
pub struct Invite {
    pub id: i64,
    /// Apelido, neste tenant, de quem criou.
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Quantas pessoas entraram por este link.
    pub uses: i64,
}

/// `{invite_id}` de `/tenants/{slug}/invites/{invite_id}` (o slug fica com `TenantMember`).
#[derive(Deserialize)]
pub struct InvitePath {
    pub invite_id: i64,
}

/// O que quem recebeu o link vê antes de aceitar.
#[derive(Serialize, ToSchema)]
pub struct InvitePreview {
    pub tenant_name: String,
    pub expires_at: DateTime<Utc>,
}
