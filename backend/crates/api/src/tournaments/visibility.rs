//! Quem vê um torneio. Organizer e owner veem todos; os outros membros veem
//! todo torneio publicado, e para eles o rascunho responde 404, como se não
//! existisse.

use sqlx::PgConnection;

use super::{
    db::{self, StoredTournament},
    status::TournamentStatus,
};
use crate::{
    shared::ApiError,
    tenants::{Role, TenantMember},
};

const NOT_FOUND: ApiError = ApiError::not_found("tournament.not_found");

/// A mesma regra está no SQL de `db::list`.
pub const fn can_see(status: TournamentStatus, role: Role) -> bool {
    matches!(role, Role::Organizer | Role::Owner) || !matches!(status, TournamentStatus::Draft)
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
    found.filter(|t| can_see(t.status, role)).ok_or(NOT_FOUND)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn who_sees_what() {
        use {
            Role::{Organizer, Owner, Player},
            TournamentStatus::{CheckIn, Draft, Finished, InProgress, Registration},
        };
        assert!(can_see(Draft, Organizer));
        assert!(can_see(Draft, Owner));
        assert!(!can_see(Draft, Player));
        for status in [Registration, CheckIn, InProgress, Finished] {
            for role in [Player, Organizer, Owner] {
                assert!(can_see(status, role));
            }
        }
    }
}
