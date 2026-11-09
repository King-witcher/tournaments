use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{Role, TenantMember};
use crate::shared::public_url;

#[derive(Deserialize, ToSchema)]
pub struct CreateTenant {
    /// Nome exibido, 1 a 64 caracteres.
    #[schema(example = "Quake BR")]
    pub name: String,
    /// Identificador na URL: a-z, 0-9 e hífen, 3 a 32 caracteres. O owner pode
    /// trocar depois (`PATCH /tenants/{slug}`).
    #[schema(example = "quake-br")]
    pub slug: String,
}

/// O que o owner edita na comunidade.
#[derive(Deserialize, ToSchema)]
pub struct UpdateTenant {
    /// Nome exibido, 1 a 64 caracteres. Ausente: não muda.
    #[schema(example = "Quake BR")]
    pub name: Option<String>,
    /// Descrição em texto simples, até 2000 caracteres (as quebras de linha
    /// ficam). Vazia: a comunidade fica sem descrição. Ausente: não muda.
    pub description: Option<String>,
    /// O endereço novo, com a validação da criação. O antigo fica livre na
    /// hora, sem redirecionamento: links salvos e mensagens já enviadas com
    /// ele param de funcionar. Links de convite continuam valendo (usam o
    /// token). Ausente ou igual ao atual: não muda.
    #[schema(example = "quake-brasil")]
    pub slug: Option<String>,
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
    /// A descrição, em texto simples. Nulo: sem descrição.
    pub description: Option<String>,
    pub role: Role,
    /// Apelido do usuário neste tenant.
    pub nickname: String,
    /// Quando a comunidade vai ser excluída, com tudo o que tem dentro. Até
    /// lá ela é só leitura: toda escrita responde 409
    /// `tenant.deletion_scheduled`, menos cancelar a exclusão (o owner) e
    /// sair. Nulo: nenhuma exclusão agendada.
    pub deletion_scheduled_for: Option<DateTime<Utc>>,
    /// O logo (quadrado), no CDN. Nulo: sem logo, ou a API sem o bucket
    /// público.
    pub logo_url: Option<String>,
    /// A capa (2:1), no CDN. Nulo: sem capa, ou a API sem o bucket público.
    pub cover_url: Option<String>,
}

impl<R> From<TenantMember<R>> for MyTenant {
    fn from(m: TenantMember<R>) -> Self {
        Self {
            slug: m.slug,
            name: m.name,
            description: m.description,
            role: m.role,
            nickname: m.nickname,
            deletion_scheduled_for: m.deletion_scheduled_for,
            logo_url: m.logo_key.as_deref().and_then(public_url),
            cover_url: m.cover_key.as_deref().and_then(public_url),
        }
    }
}
