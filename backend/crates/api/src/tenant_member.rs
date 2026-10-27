use std::marker::PhantomData;

use axum::{
    extract::{FromRequestParts, RawPathParams},
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AppState, current_user::CurrentUser, error::ApiError};

/// Id interno de um tenant. Fora da criação, só sai do banco pelo extractor
/// `TenantMember`, então quem recebe um `TenantId` já passou pela checagem de
/// participação. Toda query de dado de tenant filtra por ele.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TenantId(i64);

impl TenantId {
    pub(crate) const fn new(id: i64) -> Self {
        Self(id)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

/// Papel no tenant. A ordem das variantes é a hierarquia (`Player < Owner`).
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Player,
    Organizer,
    Owner,
}

impl Role {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::Organizer => "organizer",
            Self::Owner => "owner",
        }
    }

    /// Lê a coluna `tenant_members.role` (garantida pelo check da tabela).
    pub(crate) fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "player" => Ok(Self::Player),
            "organizer" => Ok(Self::Organizer),
            "owner" => Ok(Self::Owner),
            other => {
                tracing::error!(role = other, "papel desconhecido no banco");
                Err(ApiError::internal())
            }
        }
    }
}

/// Papel mínimo exigido por `TenantMember<R>`.
pub trait MinRole: Send + Sync {
    const ROLE: Role;
}

/// Qualquer membro.
pub struct Player;
/// Organizer ou owner.
pub struct Organizer;
/// Só o owner.
pub struct Owner;

impl MinRole for Player {
    const ROLE: Role = Role::Player;
}
impl MinRole for Organizer {
    const ROLE: Role = Role::Organizer;
}
impl MinRole for Owner {
    const ROLE: Role = Role::Owner;
}

/// Usuário logado que é membro do tenant do `{slug}` da rota, com papel de
/// pelo menos `R`. Use como parâmetro de handler em rotas `/tenants/{slug}/...`:
/// - sem sessão: 401 `auth.unauthenticated`;
/// - tenant inexistente ou usuário de fora: 404 `tenant.not_found` (não revela
///   se o tenant existe);
/// - papel abaixo de `R`: 403 `tenant.forbidden`.
pub struct TenantMember<R = Player> {
    pub tenant: TenantId,
    pub slug: String,
    pub name: String,
    pub user: CurrentUser,
    pub role: Role,
    pub nickname: String,
    min_role: PhantomData<R>,
}

impl<R: MinRole> FromRequestParts<AppState> for TenantMember<R> {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = CurrentUser::from_request_parts(parts, state).await?;
        let params = RawPathParams::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::internal())?;
        let Some((_, slug)) = params.iter().find(|(key, _)| *key == "slug") else {
            tracing::error!("TenantMember usado em rota sem {{slug}}");
            return Err(ApiError::internal());
        };

        let row = sqlx::query!(
            r#"select t.id, t.slug, t.name, m.role, m.nickname
               from tenants t
               join tenant_members m on m.tenant_id = t.id
               where t.slug = $1 and m.user_id = $2"#,
            slug,
            user.id
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::not_found("tenant.not_found"))?;

        let role = Role::from_db(&row.role)?;
        if role < R::ROLE {
            return Err(ApiError::forbidden("tenant.forbidden"));
        }
        Ok(Self {
            tenant: TenantId::new(row.id),
            slug: row.slug,
            name: row.name,
            user,
            role,
            nickname: row.nickname,
            min_role: PhantomData,
        })
    }
}
