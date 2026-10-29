//! Tenants ("comunidades"): criar, listar os meus, ver um e transferir a
//! posse (o owner rebaixa a si mesmo a organizer e promove outro membro, numa
//! transação, dentro do limite de tenants por owner). Também é a base
//! de todo dado de tenant: `TenantId`, `Role`, o extractor `TenantMember` e
//! as regras de apelido, que os outros domínios usam nas rotas
//! `/tenants/{slug}/...`, e o que a comunidade emite no outbox (`changed`,
//! `access_lost`, `organizing_changed`; `staff` são os destinatários dos
//! avisos da organização).

mod access;
mod changes;
mod db;
mod handlers;
mod nickname;
mod slug;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use access::{Organizer, Owner, Role, TenantId, TenantMember};
pub use changes::{access_lost, changed, organizing_changed};
pub use db::{my_tenant, staff, standing};
pub use nickname::{default_nickname, validate_nickname};
pub use types::MyTenant;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::create_tenant, handlers::list_my_tenants))
        .routes(routes!(handlers::get_tenant))
        .routes(routes!(handlers::transfer_ownership))
}
