//! `GET /me/tournaments`: os torneios em que quem vê está inscrito, de todas
//! as comunidades, com a situação dele (inscrito, check-in, na chave,
//! eliminado, campeão) e a próxima partida. Junta `tenants` (as
//! participações), `tournaments` (a lista que ele vê) e `matches` (o lugar
//! na chave), então fica acima deles.

mod handlers;
mod types;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(handlers::my_tournaments))
}
