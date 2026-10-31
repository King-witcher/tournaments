//! Depois de toda mudança na chave (`settle`): a série que passou a ter
//! outros lados (ou nenhum) perde o veto, e a partida seguinte de cada série
//! pronta começa quando a série aceita mais uma (a anterior foi enviada),
//! com o mapa em que a anterior foi enviada. O draft de campeões nunca fica
//! pela metade: se o organizador lançou a partida 1 antes do fim dele, o que
//! falta sai pelo sorteio do tempo esgotado (`due::draw_turn`) antes de a
//! partida 2 começar.

use engine::{core::MatchState, series, veto::Event};
use sqlx::PgConnection;

use super::{
    Ctx,
    db::{self, NewVetoEvent},
    due,
    state::Kind,
};
use crate::{
    matches::{
        db::MatchRef,
        games::{StoredGame, series_games},
        key,
        stage::Stage,
    },
    shared::ApiError,
    tournaments::StoredTournament,
};

/// Aplica e devolve se gravou alguma coisa (quem chama refaz a fase).
pub async fn advance(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
) -> Result<bool, ApiError> {
    let mut changed = false;
    for mv in &stage.vetoes {
        let Some(m) = stage.bracket.get(mv.match_id) else {
            continue;
        };
        let key = key::format(m.id);
        let at = MatchRef {
            tenant: tournament.tenant,
            tournament: tournament.id,
            stage: stage.number,
            key: &key,
        };
        let sides = match m.state {
            MatchState::Ready { a, b } | MatchState::Completed { a, b, .. } => Some((a, b)),
            _ => None,
        };
        if sides != Some(mv.sides) {
            db::remove(conn, &at).await?;
            changed = true;
            continue;
        }
        let MatchState::Ready { a, b } = m.state else {
            continue;
        };
        if !mv.state.begun() {
            continue;
        }
        let games = stage.games_of(m.id);
        if series::check_next(m.best_of, &series_games(&games), None).is_err() {
            continue;
        }
        let next = games.len() + 1;
        let mut state = mv.state.clone();
        let mut current = usize::from(state.game().unwrap_or(1));
        if current < next && current == 1 && stage.veto_rules.champion_phase {
            let ctx = Ctx::of(stage, mv);
            while due::draw_turn(conn, &at, (a, b), &mut state, &ctx, None).await? > 0 {
                changed = true;
            }
        }
        while current < next {
            let previous: &StoredGame = games[current - 1];
            let game = u8::try_from(current + 1).map_err(|_| ApiError::internal())?;
            let played_map = previous.map.as_ref().map(|m| m.id);
            state
                .apply(
                    Event::Game {
                        game,
                        previous: played_map,
                    },
                    None,
                )
                .map_err(|e| {
                    tracing::error!(%e, key, "partida seguinte do veto recusada");
                    ApiError::internal()
                })?;
            let event = NewVetoEvent {
                played_map,
                ..NewVetoEvent::new(Kind::Game, Some(i16::from(game)), None, None)
            };
            db::insert(conn, &at, (a, b), &event).await?;
            changed = true;
            current += 1;
        }
    }
    Ok(changed)
}
