//! Quem anexa e quem remove um print. Anexam os dois jogadores da partida e a
//! organização (organizer e owner), num mapa enviado ou no próximo, ainda por
//! enviar, com o torneio em andamento; o jogador, só enquanto o mapa não
//! confirmou (depois, o print não serve de prova de nada). Quem anexou remove
//! o próprio enquanto o mapa não confirmou; a organização remove qualquer um,
//! a qualquer hora.

use crate::{matches::MatchGames, shared::ApiError, tournaments::TournamentStatus};

const FORBIDDEN: ApiError = ApiError::forbidden("prints.forbidden");
const CONFIRMED: ApiError = ApiError::conflict("prints.game_confirmed");

/// Quem age.
#[derive(Debug, Clone, Copy)]
pub struct Actor {
    pub user_id: i64,
    /// Organizer ou owner.
    pub organizer: bool,
}

/// Onde está o mapa `number` na série.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Enviado, esperando confirmação (pendente ou contestado).
    Open,
    Confirmed,
    /// O próximo, ainda não enviado.
    Next,
    /// Nem enviado nem o próximo.
    Unavailable,
}

impl Slot {
    pub fn of(m: &MatchGames, number: i16) -> Self {
        match m.game(number) {
            Some(g) if g.confirmed => Self::Confirmed,
            Some(_) => Self::Open,
            None if m.next_game == Some(number) => Self::Next,
            None => Self::Unavailable,
        }
    }
}

/// Erros: 409 `tournament.not_in_progress`, 409 `prints.game_unavailable`
/// (mapa que nem foi enviado nem é o próximo), 403 `prints.forbidden` (não
/// joga a partida nem organiza), 409 `prints.game_confirmed` (jogador, mapa já
/// confirmado).
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
        return Err(ApiError::conflict("prints.game_unavailable"));
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

/// Erros: 403 `prints.forbidden` (print de outra pessoa) e 409
/// `prints.game_confirmed` (o próprio, com o mapa já confirmado).
pub fn remove(actor: Actor, uploaded_by: Option<i64>, slot: Slot) -> Result<(), ApiError> {
    if actor.organizer {
        return Ok(());
    }
    if uploaded_by != Some(actor.user_id) {
        return Err(FORBIDDEN);
    }
    if slot == Slot::Confirmed {
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
    fn players_attach_until_the_map_confirms() {
        for slot in [Slot::Next, Slot::Open] {
            assert_eq!(code(attach(IN_PROGRESS, true, PLAYER, slot)), "ok");
        }
        let confirmed = attach(IN_PROGRESS, true, PLAYER, Slot::Confirmed);
        assert_eq!(code(confirmed), "prints.game_confirmed");
        let outsider = attach(IN_PROGRESS, false, PLAYER, Slot::Next);
        assert_eq!(code(outsider), "prints.forbidden");
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
        assert_eq!(code(far), "prints.game_unavailable");
        let finished = attach(TournamentStatus::Finished, true, STAFF, Slot::Open);
        assert_eq!(code(finished), "tournament.not_in_progress");
    }

    #[test]
    fn removing() {
        assert_eq!(code(remove(PLAYER, Some(1), Slot::Open)), "ok");
        assert_eq!(code(remove(PLAYER, Some(1), Slot::Next)), "ok");
        let late = remove(PLAYER, Some(1), Slot::Confirmed);
        assert_eq!(code(late), "prints.game_confirmed");
        assert_eq!(
            code(remove(PLAYER, Some(2), Slot::Open)),
            "prints.forbidden"
        );
        assert_eq!(code(remove(PLAYER, None, Slot::Open)), "prints.forbidden");
        assert_eq!(code(remove(STAFF, Some(2), Slot::Confirmed)), "ok");
    }
}
