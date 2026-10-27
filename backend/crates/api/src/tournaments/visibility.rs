//! Quem vê um torneio. Organizer e owner veem todos; os outros membros não
//! veem rascunho nem torneio privado em que não estão inscritos, e para eles
//! a API responde 404, como se o torneio não existisse.

use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;

use super::{
    db::{self, StoredTournament},
    status::TournamentStatus,
};
use crate::{
    shared::ApiError,
    tenants::{Role, TenantMember},
};

const NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Todos os membros do tenant veem e podem se inscrever.
    #[default]
    Public,
    /// Só inscritos, organizers e owner veem. Entra-se por link de convite
    /// ou pelas mãos do organizador.
    Private,
}

impl Visibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }

    /// Lê `tournaments.visibility` (garantida pelo check da tabela).
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            other => {
                tracing::error!(visibility = other, "visibilidade desconhecida no banco");
                Err(ApiError::internal())
            }
        }
    }
}

/// A mesma regra está no SQL de `db::list`.
pub const fn can_see(
    status: TournamentStatus,
    visibility: Visibility,
    role: Role,
    registered: bool,
) -> bool {
    matches!(role, Role::Organizer | Role::Owner)
        || (!matches!(status, TournamentStatus::Draft)
            && (matches!(visibility, Visibility::Public) || registered))
}

/// O torneio `slug` como `member` o vê. 404 `tournament.not_found` se não
/// existe ou se ele não pode ver.
pub async fn find_visible<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let found = db::find(conn, member.tenant, slug, Some(member.user.id)).await?;
    visible(found, member.role)
}

/// Como `find_visible`, travando a linha do torneio até o fim da transação.
pub async fn find_visible_for_update<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<StoredTournament, ApiError> {
    let found = db::find_for_update(conn, member.tenant, slug, Some(member.user.id)).await?;
    visible(found, member.role)
}

fn visible(found: Option<StoredTournament>, role: Role) -> Result<StoredTournament, ApiError> {
    found
        .filter(|t| can_see(t.status, t.visibility, role, t.registered))
        .ok_or(NOT_FOUND)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn who_sees_what() {
        use {
            Role::{Organizer, Owner, Player},
            TournamentStatus::{Draft, Finished, Registration},
            Visibility::{Private, Public},
        };
        for status in [Draft, Registration, Finished] {
            for visibility in [Public, Private] {
                assert!(can_see(status, visibility, Organizer, false));
                assert!(can_see(status, visibility, Owner, false));
                // Inscrito em rascunho (o organizador já inscreveu) ainda não vê.
                assert_eq!(can_see(status, visibility, Player, true), status != Draft);
            }
        }
        assert!(can_see(Registration, Public, Player, false));
        assert!(can_see(Finished, Public, Player, false));
        assert!(!can_see(Registration, Private, Player, false));
        assert!(!can_see(Finished, Private, Player, false));
    }
}
