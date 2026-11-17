//! Membros de um tenant: listar, ver um (o cabeçalho da página dele), o
//! owner trocar papéis, cada um editar o próprio apelido, sair da comunidade
//! e remover um membro (`departure`). Sair ou ser removido deixa o membro
//! inativo, com o histórico; mexe nos torneios dele (`lifecycle`). As abas da
//! página de um membro ficam em `record`: os torneios da comunidade em que se
//! inscreveu (`history`, que junta `tournaments`, `participants`, `matches`
//! e a situação de `my_tournaments`), a carreira e o confronto direto
//! (`career`, montados das séries e mapas que `db::record` lê direto no
//! banco) e os mapas paginados, com o efeito no rating (lê `rating_history`
//! depois de `ratings::fresh`). Por isso o domínio fica acima de todos esses.
//! A exclusão de conta vencida (`account_deletion`, chamada pelo agendador)
//! tira a conta de todas as comunidades do mesmo jeito e pede a `auth` que a
//! anonimize.

mod account_deletion;
mod career;
mod db;
mod departure;
mod handlers;
mod history;
mod record;
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
        .routes(routes!(record::list_member_tournaments))
        .routes(routes!(record::get_member_career))
        .routes(routes!(record::list_member_games))
        .routes(routes!(record::list_member_opponents))
}
