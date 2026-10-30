//! Quem anexa e quem remove um screenshot. Anexam os jogadores da série e a
//! organização (organizer e owner), numa partida enviada ou na próxima, ainda
//! por enviar, com o torneio em andamento; o jogador, só enquanto a partida
//! ainda pode ser contestada: antes de confirmar e, depois, enquanto a
//! contestação tardia valer (até a próxima série de um dos lados começar).
//! Depois, o screenshot não serve de prova de nada. Quem anexou remove o
//! próprio enquanto a partida não confirmou; a organização remove qualquer
//! um, a qualquer hora.

use crate::{matches::MatchGames, shared::ApiError, tournaments::TournamentStatus};

const FORBIDDEN: ApiError = ApiError::forbidden("screenshots.forbidden");
const CONFIRMED: ApiError = ApiError::conflict("screenshots.game_confirmed");

/// Quem age.
#[derive(Debug, Clone, Copy)]
pub struct Actor {
    pub user_id: i64,
    /// Organizer ou owner.
    pub organizer: bool,
}

/// Onde está a partida `number` na série.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Enviada, esperando confirmação (pendente ou contestada).
    Open,
    /// Confirmada, e a contestação tardia ainda vale.
    Contestable,
    Confirmed,
    /// A próxima, ainda não enviada.
    Next,
    /// Nem enviada nem a próxima.
    Unavailable,
}

impl Slot {
    pub fn of(m: &MatchGames, number: i16) -> Self {
        match m.game(number) {
            Some(g) if g.confirmed && g.contestable => Self::Contestable,
            Some(g) if g.confirmed => Self::Confirmed,
            Some(_) => Self::Open,
            None if m.next_game == Some(number) => Self::Next,
            None => Self::Unavailable,
        }
    }
}

/// Erros: 409 `tournament.not_in_progress`, 409 `screenshots.game_unavailable`
/// (partida que nem foi enviada nem é a próxima), 403 `screenshots.forbidden`
/// (não joga a série nem organiza), 409 `screenshots.game_confirmed`
/// (jogador, partida confirmada que não dá mais para contestar).
pub fn attach(
    status: TournamentStatus,
    plays: bool,
    actor: Actor,
    slot: Slot,
) -> Result<(), ApiError> {
    if status != TournamentStatus::InProgress {
        return Err(ApiError::conflict("tournament.not_in_progress"));
    }
    if slot == Slot::Unavailable {
        return Err(ApiError::conflict("screenshots.game_unavailable"));
    }
    if actor.organizer {
        return Ok(());
    }
    if !plays {
        return Err(FORBIDDEN);
    }
    if slot == Slot::Confirmed {
        return Err(CONFIRMED);
    }
    Ok(())
}

/// Erros: 403 `screenshots.forbidden` (screenshot de outra pessoa) e 409
/// `screenshots.game_confirmed` (o próprio, com a partida já confirmada).
pub fn remove(actor: Actor, uploaded_by: Option<i64>, slot: Slot) -> Result<(), ApiError> {
    if actor.organizer {
        return Ok(());
    }
    if uploaded_by != Some(actor.user_id) {
        return Err(FORBIDDEN);
    }
    if matches!(slot, Slot::Confirmed | Slot::Contestable) {
        return Err(CONFIRMED);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const IN_PROGRESS: TournamentStatus = TournamentStatus::InProgress;
    const PLAYER: Actor = Actor {
        user_id: 1,
        organizer: false,
    };
    const STAFF: Actor = Actor {
        user_id: 9,
        organizer: true,
    };

    fn code(r: Result<(), ApiError>) -> &'static str {
        r.err().map_or("ok", |e| e.code)
    }

    #[test]
    fn players_attach_while_the_game_can_be_contested() {
        for slot in [Slot::Next, Slot::Open, Slot::Contestable] {
            assert_eq!(code(attach(IN_PROGRESS, true, PLAYER, slot)), "ok");
        }
        let confirmed = attach(IN_PROGRESS, true, PLAYER, Slot::Confirmed);
        assert_eq!(code(confirmed), "screenshots.game_confirmed");
        let outsider = attach(IN_PROGRESS, false, PLAYER, Slot::Next);
        assert_eq!(code(outsider), "screenshots.forbidden");
    }

    #[test]
    fn staff_attach_to_any_sent_map() {
        for slot in [Slot::Next, Slot::Open, Slot::Confirmed] {
            assert_eq!(code(attach(IN_PROGRESS, false, STAFF, slot)), "ok");
        }
    }

    #[test]
    fn only_sent_or_next_maps_and_only_in_progress() {
        let far = attach(IN_PROGRESS, true, STAFF, Slot::Unavailable);
        assert_eq!(code(far), "screenshots.game_unavailable");
        let finished = attach(TournamentStatus::Finished, true, STAFF, Slot::Open);
        assert_eq!(code(finished), "tournament.not_in_progress");
    }

    #[test]
    fn removing() {
        assert_eq!(code(remove(PLAYER, Some(1), Slot::Open)), "ok");
        assert_eq!(code(remove(PLAYER, Some(1), Slot::Next)), "ok");
        for slot in [Slot::Confirmed, Slot::Contestable] {
            let late = remove(PLAYER, Some(1), slot);
            assert_eq!(code(late), "screenshots.game_confirmed");
        }
        assert_eq!(
            code(remove(PLAYER, Some(2), Slot::Open)),
            "screenshots.forbidden"
        );
        assert_eq!(
            code(remove(PLAYER, None, Slot::Open)),
            "screenshots.forbidden"
        );
        assert_eq!(code(remove(STAFF, Some(2), Slot::Confirmed)), "ok");
    }
}
