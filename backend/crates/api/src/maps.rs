//! Mapas do jogo: tabela global, como os campeões, com a marcação do pool de
//! duelo. A lista alimenta o detalhe por mapa do resultado (`matches`); o
//! veto vai ter um pool por torneio a partir dela. Domínio pequeno: um
//! arquivo só, com o SQL junto.

use std::collections::HashMap;

use axum::{Json, extract::State};
use engine::core::MapId;
use serde::Serialize;
use sqlx::PgConnection;
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

/// Ids dos mapas ativos entre `slugs`; o que não existe (ou saiu do jogo)
/// fica de fora.
pub async fn active_ids(
    conn: &mut PgConnection,
    slugs: &[String],
) -> Result<HashMap<String, MapId>, sqlx::Error> {
    let rows = sqlx::query!(
        "select slug, id from maps where active and slug = any($1)",
        slugs
    )
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|r| (r.slug, MapId(r.id))).collect())
}
