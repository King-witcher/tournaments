//! Tenants ("comunidades"): criar, listar os meus, ver um e transferir a
//! posse (o owner rebaixa a si mesmo a organizer e promove outro membro, numa
//! transação, dentro do limite de tenants por owner). Também é a base
//! de todo dado de tenant: `TenantId`, `Role`, o extractor `TenantMember` e
//! as regras de apelido, que os outros domínios usam nas rotas
//! `/tenants/{slug}/...`.

mod access;
mod db;
mod handlers;
mod nickname;
mod slug;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use access::{Organizer, Owner, Role, TenantId, TenantMember};
pub use db::{my_tenant, standing};
pub use nickname::{default_nickname, validate_nickname};
pub use types::MyTenant;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::create_tenant, handlers::list_my_tenants))
        .routes(routes!(handlers::get_tenant))
        .routes(routes!(handlers::transfer_ownership))
}
