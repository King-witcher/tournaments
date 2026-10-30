//! Quando uma partida da série foi liberada, isto é, passou a poder ser
//! enviada pelo mesmo critério do envio: o prazo de confirmação do envio
//! feito antes da duração prevista conta do fim dela (`report`), para o
//! adversário que ainda está jogando não perder o prazo.
//!
//! Liberada é o mais tarde destes momentos:
//! - a série pronta para os dois lados: o início do torneio, a série
//!   anterior de cada lado decidida e, no suíço, a rodada anterior inteira
//!   (o pareamento espera por ela);
//! - a partida anterior da série enviada;
//! - com veto, o veto dela concluído (`veto::released_at`), seja ele por
//!   partida, seja da série inteira antes da primeira.

use chrono::{DateTime, Utc};
use engine::core::{BracketSide, Match, ParticipantId};

use super::{stage::Stage, veto};

/// A partida `game` da série `m`, entre `sides`, no torneio que começou em
/// `started_at`. `None` só sem nada disso (torneio antigo, sem o início).
pub fn released_at(
    stage: &Stage,
    m: &Match,
    (a, b): (ParticipantId, ParticipantId),
    game: i16,
    started_at: Option<DateTime<Utc>>,
) -> Option<DateTime<Utc>> {
    let previous_round = (m.id.side == BracketSide::Swiss)
        .then(|| m.id.round.checked_sub(1))
        .flatten();
    let ready = stage
        .results
        .iter()
        .filter(|r| r.match_id != m.id)
        .filter(|r| {
            let theirs = [r.side_a, r.side_b].iter().any(|s| *s == a || *s == b);
            let round =
                r.match_id.side == BracketSide::Swiss && Some(r.match_id.round) == previous_round;
            theirs || round
        })
        .map(|r| r.confirmed_at);
    let previous = stage
        .games_of(m.id)
        .into_iter()
        .filter(|g| g.number == game - 1)
        .map(|g| g.reported_at);
    started_at
        .into_iter()
        .chain(ready)
        .chain(previous)
        .chain(veto::released_at(stage, m.id))
        .max()
}
