//! Prints gravados → resposta da API, com a URL assinada de cada um e o que
//! quem vê pode fazer.

use chrono::{DateTime, Utc};

use super::{
    db::PrintRow,
    policy::{self, Actor, Slot},
    types::{GamePrints, MatchPrints, Print, PrintAuthor},
};
use crate::{matches::MatchGames, shared::Storage};

/// Os limites que a resposta repete.
#[derive(Clone, Copy)]
pub struct Limits {
    pub max_bytes: u32,
    pub max_per_game: u8,
}

/// Sem o bucket: nada de prints.
pub fn disabled(limits: Limits) -> MatchPrints {
    MatchPrints {
        enabled: false,
        required: false,
        max_bytes: limits.max_bytes,
        max_per_game: limits.max_per_game,
        games: Vec::new(),
    }
}

/// Um mapa por mapa enviado e o próximo, cada um com os prints dele.
pub fn match_prints(
    m: &MatchGames,
    rows: &[PrintRow],
    storage: &Storage,
    actor: Actor,
    limits: Limits,
    now: DateTime<Utc>,
) -> MatchPrints {
    let t = &m.tournament;
    let numbers = m.games.iter().map(|g| g.number).chain(m.next_game);
    let games = numbers
        .map(|number| {
            let slot = Slot::of(m, number);
            let prints: Vec<Print> = rows
                .iter()
                .filter(|r| r.number == number)
                .map(|r| print(r, storage, actor, slot, now))
                .collect();
            let room = prints.len() < usize::from(limits.max_per_game);
            let allowed = policy::attach(t.status, m.plays(actor.user_id), actor, slot).is_ok();
            GamePrints {
                number,
                can_attach: room && allowed,
                prints,
            }
        })
        .collect();
    MatchPrints {
        enabled: true,
        required: t.prints_required,
        max_bytes: limits.max_bytes,
        max_per_game: limits.max_per_game,
        games,
    }
}

pub fn print(
    r: &PrintRow,
    storage: &Storage,
    actor: Actor,
    slot: Slot,
    now: DateTime<Utc>,
) -> Print {
    Print {
        id: r.id,
        url: storage.get_url(&r.object_key, now).into(),
        content_type: r.content_type,
        uploaded_by: r
            .uploaded_by
            .zip(r.nickname.clone())
            .map(|(user_id, nickname)| PrintAuthor { user_id, nickname }),
        attached_at: r.attached_at,
        can_remove: policy::remove(actor, r.uploaded_by, slot).is_ok(),
    }
}
