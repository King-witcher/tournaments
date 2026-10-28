use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use utoipa::ToSchema;

use super::{db, kv, session::SESSION_COOKIE};
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

impl CurrentUser {
    /// Dono da sessão com este hash: o dono vem do Redis e o usuário, do
    /// banco. Sessão vencida ou conta que não existe mais → `None`.
    pub(super) async fn of_session(
        state: &AppState,
        hash: &[u8; 32],
    ) -> Result<Option<Self>, ApiError> {
        let Some(user_id) = kv::session_user(&state.redis, hash).await? else {
            return Ok(None);
        };
        Ok(db::find_user(&state.db, user_id).await?)
    }
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

        Self::of_session(state, &hash_token(token))
            .await?
            .ok_or(ApiError::unauthenticated())
    }
}

/// `Option<CurrentUser>`: rota pública que muda a resposta para quem está
/// logado (sem sessão válida, `None`).
impl OptionalFromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let Some(cookie) = jar.get(SESSION_COOKIE) else {
            return Ok(None);
        };
        Self::of_session(state, &hash_token(cookie.value())).await
    }
}
