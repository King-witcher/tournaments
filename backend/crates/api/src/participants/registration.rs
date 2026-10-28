//! Inscrição num torneio pelos três caminhos (o próprio membro, link de
//! convite, organizador): o que cada um exige e o passo de inscrever, que o
//! aceite do convite de torneio também usa.

use sqlx::PgConnection;

use super::db;
use crate::{
    shared::ApiError,
    tenants::TenantId,
    tournaments::{self, StoredTournament, TournamentStatus, Visibility},
};

/// Por onde a inscrição chega.
#[derive(Debug, Clone, Copy)]
pub enum Entry {
    /// O próprio membro, pelo botão: só em torneio público.
    Member,
    /// Link de convite de torneio (o id do convite).
    Invite(i64),
    /// O organizador inscrevendo um membro (o id de quem inscreveu).
    Organizer(i64),
}

/// Status e visibilidade: 409 `tournament.private`,
/// `tournament.registration_closed` ou `tournament.participants_locked`.
pub fn check_open(
    status: TournamentStatus,
    visibility: Visibility,
    entry: Entry,
) -> Result<(), ApiError> {
    match entry {
        Entry::Member if visibility == Visibility::Private => {
            Err(ApiError::conflict("tournament.private"))
        }
        Entry::Member | Entry::Invite(_) if !status.registration_open() => {
            Err(ApiError::conflict("tournament.registration_closed"))
        }
        Entry::Organizer(_) if !status.participants_editable() => {
            Err(ApiError::conflict("tournament.participants_locked"))
        }
        _ => Ok(()),
    }
}

/// 409 `tournament.full` `{max}` quando não sobra vaga.
pub fn check_capacity(registered: i64, max_participants: i32) -> Result<(), ApiError> {
    if registered >= i64::from(max_participants) {
        return Err(ApiError::conflict("tournament.full")
            .with_params([("max", i64::from(max_participants))]));
    }
    Ok(())
}

/// Inscreve `user_id` no torneio, que quem chama já travou
/// (`find_for_update`), então a contagem de inscritos vale até o commit.
/// Quem já estava inscrito continua, sem erro: devolve `false`. Membro
/// inexistente: 404 `member.not_found`.
pub async fn register(
    conn: &mut PgConnection,
    tenant: TenantId,
    tournament: &StoredTournament,
    user_id: i64,
    entry: Entry,
) -> Result<bool, ApiError> {
    if db::is_registered(conn, tournament.id, user_id).await? {
        return Ok(false);
    }
    check_open(tournament.status, tournament.visibility, entry)?;
    check_capacity(tournament.participants, tournament.setup.max_participants)?;
    let (invite_id, added_by) = match entry {
        Entry::Member => (None, None),
        Entry::Invite(invite_id) => (Some(invite_id), None),
        Entry::Organizer(by) => (None, Some(by)),
    };
    if !db::insert(conn, tenant, tournament.id, user_id, invite_id, added_by).await? {
        return Err(ApiError::not_found("member.not_found"));
    }
    tournaments::changed(conn, tournament.id).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tournaments::{
        TournamentStatus::{CheckIn, Draft, Finished, InProgress, Registration},
        Visibility::{Private, Public},
    };

    fn code(status: TournamentStatus, visibility: Visibility, entry: Entry) -> &'static str {
        check_open(status, visibility, entry).map_or_else(|e| e.code, |()| "ok")
    }

    #[test]
    fn members_register_themselves_only_in_open_public_tournaments() {
        assert_eq!(code(Registration, Public, Entry::Member), "ok");
        assert_eq!(
            code(Registration, Private, Entry::Member),
            "tournament.private"
        );
        for status in [Draft, CheckIn, InProgress, Finished] {
            assert_eq!(
                code(status, Public, Entry::Member),
                "tournament.registration_closed"
            );
        }
    }

    #[test]
    fn invite_links_need_open_registration_in_any_visibility() {
        for visibility in [Public, Private] {
            assert_eq!(code(Registration, visibility, Entry::Invite(1)), "ok");
            assert_eq!(
                code(Draft, visibility, Entry::Invite(1)),
                "tournament.registration_closed"
            );
        }
    }

    #[test]
    fn organizers_add_until_the_tournament_starts() {
        for status in [Draft, Registration, CheckIn] {
            assert_eq!(code(status, Private, Entry::Organizer(1)), "ok");
        }
        for status in [InProgress, Finished] {
            assert_eq!(
                code(status, Public, Entry::Organizer(1)),
                "tournament.participants_locked"
            );
        }
    }

    #[test]
    fn capacity() {
        assert!(check_capacity(15, 16).is_ok());
        let err = check_capacity(16, 16).unwrap_err();
        assert_eq!(
            (err.code, err.params),
            ("tournament.full", vec![("max", 16)])
        );
    }
}
