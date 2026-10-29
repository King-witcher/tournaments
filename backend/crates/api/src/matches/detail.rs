//! Uma partida com lados, mapas e histórico: o que a página da partida
//! mostra e o que toda ação sobre ela devolve.

use engine::core::Match;
use sqlx::PgConnection;

use super::{
    db, key,
    stage::Stage,
    tasks,
    types::MatchDetail,
    veto,
    view::{self, Viewer},
};
use crate::{shared::ApiError, tournaments::StoredTournament};

const NOT_FOUND: ApiError = ApiError::not_found("match.not_found");

/// A partida `key` da fase. 404 `match.not_found` se o identificador não
/// existe na chave.
pub fn find<'a>(stage: &'a Stage, key: &str) -> Result<&'a Match, ApiError> {
    let id = key::parse(key).ok_or(NOT_FOUND)?;
    stage.bracket.get(id).ok_or(NOT_FOUND)
}

pub async fn load(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    viewer: &Viewer,
    m: &Match,
) -> Result<MatchDetail, ApiError> {
    let summary = view::summary(tournament.status, stage, m, viewer);
    let games = stage.games_of(m.id);
    let mine = view::viewer_side(stage, m, viewer);
    let events = db::events(conn, tournament.id, stage.number, &summary.key).await?;
    let entrant = |user: Option<i64>| {
        user.and_then(|u| stage.entrants.iter().find(|e| e.user_id == u))
            .map(view::entrant)
    };
    Ok(MatchDetail {
        a: entrant(summary.a.user_id),
        b: entrant(summary.b.user_id),
        format: stage.format.into(),
        part_rounds: stage
            .bracket
            .matches()
            .iter()
            .filter(|x| x.id.side == m.id.side)
            .map(|x| x.id.round)
            .max()
            .unwrap_or(1),
        status: tournament.status,
        games: games
            .iter()
            .map(|g| {
                let task = tasks::game_task(tournament.status, m, g, mine, viewer);
                view::game(g, task)
            })
            .collect(),
        next_game: tasks::next_game(tournament.status, m, &games),
        viewer_side: mine,
        history: events
            .into_iter()
            .map(|e| view::event(stage, e))
            .collect::<Result<_, _>>()?,
        can_manage: viewer.organizer,
        veto: veto::view(stage, m, mine),
        summary,
    })
}
