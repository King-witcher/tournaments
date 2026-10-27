use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use utoipa::ToSchema;

use super::{db, session::SESSION_COOKIE};
use crate::{
    AppState,
    shared::{ApiError, token::hash_token},
};

/// Usuário logado. Use como parâmetro de handler para exigir login:
/// sem sessão válida a rota responde 401 `auth.unauthenticated`.
#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct CurrentUser {
    pub id: i64,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub locale: String,
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get(SESSION_COOKIE)
            .ok_or(ApiError::unauthenticated())?
            .value();

        db::user_by_session(&state.db, &hash_token(token))
            .await?
            .ok_or(ApiError::unauthenticated())
    }
}
