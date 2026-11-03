//! `GET /me/tournaments`: os torneios em que quem vê está inscrito, de todas
//! as comunidades, com a situação dele (inscrito, check-in, na chave,
//! eliminado, campeão) e a próxima partida, e os que estão abertos à
//! inscrição dele. Junta `tenants` (as participações), `tournaments` (a lista
//! que ele vê), `participants` (quem pode se inscrever) e `matches` (o lugar
//! na chave), então fica acima deles. A situação de um inscrito
//! (`situation`) também serve ao perfil do membro (`members`).

mod handlers;
mod situation;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub use handlers::{before_bracket, team_of};
pub use types::{MyPlacement, Situation};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(handlers::my_tournaments))
}
