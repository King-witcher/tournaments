//! Tenants ("comunidades"): criar, listar os meus, ver um, transferir a
//! posse (o owner rebaixa a si mesmo a organizer e promove outro membro, numa
//! transação, dentro do limite de tenants por owner) e excluir (`deletion`:
//! o owner agenda para 7 dias, a comunidade fica só leitura até lá, e o
//! agendador apaga na data). Também é a base de todo dado de tenant:
//! `TenantId`, `Role`, o extractor `TenantMember` (que recusa escrita na
//! comunidade com a exclusão agendada; `DuringDeletion` passa) e as regras de
//! apelido, que os outros domínios usam nas rotas `/tenants/{slug}/...`, e o
//! que a comunidade emite no outbox (`changed`, `access_lost`,
//! `organizing_changed`; `staff` são os destinatários dos avisos da
//! organização).

mod access;
mod changes;
mod db;
mod deletion;
mod handlers;
mod nickname;
mod slug;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use access::{DuringDeletion, Organizer, Owner, Role, TenantId, TenantMember};
pub use changes::{access_lost, changed, organizing_changed};
pub use db::{my_tenant, staff, standing};
pub use deletion::delete_due;
pub use nickname::{default_nickname, validate_nickname};
pub use types::MyTenant;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::create_tenant, handlers::list_my_tenants))
        .routes(routes!(handlers::get_tenant))
        .routes(routes!(handlers::transfer_ownership))
        .routes(routes!(
            deletion::schedule_tenant_deletion,
            deletion::cancel_tenant_deletion
        ))
}
