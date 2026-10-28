//! Fase refeita → resposta da API, do ponto de vista de quem vê (as partidas
//! dele e o que cada uma espera dele, em `tasks`).

use engine::core::{self as ec, Match, Occupant, ParticipantId, Slot};

use super::{
    db::EventRow,
    games::{GameStatus, StoredGame, confirmed_wins},
    key,
    result::{Side, StoredResult},
    stage::Stage,
    tasks,
    types::{
        BracketEntrant, BracketMatch, BracketPart, EventActor, EventKind, EventOutcome, GameDetail,
        MapRef, MatchEvent, MatchResult, MatchSide, MatchState, SideSource, SourceOutcome,
        Standing, TournamentBracket, ViewerTask,
    },
};
use crate::{
    participants::Seeded,
    shared::ApiError,
    tenants::{Role, TenantMember},
    tournaments::{StoredTournament, TournamentStatus},
};

/// Quem vê: o participante dele na chave, se joga, e se organiza.
pub struct Viewer {
    pub participant: Option<ParticipantId>,
    pub organizer: bool,
}

impl Viewer {
    pub fn new<R>(member: &TenantMember<R>, stage: &Stage) -> Self {
        Self {
            participant: stage.participant_of(member.user.id),
            organizer: member.role >= Role::Organizer,
        }
    }
}

pub fn entrant(e: &Seeded) -> BracketEntrant {
    BracketEntrant {
        user_id: e.user_id,
        nickname: e.nickname.clone(),
        avatar_url: e.avatar_url.clone(),
        seed: e.seed,
        withdrawn: e.withdrawn,
        left: e.left,
    }
}

pub fn bracket(
    tournament: &StoredTournament,
    stage: &Stage,
    viewer: &Viewer,
) -> Result<TournamentBracket, ApiError> {
    let user = |p: ParticipantId| stage.entrant(p).map(|e| e.user_id);
    let standings = stage
        .bracket
        .standings()
        .into_iter()
        .map(|p| {
            Ok(Standing {
                user_id: user(p.participant).ok_or_else(ApiError::internal)?,
                rank: p.rank,
                tied: p.tied,
            })
        })
        .collect::<Result<_, ApiError>>()?;
    let matches: Vec<BracketMatch> = stage
        .bracket
        .matches()
        .iter()
        .map(|m| summary(tournament.status, stage, m, viewer))
        .collect();
    let touched = stage
        .bracket
        .matches()
        .iter()
        .filter(|m| stage.result(m.id).is_some() || !stage.games_of(m.id).is_empty())
        .count();
    Ok(TournamentBracket {
        format: stage.format.into(),
        status: tournament.status,
        entrants: stage.entrants.iter().map(entrant).collect(),
        matches,
        standings,
        contest_minutes: tournament.contest_minutes,
        results: touched as i64,
    })
}

/// Os dois lados que a partida tem agora, como o `engine` os vê.
fn occupants(stage: &Stage, m: &Match) -> (Occupant, Occupant) {
    match m.state {
        // O reset que não precisou acontecer não tem lados.
        ec::MatchState::NotNeeded => (Occupant::Unknown, Occupant::Unknown),
        _ => stage
            .bracket
            .occupants(m.id)
            .unwrap_or((Occupant::Unknown, Occupant::Unknown)),
    }
}

/// O lado de quem vê na partida `m`, se joga.
pub fn viewer_side(stage: &Stage, m: &Match, viewer: &Viewer) -> Option<Side> {
    let me = viewer.participant?;
    match occupants(stage, m) {
        (Occupant::Participant(a), _) if a == me => Some(Side::A),
        (_, Occupant::Participant(b)) if b == me => Some(Side::B),
        _ => None,
    }
}

pub fn summary(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    viewer: &Viewer,
) -> BracketMatch {
    let (oa, ob) = occupants(stage, m);
    let mine = viewer_side(stage, m, viewer);
    let games = stage.games_of(m.id);
    let (a_maps, b_maps) = confirmed_wins(&games);
    let count = |s: GameStatus| games.iter().filter(|g| g.status == s).count() as u8;
    BracketMatch {
        key: key::format(m.id),
        part: match m.id.side {
            ec::BracketSide::Winners => BracketPart::Winners,
            ec::BracketSide::Losers => BracketPart::Losers,
            ec::BracketSide::GrandFinal => BracketPart::GrandFinal,
        },
        round: m.id.round,
        index: m.id.index,
        best_of: m.best_of,
        state: match m.state {
            ec::MatchState::Pending => MatchState::Pending,
            ec::MatchState::Ready { .. } => MatchState::Ready,
            ec::MatchState::Bye { .. } => MatchState::Bye,
            ec::MatchState::Completed { .. } => MatchState::Completed,
            ec::MatchState::NotNeeded => MatchState::NotNeeded,
        },
        a: side(stage, m.a, oa),
        b: side(stage, m.b, ob),
        result: stage.result(m.id).map(match_result),
        a_maps,
        b_maps,
        pending_maps: count(GameStatus::Pending),
        contested_maps: count(GameStatus::Contested),
        yours: mine.is_some(),
        task: tasks::match_task(status, m, &games, mine, viewer),
    }
}

fn side(stage: &Stage, slot: Slot, occupant: Occupant) -> MatchSide {
    let from = |outcome, id| {
        Some(SideSource {
            outcome,
            match_key: key::format(id),
        })
    };
    MatchSide {
        user_id: match occupant {
            Occupant::Participant(p) => stage.entrant(p).map(|e| e.user_id),
            _ => None,
        },
        bye: occupant == Occupant::Bye,
        from: match slot {
            Slot::WinnerOf(id) => from(SourceOutcome::Winner, id),
            Slot::LoserOf(id) => from(SourceOutcome::Loser, id),
            Slot::Participant(_) | Slot::Bye => None,
        },
    }
}

fn match_result(r: &StoredResult) -> MatchResult {
    MatchResult {
        walkover: r.score.is_none(),
        winner: r.winner_side(),
        a_wins: r.score.map(|(a, _)| a),
        b_wins: r.score.map(|(_, b)| b),
        confirmed_via: r.confirmed_via,
        confirmed_at: r.confirmed_at,
    }
}

pub fn game(g: &StoredGame, task: Option<ViewerTask>) -> GameDetail {
    GameDetail {
        number: g.number,
        winner: g.winner_side(),
        map: g.map.as_ref().map(|m| MapRef {
            slug: m.slug.clone(),
            name: m.name.clone(),
        }),
        a_frags: g.frags.map(|(a, _)| a),
        b_frags: g.frags.map(|(_, b)| b),
        status: g.status,
        reported_side: g.reported_side,
        reported_at: g.reported_at,
        confirm_by: (g.status == GameStatus::Pending)
            .then_some(g.confirm_by)
            .flatten(),
        contest_comment: g.contest_comment.clone(),
        contested_at: g.contested_at,
        confirmed_via: g.confirmed_via,
        confirmed_at: g.confirmed_at,
        task,
    }
}

pub fn event(stage: &Stage, e: EventRow) -> Result<MatchEvent, ApiError> {
    let maps = |n: Option<i16>| n.map(u8::try_from).transpose();
    let outcome = match e.winner {
        Some(winner) => {
            let entrant = stage
                .entrant(ParticipantId(winner))
                .ok_or_else(ApiError::internal)?;
            Some(EventOutcome {
                winner: entrant.user_id,
                winner_nickname: entrant.nickname.clone(),
                walkover: e.game.is_none() && e.winner_maps.is_none(),
                winner_maps: maps(e.winner_maps).map_err(|_| ApiError::internal())?,
                loser_maps: maps(e.loser_maps).map_err(|_| ApiError::internal())?,
            })
        }
        None => None,
    };
    Ok(MatchEvent {
        kind: EventKind::from_db(&e.kind)?,
        game: e.game,
        actor: e.actor.map(|user_id| EventActor {
            user_id,
            nickname: e.actor_name.unwrap_or_default(),
            avatar_url: e.actor_avatar,
        }),
        at: e.created_at,
        outcome,
        comment: e.comment,
    })
}
