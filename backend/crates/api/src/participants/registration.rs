//! Inscrição num torneio pelos três caminhos (o próprio membro, link de
//! convite, organizador): o que cada um exige e o passo de inscrever, que o
//! aceite do convite de torneio também usa.

use sqlx::PgConnection;

use super::db;
use crate::{
    shared::ApiError,
    tenants::TenantId,
    tournaments::{self, Admission, StoredTournament, TournamentStatus},
};

/// Por onde a inscrição chega.
#[derive(Debug, Clone, Copy)]
pub enum Entry {
    /// O próprio membro, pelo botão: só com a inscrição aberta (`Admission::Open`).
    Member,
    /// Link de convite de torneio (o id do convite).
    Invite(i64),
    /// O organizador inscrevendo um membro (o id de quem inscreveu).
    Organizer(i64),
}

/// Status e tipo de inscrição: 409 `tournament.invite_only`,
/// `tournament.registration_closed` ou `tournament.participants_locked`.
pub fn check_open(
    status: TournamentStatus,
    admission: Admission,
    entry: Entry,
) -> Result<(), ApiError> {
    match entry {
        Entry::Member if admission == Admission::InviteOnly => {
            Err(ApiError::conflict("tournament.invite_only"))
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
/// (`find_for_update`), então a contagem de inscritos vale até o commit. Com
/// o torneio em check-in, o inscrito entra com o check-in feito (de quem
/// inscreveu, se foi o organizador).
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
    check_open(tournament.status, tournament.admission, entry)?;
    check_capacity(tournament.participants, tournament.setup.max_participants)?;
    let (invite_id, added_by) = match entry {
        Entry::Member => (None, None),
        Entry::Invite(invite_id) => (Some(invite_id), None),
        Entry::Organizer(by) => (None, Some(by)),
    };
    // Durante o check-in a inscrição fica aberta, e quem entra já entra com o
    // check-in feito (o banco exige: `participants_check_in_on_entry`).
    let checked_in = tournament.status.check_in_open();
    let entered = db::insert(
        conn,
        tenant,
        tournament.id,
        user_id,
        invite_id,
        added_by,
        checked_in,
    )
    .await?;
    if !entered {
        return Err(ApiError::not_found("member.not_found"));
    }
    tournaments::changed(conn, tournament.id).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tournaments::{
        Admission::{InviteOnly, Open},
        TournamentStatus::{CheckIn, Draft, Finished, InProgress, Registration},
    };

    fn code(status: TournamentStatus, admission: Admission, entry: Entry) -> &'static str {
        check_open(status, admission, entry).map_or_else(|e| e.code, |()| "ok")
    }

    #[test]
    fn members_register_themselves_until_the_start() {
        for status in [Registration, CheckIn] {
            assert_eq!(code(status, Open, Entry::Member), "ok");
            assert_eq!(
                code(status, InviteOnly, Entry::Member),
                "tournament.invite_only"
            );
        }
        for status in [Draft, InProgress, Finished] {
            assert_eq!(
                code(status, Open, Entry::Member),
                "tournament.registration_closed"
            );
        }
    }

    #[test]
    fn invite_links_work_until_the_start_in_any_admission() {
        for admission in [Open, InviteOnly] {
            for status in [Registration, CheckIn] {
                assert_eq!(code(status, admission, Entry::Invite(1)), "ok");
            }
            for status in [Draft, InProgress, Finished] {
                assert_eq!(
                    code(status, admission, Entry::Invite(1)),
                    "tournament.registration_closed"
                );
            }
        }
    }

    #[test]
    fn organizers_add_until_the_tournament_starts() {
        for status in [Draft, Registration, CheckIn] {
            assert_eq!(code(status, InviteOnly, Entry::Organizer(1)), "ok");
        }
        for status in [InProgress, Finished] {
            assert_eq!(
                code(status, Open, Entry::Organizer(1)),
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
