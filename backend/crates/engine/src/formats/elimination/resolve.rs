//! Resolução dos lados e dos estados das partidas conforme os resultados.

use super::build::mid;
use crate::core::{
    BracketSide, Match, MatchId, MatchState, Occupant, Outcome, ParticipantId, Slot,
};

pub(super) fn loser_of(a: ParticipantId, b: ParticipantId, outcome: Outcome) -> ParticipantId {
    if outcome.winner() == a { b } else { a }
}

fn lookup(matches: &[Match], id: MatchId) -> Option<&Match> {
    matches
        .binary_search_by_key(&id, |m| m.id)
        .ok()
        .map(|i| &matches[i])
}

/// Quem ocupa `slot` com os estados atuais de `matches`.
pub(super) fn occupant(matches: &[Match], slot: Slot) -> Occupant {
    match slot {
        Slot::Participant(p) => Occupant::Participant(p),
        Slot::Bye => Occupant::Bye,
        Slot::WinnerOf(id) => match lookup(matches, id).map(|m| m.state) {
            Some(MatchState::Completed { outcome, .. }) => Occupant::Participant(outcome.winner()),
            Some(MatchState::Bye {
                advanced: Some(p), ..
            }) => Occupant::Participant(p),
            Some(MatchState::Bye { advanced: None }) => Occupant::Bye,
            _ => Occupant::Unknown,
        },
        Slot::LoserOf(id) => match lookup(matches, id).map(|m| m.state) {
            Some(MatchState::Completed { a, b, outcome }) => {
                Occupant::Participant(loser_of(a, b, outcome))
            }
            Some(MatchState::Bye { .. }) => Occupant::Bye,
            _ => Occupant::Unknown,
        },
    }
}

/// Resolve os estados pendentes. Toda dependência aponta para uma partida
/// anterior na ordem de `matches`, então uma passada basta (os byes cascateiam).
pub(super) fn resolve(matches: &mut [Match]) {
    use Occupant::{Bye, Participant};
    for i in 0..matches.len() {
        if !matches!(matches[i].state, MatchState::Pending) {
            continue;
        }
        let m = matches[i];
        if m.id.side == BracketSide::GrandFinal
            && m.id.round == 2
            && let Some(Match {
                state: MatchState::Completed { a, outcome, .. },
                ..
            }) = lookup(matches, mid(BracketSide::GrandFinal, 1, 1))
            && outcome.winner() == *a
        {
            matches[i].state = MatchState::NotNeeded;
            continue;
        }
        let new_state = match (occupant(matches, m.a), occupant(matches, m.b)) {
            (Participant(a), Participant(b)) => MatchState::Ready { a, b },
            (Bye, Bye) => MatchState::Bye { advanced: None },
            (Bye, Participant(x)) | (Participant(x), Bye) => MatchState::Bye { advanced: Some(x) },
            _ => continue,
        };
        matches[i].state = new_state;
    }
}
