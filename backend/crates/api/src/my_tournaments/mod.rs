//! `GET /me/tournaments`: os torneios em que quem vê está inscrito, de todas
//! as comunidades, com a situação dele (inscrito, check-in, na chave,
//! eliminado, campeão) e a próxima partida, e os que estão abertos à
//! inscrição dele; `GET /tenants/{slug}/tournaments/{tournament}/mine`: a
//! mesma entrada de um torneio só (a página do torneio). Junta `tenants` (as
//! participações), `tournaments` (a lista que ele vê, de todas as comunidades
//! numa query), `participants` (quem pode se inscrever), `teams` (o time dele)
//! e `matches` (o lugar na chave), então fica acima deles. A entrada sai de
//! `entry`, com as idas ao banco de todos os torneios juntas; a mesma entrada
//! (situação, colocação e time de um inscrito) serve ao perfil do membro
//! (`members`).

mod entry;
mod handlers;
mod situation;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use entry::{Registered, entries};
pub use types::{MyPlacement, Situation};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handlers::my_tournaments))
        .routes(routes!(handlers::my_tournament))
}
