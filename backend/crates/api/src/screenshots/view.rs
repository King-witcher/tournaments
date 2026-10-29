//! Screenshots gravados → resposta da API, com a URL assinada de cada um e o
//! que quem vê pode fazer.

use chrono::{DateTime, Utc};

use super::{
    db::ScreenshotRow,
    policy::{self, Actor, Slot},
    types::{GameScreenshots, MatchScreenshots, Screenshot, ScreenshotAuthor},
};
use crate::{matches::MatchGames, shared::Storage};

/// Os limites que a resposta repete.
#[derive(Clone, Copy)]
pub struct Limits {
    pub max_bytes: u32,
    pub max_per_game: u8,
}

/// Sem o bucket: nada de screenshots.
pub fn disabled(limits: Limits) -> MatchScreenshots {
    MatchScreenshots {
        enabled: false,
        required: false,
        max_bytes: limits.max_bytes,
        max_per_game: limits.max_per_game,
        games: Vec::new(),
    }
}

/// Um mapa por mapa enviado e o próximo, cada um com os screenshots dele.
pub fn match_screenshots(
    m: &MatchGames,
    rows: &[ScreenshotRow],
    storage: &Storage,
    actor: Actor,
    limits: Limits,
    now: DateTime<Utc>,
) -> MatchScreenshots {
    let t = &m.tournament;
    let numbers = m.games.iter().map(|g| g.number).chain(m.next_game);
    let games = numbers
        .map(|number| {
            let slot = Slot::of(m, number);
            let screenshots: Vec<Screenshot> = rows
                .iter()
                .filter(|r| r.number == number)
                .map(|r| screenshot(r, storage, actor, slot, now))
                .collect();
            let room = screenshots.len() < usize::from(limits.max_per_game);
            let allowed = policy::attach(t.status, m.plays(actor.user_id), actor, slot).is_ok();
            GameScreenshots {
                number,
                can_attach: room && allowed,
                screenshots,
            }
        })
        .collect();
    MatchScreenshots {
        enabled: true,
        required: t.screenshots_required,
        max_bytes: limits.max_bytes,
        max_per_game: limits.max_per_game,
        games,
    }
}

pub fn screenshot(
    r: &ScreenshotRow,
    storage: &Storage,
    actor: Actor,
    slot: Slot,
    now: DateTime<Utc>,
) -> Screenshot {
    Screenshot {
        id: r.id,
        url: storage.get_url(&r.object_key, now).into(),
        content_type: r.content_type,
        uploaded_by: r
            .uploaded_by
            .zip(r.nickname.clone())
            .map(|(user_id, nickname)| ScreenshotAuthor { user_id, nickname }),
        attached_at: r.attached_at,
        can_remove: policy::remove(actor, r.uploaded_by, slot).is_ok(),
    }
}
