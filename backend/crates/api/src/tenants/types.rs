use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{Role, TenantMember};

#[derive(Deserialize, ToSchema)]
pub struct CreateTenant {
    /// Nome exibido, 1 a 64 caracteres.
    #[schema(example = "Quake BR")]
    pub name: String,
    /// Identificador na URL: a-z, 0-9 e hífen, 3 a 32 caracteres. Não muda depois.
    #[schema(example = "quake-br")]
    pub slug: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TransferOwnership {
    /// Membro ativo que vira owner.
    pub user_id: i64,
    /// O slug do tenant, digitado para confirmar.
    #[schema(example = "quake-br")]
    pub confirm_slug: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ScheduleTenantDeletion {
    /// O slug do tenant, digitado para confirmar.
    #[schema(example = "quake-br")]
    pub confirm_slug: String,
}

/// Um tenant do ponto de vista do usuário logado.
#[derive(Serialize, ToSchema)]
pub struct MyTenant {
    pub slug: String,
    pub name: String,
    pub role: Role,
    /// Apelido do usuário neste tenant.
    pub nickname: String,
    /// Quando a comunidade vai ser excluída, com tudo o que tem dentro. Até
    /// lá ela é só leitura: toda escrita responde 409
    /// `tenant.deletion_scheduled`, menos cancelar a exclusão (o owner) e
    /// sair. Nulo: nenhuma exclusão agendada.
    pub deletion_scheduled_for: Option<DateTime<Utc>>,
}

impl<R> From<TenantMember<R>> for MyTenant {
    fn from(m: TenantMember<R>) -> Self {
        Self {
            slug: m.slug,
            name: m.name,
            role: m.role,
            nickname: m.nickname,
            deletion_scheduled_for: m.deletion_scheduled_for,
        }
    }
}
