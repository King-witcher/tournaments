use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{AppState, auth::SESSION_COOKIE, auth::hash_token, error::ApiError};

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
        let hash = hash_token(token);

        sqlx::query_as!(
            CurrentUser,
            r#"select u.id, u.display_name, u.avatar_url, u.locale
               from sessions s
               join users u on u.id = s.user_id
               where s.id = $1 and s.expires_at > now()"#,
            &hash[..]
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::unauthenticated())
    }
}
