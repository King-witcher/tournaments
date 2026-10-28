//! `Json` e `Path` do axum com a rejeição convertida em `ApiError`, para que
//! corpo ou caminho inválido também saia como `{"error": "<código>"}` e não
//! como texto puro.

use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};

use super::ApiError;

/// Corpo JSON. Inválido → status do axum (400, 415 ou 422) com `request.invalid_body`.
pub struct JsonBody<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for JsonBody<T> {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        axum::Json::<T>::from_request(req, state)
            .await
            .map(|axum::Json(value)| Self(value))
            .map_err(|rej| ApiError::new(rej.status(), "request.invalid_body"))
    }
}

/// Campo de PATCH em que `null` quer dizer limpar: com `#[serde(default,
/// deserialize_with = "nullable")]` num `Option<Option<T>>`, ausente fica
/// `None` (não muda), `null` vira `Some(None)` e um valor, `Some(Some(v))`.
pub fn nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Parâmetros do caminho. Inválido (ex.: id que não é número) → 400 `request.invalid_path`.
pub struct PathParams<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned + Send> FromRequestParts<S> for PathParams<T> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        axum::extract::Path::<T>::from_request_parts(parts, state)
            .await
            .map(|axum::extract::Path(value)| Self(value))
            .map_err(|rej| ApiError::new(rej.status(), "request.invalid_path"))
    }
}
