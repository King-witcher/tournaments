//! Depois de toda mudança na chave (`settle`): a partida que passou a ter
//! outros lados (ou nenhum) perde o veto, e o jogo seguinte de cada partida
//! pronta começa quando a série aceita mais um mapa (o anterior foi enviado),
//! com quem começa os campeões gravado no evento (quem perdeu o anterior;
//! no modo veto, quem não escolheu o mapa) e o mapa em que o anterior foi
//! enviado.

use engine::{core::MatchState, series, veto::Event};
use sqlx::PgConnection;

use super::{
    db::{self, NewVetoEvent},
    state::{Kind, api_side, ev_side, other},
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
        while current < next {
            let previous: &StoredGame = games[current - 1];
            let loser = other(previous.winner_side());
            let starter = state.next_starter(ev_side(loser));
            let game = u8::try_from(current + 1).map_err(|_| ApiError::internal())?;
            let played_map = previous.map.as_ref().map(|m| m.id);
            state
                .apply(
                    Event::Game {
                        game,
                        starter,
                        previous: played_map,
                    },
                    None,
                )
                .map_err(|e| {
                    tracing::error!(%e, key, "jogo seguinte do veto recusado");
                    ApiError::internal()
                })?;
            let event = NewVetoEvent {
                played_map,
                ..NewVetoEvent::new(
                    Kind::Game,
                    Some(i16::from(game)),
                    Some(api_side(starter)),
                    None,
                )
            };
            db::insert(conn, &at, (a, b), &event).await?;
            changed = true;
            current += 1;
        }
    }
    Ok(changed)
}
