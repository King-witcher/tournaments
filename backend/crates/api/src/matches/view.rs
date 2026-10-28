//! Fase refeita → resposta da API, do ponto de vista de quem vê (as partidas
//! dele e o que cada uma espera dele).

use engine::core::{self as ec, Match, Occupant, ParticipantId, Slot};

use super::{
    db::{EventRow, GameRow},
    key,
    result::{ResultStatus, StoredResult},
    stage::Stage,
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
    Ok(TournamentBracket {
        format: stage.format.into(),
        status: tournament.status,
        entrants: stage.entrants.iter().map(entrant).collect(),
        matches: stage
            .bracket
            .matches()
            .iter()
            .map(|m| summary(tournament.status, stage, m, viewer))
            .collect(),
        standings,
        contest_minutes: tournament.contest_minutes,
        results: stage.results.len() as i64,
    })
}

pub fn summary(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    viewer: &Viewer,
) -> BracketMatch {
    let (oa, ob) = match m.state {
        // O reset que não precisou acontecer não tem lados.
        ec::MatchState::NotNeeded => (Occupant::Unknown, Occupant::Unknown),
        _ => stage
            .bracket
            .occupants(m.id)
            .unwrap_or((Occupant::Unknown, Occupant::Unknown)),
    };
    let player = |o: Occupant| match o {
        Occupant::Participant(p) => Some(p),
        _ => None,
    };
    let yours = viewer
        .participant
        .is_some_and(|me| [player(oa), player(ob)].contains(&Some(me)));
    let result = stage.result(m.id);
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
        result: result.map(match_result),
        yours,
        task: task(status, m, result, yours, viewer),
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
        status: r.status,
        walkover: r.score.is_none(),
        winner: r.winner_side(),
        a_wins: r.score.map(|(a, _)| a),
        b_wins: r.score.map(|(_, b)| b),
        reported_side: r.reported_side,
        reported_at: r.reported_at,
        confirm_by: (r.status == ResultStatus::Pending)
            .then_some(r.confirm_by)
            .flatten(),
        contest_comment: r.contest_comment.clone(),
        contested_at: r.contested_at,
        confirmed_via: r.confirmed_via,
        confirmed_at: r.confirmed_at,
    }
}

/// O que a partida espera de quem vê. Só com o torneio em andamento e a
/// partida pronta.
fn task(
    status: TournamentStatus,
    m: &Match,
    result: Option<&StoredResult>,
    yours: bool,
    viewer: &Viewer,
) -> Option<ViewerTask> {
    if status != TournamentStatus::InProgress || !matches!(m.state, ec::MatchState::Ready { .. }) {
        return None;
    }
    let Some(r) = result else {
        return yours.then_some(ViewerTask::Report);
    };
    match r.status {
        ResultStatus::Contested if viewer.organizer => Some(ViewerTask::Resolve),
        ResultStatus::Contested => yours.then_some(ViewerTask::AwaitOrganizer),
        ResultStatus::Pending if yours => {
            let mine = viewer.participant.and_then(|p| r.side_of(p));
            Some(if mine == r.reported_side {
                ViewerTask::AwaitOpponent
            } else {
                ViewerTask::Respond
            })
        }
        ResultStatus::Pending | ResultStatus::Confirmed => None,
    }
}

pub fn game(g: GameRow) -> GameDetail {
    GameDetail {
        number: g.number,
        map: MapRef {
            slug: g.map_slug,
            name: g.map_name,
        },
        a_frags: g.a_frags,
        b_frags: g.b_frags,
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
                walkover: e.winner_maps.is_none(),
                winner_maps: maps(e.winner_maps).map_err(|_| ApiError::internal())?,
                loser_maps: maps(e.loser_maps).map_err(|_| ApiError::internal())?,
            })
        }
        None => None,
    };
    Ok(MatchEvent {
        kind: EventKind::from_db(&e.kind)?,
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
