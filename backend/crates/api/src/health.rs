use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(health))
}

#[derive(Serialize, ToSchema)]
pub struct Health {
    pub database: ServiceStatus,
}

#[derive(Serialize, ToSchema, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    Ok,
    Unavailable,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "infra",
    responses(
        (status = OK, body = Health, description = "API e banco respondendo"),
        (status = SERVICE_UNAVAILABLE, body = Health, description = "Banco indisponível"),
    )
)]
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    match sqlx::query("select 1").execute(&state.db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(Health {
                database: ServiceStatus::Ok,
            }),
        ),
        Err(err) => {
            tracing::error!(error = %err, "health check: banco indisponível");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Health {
                    database: ServiceStatus::Unavailable,
                }),
            )
        }
    }
}
