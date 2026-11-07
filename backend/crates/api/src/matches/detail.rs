//! Uma partida com lados, mapas e histórico: o que a página da partida
//! mostra e o que toda ação sobre ela devolve.

use engine::core::{Match, ParticipantId};
use sqlx::PgConnection;

use super::{
    contest, db,
    games::GameStatus,
    key::{self, MatchKey},
    progress,
    stage::Stage,
    tasks,
    types::MatchDetail,
    veto,
    view::{self, Viewer},
};
use crate::{
    shared::ApiError,
    teams::StoredTeam,
    tournaments::{StoredTournament, TeamAction},
};

const NOT_FOUND: ApiError = ApiError::not_found("match.not_found");

/// A partida `key` da fase. 404 `match.not_found` se o identificador não
/// existe na chave (ou é de outra fase).
pub fn find<'a>(stage: &'a Stage, key: &MatchKey) -> Result<&'a Match, ApiError> {
    if key.stage != stage.number {
        return Err(NOT_FOUND);
    }
    let id = key::parse(&key.local).ok_or(NOT_FOUND)?;
    stage.bracket.get(id).ok_or(NOT_FOUND)
}

pub async fn load(
    conn: &mut PgConnection,
    tournament: &StoredTournament,
    stage: &Stage,
    viewer: &Viewer,
    m: &Match,
) -> Result<MatchDetail, ApiError> {
    let playing = progress::playing(stage);
    let summary = view::summary(tournament.status, stage, m, viewer, &playing);
    let games = stage.games_of(m.id);
    let mine = view::viewer_side(stage, m, viewer);
    let local = key::format(m.id);
    let (events, lineups) = db::match_history(conn, tournament.id, stage.number, &local).await?;
    let team = |id: Option<i64>| {
        id.and_then(|id| stage.entrant(ParticipantId(id)))
            .map(StoredTeam::view)
    };
    Ok(MatchDetail {
        a: team(summary.a.team_id),
        b: team(summary.b.team_id),
        stage: u16::try_from(stage.number).unwrap_or(1),
        format: stage.format.into(),
        part_rounds: stage.bracket.total_rounds().unwrap_or_else(|| {
            stage
                .bracket
                .matches()
                .iter()
                .filter(|x| x.id.side == m.id.side)
                .map(|x| x.id.round)
                .max()
                .unwrap_or(1)
        }),
        status: tournament.status,
        games: games
            .iter()
            .map(|g| {
                // Quem contestou por último (a contestação que vale agora).
                let contested_by = (g.status == GameStatus::Contested)
                    .then(|| {
                        events
                            .iter()
                            .rev()
                            .find(|e| e.kind == "contested" && e.game == Some(g.number))
                            .and_then(|e| e.actor_name.clone())
                    })
                    .flatten();
                let extra = view::GameExtra {
                    task: tasks::game_task(tournament.status, stage, m, g, mine, viewer),
                    can_contest: contest::allowed(viewer.user_id, tournament.status, stage, m, g),
                    contested_by,
                };
                view::game(g, extra, &lineups)
            })
            .collect(),
        next_game: tasks::next_game(tournament.status, m, &games),
        viewer_side: mine,
        can_report: mine.is_some() && viewer.may(TeamAction::Report),
        history: events
            .into_iter()
            .map(|e| view::event(stage, e))
            .collect::<Result<_, _>>()?,
        can_manage: viewer.organizer,
        veto: veto::view(stage, m, mine, Some(viewer.user_id)),
        summary,
        now: chrono::Utc::now(),
    })
}
