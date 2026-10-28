//! Sair da comunidade e remover um membro. A linha do membro fica (inativa),
//! com o histórico: ele perde o acesso, sai das inscrições dos torneios que
//! não começaram e tem os convites que criou revogados. Voltar por convite
//! reativa a mesma linha (`invites`).

use sqlx::PgConnection;

use super::db;
use crate::{
    invites, lifecycle,
    shared::ApiError,
    tenants::{Role, TenantId},
};

/// Quem `by` pode remover: o owner, qualquer um; o organizer, só players.
/// Ninguém remove o owner (409 `member.is_owner`); papel abaixo, 403
/// `tenant.forbidden`.
pub fn check_removal(by: Role, target: Role) -> Result<(), ApiError> {
    match (by, target) {
        (_, Role::Owner) => Err(ApiError::conflict("member.is_owner")),
        (Role::Owner, _) | (Role::Organizer, Role::Player) => Ok(()),
        _ => Err(ApiError::forbidden("tenant.forbidden")),
    }
}

/// O owner não sai: transfere a posse antes (409
/// `member.owner_must_transfer`).
pub fn check_leave(role: Role) -> Result<(), ApiError> {
    if role == Role::Owner {
        return Err(ApiError::conflict("member.owner_must_transfer"));
    }
    Ok(())
}

/// `user_id` deixa o tenant (`by`: quem removeu; `None`, saiu sozinho), numa
/// transação que quem chama abre. `allowed` confere o papel dele, lido com a
/// linha travada. 404 `member.not_found` se ele não é membro ativo.
pub async fn depart(
    conn: &mut PgConnection,
    tenant: TenantId,
    user_id: i64,
    by: Option<i64>,
    allowed: impl FnOnce(Role) -> Result<(), ApiError>,
) -> Result<(), ApiError> {
    let role = db::lock_role(conn, tenant, user_id)
        .await?
        .ok_or(ApiError::not_found("member.not_found"))?;
    allowed(role)?;
    lifecycle::leave_tournaments(conn, tenant, user_id).await?;
    invites::revoke_created_by(conn, tenant, user_id).await?;
    db::deactivate(conn, tenant, user_id, by).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use Role::{Organizer, Owner, Player};

    fn code(r: Result<(), ApiError>) -> &'static str {
        r.map_or_else(|e| e.code, |()| "ok")
    }

    #[test]
    fn who_removes_whom() {
        assert_eq!(code(check_removal(Owner, Organizer)), "ok");
        assert_eq!(code(check_removal(Owner, Player)), "ok");
        assert_eq!(code(check_removal(Organizer, Player)), "ok");
        assert_eq!(
            code(check_removal(Organizer, Organizer)),
            "tenant.forbidden"
        );
        assert_eq!(code(check_removal(Player, Player)), "tenant.forbidden");
        for by in [Owner, Organizer, Player] {
            assert_eq!(code(check_removal(by, Owner)), "member.is_owner");
        }
    }

    #[test]
    fn the_owner_transfers_before_leaving() {
        assert_eq!(code(check_leave(Organizer)), "ok");
        assert_eq!(code(check_leave(Player)), "ok");
        assert_eq!(code(check_leave(Owner)), "member.owner_must_transfer");
    }
}
