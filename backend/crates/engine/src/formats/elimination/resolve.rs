//! Resolução dos lados e dos estados das partidas conforme os resultados.

use super::build::mid;
use crate::core::{BracketSide, Match, MatchId, MatchState, Outcome, ParticipantId, Slot};

#[derive(Clone, Copy)]
enum Side {
    Unknown,
    Known(ParticipantId),
    Bye,
}

pub(super) fn loser_of(a: ParticipantId, b: ParticipantId, outcome: Outcome) -> ParticipantId {
    if outcome.winner() == a { b } else { a }
}

fn lookup(matches: &[Match], id: MatchId) -> Option<&Match> {
    matches
        .binary_search_by_key(&id, |m| m.id)
        .ok()
        .map(|i| &matches[i])
}

fn resolve_slot(matches: &[Match], slot: Slot) -> Side {
    match slot {
        Slot::Participant(p) => Side::Known(p),
        Slot::Bye => Side::Bye,
        Slot::WinnerOf(id) => match lookup(matches, id).map(|m| m.state) {
            Some(MatchState::Completed { outcome, .. }) => Side::Known(outcome.winner()),
            Some(MatchState::Bye {
                advanced: Some(p), ..
            }) => Side::Known(p),
            Some(MatchState::Bye { advanced: None }) => Side::Bye,
            _ => Side::Unknown,
        },
        Slot::LoserOf(id) => match lookup(matches, id).map(|m| m.state) {
            Some(MatchState::Completed { a, b, outcome }) => Side::Known(loser_of(a, b, outcome)),
            Some(MatchState::Bye { .. }) => Side::Bye,
            _ => Side::Unknown,
        },
    }
}

/// Resolve os estados pendentes. Toda dependência aponta para uma partida
/// anterior na ordem de `matches`, então uma passada basta (os byes cascateiam).
pub(super) fn resolve(matches: &mut [Match]) {
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
        let new_state = match (resolve_slot(matches, m.a), resolve_slot(matches, m.b)) {
            (Side::Known(a), Side::Known(b)) => MatchState::Ready { a, b },
            (Side::Bye, Side::Bye) => MatchState::Bye { advanced: None },
            (Side::Bye, Side::Known(x)) | (Side::Known(x), Side::Bye) => {
                MatchState::Bye { advanced: Some(x) }
            }
            _ => continue,
        };
        matches[i].state = new_state;
    }
}
