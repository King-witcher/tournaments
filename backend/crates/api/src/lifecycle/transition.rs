//! As transições manuais de status depois de publicar, num lugar só: de onde
//! cada uma sai, para onde vai e o que faz junto. Publicar fica em
//! `tournaments` (só mexe no torneio) e a abertura automática do check-in em
//! `tournaments::schedule`, aplicada ao ler. Iniciar gera a chave e desfazer
//! a descarta, com os resultados (`matches::start`, `matches::discard`); em
//! andamento ↔ encerrado anda sozinho conforme a chave, em `matches`.
//!
//! ```text
//! com check-in: inscrição ──abrir check-in (ou a hora)──▶ check-in ──iniciar──▶ em andamento
//!                         ◀───────reabrir inscrição──────          ◀──desfazer──
//! sem check-in: inscrição ──────────────────iniciar────────────────────────────▶ em andamento
//!                         ◀─────────────────desfazer─────────────────────────────
//! ```

use engine::core::ParticipantId;
use sqlx::PgConnection;

use crate::{
    matches, participants,
    shared::ApiError,
    tournaments::{self, StoredTournament, TournamentStatus},
};

/// Mínimo de elegíveis para iniciar (a chave precisa de dois lados).
const MIN_TO_START: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// Inscrição → check-in, antes da hora da abertura automática.
    OpenCheckIn,
    /// Check-in → inscrição. A abertura automática não reabre a mesma janela.
    ReopenRegistration,
    /// Check-in (ou inscrição, sem check-in) → em andamento.
    Start,
    /// Em andamento → check-in (ou inscrição, sem check-in), para quando se
    /// iniciou por engano.
    UndoStart,
}

impl Transition {
    /// O status depois da transição, saindo de `from`. Erros (409):
    /// `tournament.not_registration`, `tournament.check_in_disabled`,
    /// `tournament.not_check_in`, `tournament.not_published`,
    /// `tournament.check_in_not_open`, `tournament.already_started` e
    /// `tournament.not_in_progress`.
    pub fn target(
        self,
        from: TournamentStatus,
        has_check_in: bool,
    ) -> Result<TournamentStatus, ApiError> {
        use TournamentStatus::*;
        let err = ApiError::conflict;
        match (self, from) {
            (Self::OpenCheckIn, Registration) if has_check_in => Ok(CheckIn),
            (Self::OpenCheckIn, Registration) => Err(err("tournament.check_in_disabled")),
            (Self::OpenCheckIn, _) => Err(err("tournament.not_registration")),
            (Self::ReopenRegistration, CheckIn) => Ok(Registration),
            (Self::ReopenRegistration, _) => Err(err("tournament.not_check_in")),
            (Self::Start, CheckIn) => Ok(InProgress),
            (Self::Start, Registration) if !has_check_in => Ok(InProgress),
            (Self::Start, Registration) => Err(err("tournament.check_in_not_open")),
            (Self::Start, Draft) => Err(err("tournament.not_published")),
            (Self::Start, InProgress | Finished) => Err(err("tournament.already_started")),
            (Self::UndoStart, InProgress) if has_check_in => Ok(CheckIn),
            (Self::UndoStart, InProgress) => Ok(Registration),
            (Self::UndoStart, _) => Err(err("tournament.not_in_progress")),
        }
    }
}

/// Aplica `transition` ao torneio, que quem chama já travou
/// (`find_for_update`), com o que ela faz junto.
pub async fn apply(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    transition: Transition,
) -> Result<(), ApiError> {
    let to = transition.target(tournament.status, tournament.schedule.has_check_in())?;
    match transition {
        Transition::ReopenRegistration => {
            tournaments::mark_registration_reopened(conn, tournament.id).await?;
        }
        Transition::Start => {
            let seeded: Vec<ParticipantId> =
                participants::finalize_seeding(conn, tournament, MIN_TO_START).await?;
            matches::start(conn, tournament, &seeded).await?;
        }
        Transition::UndoStart => {
            matches::discard(conn, tournament).await?;
        }
        Transition::OpenCheckIn => {}
    }
    tournaments::set_status(conn, tournament.id, to).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use TournamentStatus::*;

    const ALL: [TournamentStatus; 5] = [Draft, Registration, CheckIn, InProgress, Finished];

    /// Para onde cada status vai, com e sem check-in (código do erro se não vai).
    fn targets(t: Transition, has_check_in: bool) -> [Result<TournamentStatus, &'static str>; 5] {
        ALL.map(|from| t.target(from, has_check_in).map_err(|e| e.code))
    }

    #[test]
    fn open_check_in() {
        let not = Err("tournament.not_registration");
        assert_eq!(
            targets(Transition::OpenCheckIn, true),
            [not, Ok(CheckIn), not, not, not]
        );
        assert_eq!(
            targets(Transition::OpenCheckIn, false)[1],
            Err("tournament.check_in_disabled")
        );
    }

    #[test]
    fn reopen_registration() {
        let not = Err("tournament.not_check_in");
        assert_eq!(
            targets(Transition::ReopenRegistration, true),
            [not, not, Ok(Registration), not, not]
        );
    }

    #[test]
    fn start_needs_check_in_open_or_no_check_in() {
        let started = Err("tournament.already_started");
        assert_eq!(
            targets(Transition::Start, true),
            [
                Err("tournament.not_published"),
                Err("tournament.check_in_not_open"),
                Ok(InProgress),
                started,
                started
            ]
        );
        assert_eq!(targets(Transition::Start, false)[1], Ok(InProgress));
    }

    #[test]
    fn undo_start_goes_back_to_where_it_started() {
        let not = Err("tournament.not_in_progress");
        assert_eq!(
            targets(Transition::UndoStart, true),
            [not, not, not, Ok(CheckIn), not]
        );
        assert_eq!(targets(Transition::UndoStart, false)[3], Ok(Registration));
    }
}
