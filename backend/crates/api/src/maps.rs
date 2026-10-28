//! Mapas do jogo: tabela global, como os campeões, com a marcação do pool de
//! duelo. A lista alimenta o detalhe por mapa do resultado (`matches`); o
//! veto vai ter um pool por torneio a partir dela. Domínio pequeno: um
//! arquivo só, com o SQL junto.

use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, shared::ApiError};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_maps))
}

#[derive(Serialize, ToSchema)]
pub struct Map {
    #[schema(example = "blood-covenant")]
    pub slug: String,
    /// Nome próprio, igual em todos os idiomas.
    #[schema(example = "Blood Covenant")]
    pub name: String,
    /// Faz parte do pool de duelo.
    pub duel_pool: bool,
}

#[utoipa::path(
    get,
    path = "/maps",
    tag = "maps",
    responses(
        (status = OK, body = Vec<Map>, description = "Mapas ativos: os do pool de duelo primeiro, depois os outros, por nome"),
    )
)]
pub async fn list_maps(State(state): State<AppState>) -> Result<Json<Vec<Map>>, ApiError> {
    let maps = sqlx::query_as!(
        Map,
        "select slug, name, duel_pool from maps where active order by duel_pool desc, name"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(maps))
}
