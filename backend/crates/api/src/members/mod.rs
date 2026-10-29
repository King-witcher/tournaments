//! Membros de um tenant: listar, ver o perfil de um (com os torneios da
//! comunidade em que se inscreveu, `history`), o owner trocar papéis, cada
//! um editar o próprio apelido, sair da comunidade e remover um membro
//! (`departure`). Sair ou ser removido deixa o membro inativo, com o
//! histórico; mexe nos torneios (`lifecycle`) e nos convites (`invites`) dele,
//! e o perfil junta `tournaments`, `participants`, `matches` e a situação de
//! `my_tournaments`, então este domínio fica acima deles. A exclusão de conta
//! vencida (`account_deletion`, chamada pelo agendador) tira a conta de todas
//! as comunidades do mesmo jeito e pede a `auth` que a anonimize.

mod account_deletion;
mod db;
mod departure;
mod handlers;
mod history;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use account_deletion::delete_due_accounts;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            handlers::update_my_membership,
            handlers::leave_tenant
        ))
        .routes(routes!(handlers::list_members))
        .routes(routes!(handlers::get_member, handlers::remove_member))
        .routes(routes!(handlers::set_member_role))
}
