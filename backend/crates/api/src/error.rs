use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

/// Corpo de erro da API: só um código estável, o texto fica no front.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    #[schema(example = "auth.unauthenticated")]
    pub error: &'static str,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
}

impl ApiError {
    pub const fn new(status: StatusCode, code: &'static str) -> Self {
        Self { status, code }
    }

    pub const fn unauthenticated() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated")
    }

    pub const fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorBody { error: self.code })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "erro de banco");
        Self::internal()
    }
}
