//! Mapas do jogo: tabela global, como os campeões, com a marcação do pool de
//! duelo. A lista alimenta o detalhe por mapa do resultado (`matches`) e o
//! pool padrão de cada torneio (`duel_pool`), de onde sai o veto. Domínio
//! pequeno: um arquivo só, com o SQL junto.

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
    /// Está no pool padrão de um torneio novo (`duel_pool`).
    pub default_pool: bool,
}

#[utoipa::path(
    get,
    path = "/maps",
    tag = "maps",
    responses(
        (status = OK, body = Vec<Map>, description = "Mapas ativos: os do pool de duelo primeiro, depois os outros, por nome; `default_pool` marca o pool padrão de um torneio novo"),
    )
)]
pub async fn list_maps(State(state): State<AppState>) -> Result<Json<Vec<Map>>, ApiError> {
    let maps = sqlx::query_as!(
        Map,
        r#"select slug, name, duel_pool,
                  (duel_pool and id in (select id from maps where active and duel_pool
                                        order by id limit $1)) as "default_pool!"
           from maps where active order by duel_pool desc, name"#,
        crate::tournaments::DEFAULT_POOL_SIZE
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

/// Os `n` primeiros mapas ativos do pool de duelo, na ordem da tabela: o pool
/// padrão de um torneio.
pub async fn duel_pool(conn: &mut PgConnection, n: i64) -> Result<Vec<MapId>, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        "select id from maps where active and duel_pool order by id limit $1",
        n
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(MapId)
    .collect())
}
