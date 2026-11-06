use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use utoipa::ToSchema;

use super::{avatar::resolve_avatar, db};
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
    /// O avatar que as telas mostram: a foto enviada ou, sem ela, o do
    /// Discord.
    pub avatar_url: Option<String>,
    /// Só a foto enviada (`/me/avatar`); nula sem ela, mesmo com o avatar do
    /// Discord.
    pub photo_url: Option<String>,
    pub locale: String,
}

impl CurrentUser {
    /// A conta como o banco a guarda: a foto (`avatar_key`) e o avatar do
    /// Discord (`discord_avatar`) viram as duas URLs.
    pub fn from_row(
        id: i64,
        display_name: String,
        avatar_key: Option<String>,
        discord_avatar: Option<String>,
        locale: String,
    ) -> Self {
        let photo_url = resolve_avatar(avatar_key);
        Self {
            id,
            display_name,
            avatar_url: photo_url.clone().or(discord_avatar),
            photo_url,
            locale,
        }
    }

    /// Dono da sessão com este hash: o dono vem da memória e o usuário, do
    /// banco. Sessão vencida ou conta que não existe mais → `None`.
    pub(super) async fn of_session(
        state: &AppState,
        hash: &[u8; 32],
    ) -> Result<Option<Self>, ApiError> {
        let Some(user_id) = state.sessions.owner(hash).await? else {
            return Ok(None);
        };
        Ok(db::find_user(&state.db, user_id).await?)
    }

    /// O id do dono da sessão do cookie, sem ler o usuário:
    /// para quem lê o usuário na mesma query de outra coisa (`TenantMember`,
    /// uma ida ao banco a menos por requisição). Quem chama confere que a
    /// conta existe. 401 `auth.unauthenticated` sem sessão válida.
    pub async fn session_owner(parts: &Parts, state: &AppState) -> Result<i64, ApiError> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = state
            .auth
            .session_token(&jar)
            .ok_or(ApiError::unauthenticated())?;
        state
            .sessions
            .owner(&hash_token(token))
            .await?
            .ok_or(ApiError::unauthenticated())
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = state
            .auth
            .session_token(&jar)
            .ok_or(ApiError::unauthenticated())?;

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
        let Some(token) = state.auth.session_token(&jar) else {
            return Ok(None);
        };
        Self::of_session(state, &hash_token(token)).await
    }
}
