use std::collections::BTreeMap;

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
    /// Números que a mensagem traduzida mostra, quando o código tem
    /// (ex.: `champions.pool_exhausted` → `{"required": 33, "available": 17}`).
    /// Alguns códigos trazem também um texto (ex.: `locale`, o idioma da
    /// tradução com erro).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<&'static str, ParamValue>,
}

/// Valor de um parâmetro do erro: número ou texto curto.
#[derive(Serialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum ParamValue {
    Number(i64),
    Text(&'static str),
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub params: Vec<(&'static str, i64)>,
    /// Parâmetros de texto (ver `ParamValue::Text`).
    pub text_params: Vec<(&'static str, &'static str)>,
}

impl ApiError {
    pub const fn new(status: StatusCode, code: &'static str) -> Self {
        Self {
            status,
            code,
            params: Vec::new(),
            text_params: Vec::new(),
        }
    }

    /// Acrescenta números à resposta, para o front interpolar na tradução.
    pub fn with_params<const N: usize>(mut self, params: [(&'static str, i64); N]) -> Self {
        self.params.extend(params);
        self
    }

    /// Acrescenta um parâmetro de texto à resposta.
    pub fn with_text_param(mut self, key: &'static str, value: &'static str) -> Self {
        self.text_params.push((key, value));
        self
    }

    pub const fn unauthenticated() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated")
    }

    pub const fn forbidden(code: &'static str) -> Self {
        Self::new(StatusCode::FORBIDDEN, code)
    }

    pub const fn not_found(code: &'static str) -> Self {
        Self::new(StatusCode::NOT_FOUND, code)
    }

    pub const fn conflict(code: &'static str) -> Self {
        Self::new(StatusCode::CONFLICT, code)
    }

    /// Recurso que existiu mas não vale mais (convite expirado ou revogado).
    pub const fn gone(code: &'static str) -> Self {
        Self::new(StatusCode::GONE, code)
    }

    /// Entrada que chegou bem formada mas não passa na validação.
    pub const fn invalid(code: &'static str) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code)
    }

    pub const fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: self.code,
            params: self
                .params
                .into_iter()
                .map(|(k, v)| (k, ParamValue::Number(v)))
                .chain(
                    self.text_params
                        .into_iter()
                        .map(|(k, v)| (k, ParamValue::Text(v))),
                )
                .collect(),
        };
        (self.status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "erro de banco");
        Self::internal()
    }
}

impl From<redis::RedisError> for ApiError {
    fn from(err: redis::RedisError) -> Self {
        tracing::error!(error = %err, "erro do Redis");
        Self::internal()
    }
}
