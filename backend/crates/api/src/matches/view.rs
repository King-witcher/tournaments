//! Fase refeita → resposta da API, do ponto de vista de quem vê (as partidas
//! dele e o que cada uma espera dele, em `tasks`).

use std::collections::BTreeSet;

use engine::{
    core::{self as ec, Match, Occupant, ParticipantId, Slot},
    formats::{self as ef, table::Criterion},
};

use super::{
    chain,
    db::{EventRow, LineupRow},
    games::{GameStatus, StoredGame, confirmed_wins},
    key, progress,
    result::{Side, StoredResult},
    stage::{Chain, Stage},
    tasks, ties,
    types::{
        BracketMatch, BracketPart, ContestProposal, EventActor, EventKind, EventOutcome,
        GameDetail, MapRef, MatchEvent, MatchResult, MatchSide, MatchState, PendingTieGroup,
        SideSource, SourceOutcome, Standing, TableCriterion, TableRow, TournamentBracket,
        ViewerTask,
    },
    veto,
};
use crate::{
    shared::ApiError,
    teams::{StoredTeam, TeamMember},
    tenants::{Role, TenantMember},
    tournaments::{StoredTournament, TeamAction, TeamRules, TournamentStatus},
};

/// Quem vê: o participante dele na chave, se joga, se organiza e o que a regra
/// do torneio deixa ele fazer pelo time.
pub struct Viewer {
    pub user_id: i64,
    pub participant: Option<ParticipantId>,
    pub organizer: bool,
    pub captain: bool,
    pub rules: TeamRules,
}

impl Viewer {
    pub fn new<R>(member: &TenantMember<R>, stage: &Stage) -> Self {
        let participant = stage.participant_of(member.user.id);
        Self {
            user_id: member.user.id,
            participant,
            organizer: member.role >= Role::Organizer,
            captain: stage.is_captain(member.user.id),
            rules: stage.team_rules,
        }
    }

    /// A regra deixa quem vê fazer `action` pelo time dele.
    pub const fn may(&self, action: TeamAction) -> bool {
        self.rules.allows(action, self.captain)
    }
}

/// As colocações `placements` como a API mostra, com os times de `teams`.
fn standings_of(
    placements: &[ec::Placement],
    teams: &[StoredTeam],
) -> Result<Vec<Standing>, ApiError> {
    placements
        .iter()
        .map(|p| {
            let team = teams.iter().find(|t| t.id == p.participant.0);
            Ok(Standing {
                team_id: team.map(|t| t.id).ok_or_else(ApiError::internal)?,
                rank: p.rank,
                tied: p.tied,
            })
        })
        .collect()
}

/// A chave da fase `stage`, com a classificação final do torneio
/// (`chain::standings` das fases começadas, `chain`).
pub fn bracket(
    tournament: &StoredTournament,
    stage: &Stage,
    viewer: &Viewer,
    chain: &Chain,
) -> Result<TournamentBracket, ApiError> {
    let team = |p: ParticipantId| stage.entrant(p).map(|e| e.id);
    let standings = standings_of(&stage.bracket.placements(), &stage.entrants)?;
    let composite = chain::standings(chain);
    let standings_final = standings_of(&composite, &chain.teams)?;
    let other_teams = chain
        .teams
        .iter()
        .filter(|t| stage.entrant(t.participant()).is_none())
        .filter(|t| composite.iter().any(|p| p.participant.0 == t.id))
        .map(StoredTeam::view)
        .collect();
    let playing = progress::playing(stage);
    let matches: Vec<BracketMatch> = stage
        .bracket
        .matches()
        .iter()
        .map(|m| summary(tournament.status, stage, m, viewer, &playing))
        .collect();
    let touched = stage
        .bracket
        .matches()
        .iter()
        .filter(|m| stage.result(m.id).is_some() || !stage.games_of(m.id).is_empty())
        .count();
    let table = stage
        .bracket
        .table()
        .map(|rows| table_rows(stage, rows))
        .transpose()?;
    let pending_ties = stage
        .bracket
        .pending_ties()
        .iter()
        .map(|t| {
            Ok(PendingTieGroup {
                position: t.position,
                team_ids: t
                    .participants
                    .iter()
                    .map(|p| team(*p).ok_or_else(ApiError::internal))
                    .collect::<Result<_, ApiError>>()?,
            })
        })
        .collect::<Result<_, ApiError>>()?;
    let format = match tournament.stage(stage.number) {
        Some((setup, _)) => (*setup).into(),
        None => stage.format.into(),
    };
    Ok(TournamentBracket {
        stage: u16::try_from(stage.number).unwrap_or(1),
        format,
        status: tournament.status,
        entrants: stage.entrants.iter().map(StoredTeam::view).collect(),
        matches,
        standings,
        standings_final,
        other_teams,
        table,
        pending_ties,
        total_rounds: stage.bracket.total_rounds(),
        contest_minutes: tournament.contest_minutes,
        results: touched as i64,
    })
}

/// A tabela, com os empates do fim: quem espera a organização divide a melhor
/// posição do grupo, e o grupo que ela ordenou leva o motivo.
fn table_rows(stage: &Stage, rows: &[ef::table::TableRow]) -> Result<Vec<TableRow>, ApiError> {
    let pending = stage.bracket.pending_ties();
    let mut out = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        let tie = pending
            .iter()
            .find(|t| t.participants.contains(&r.participant));
        let reason = decided_group(rows, i)
            .and_then(|group| ties::decision_for(stage, &group))
            .and_then(|d| d.reason.clone());
        out.push(table_row(stage, r, tie.map(|t| t.position), reason)?);
    }
    Ok(out)
}

/// O grupo que a organização ordenou em que está a linha `i`: as linhas
/// ligadas por `ahead_by` `organizer`.
fn decided_group(rows: &[ef::table::TableRow], i: usize) -> Option<Vec<ParticipantId>> {
    let linked = |k: usize| rows[k].ahead_by == Some(Criterion::Organizer);
    let mut start = i;
    while start > 0 && linked(start - 1) {
        start -= 1;
    }
    let mut end = i;
    while end < rows.len() && linked(end) {
        end += 1;
    }
    (end > start).then(|| rows[start..=end].iter().map(|r| r.participant).collect())
}

fn table_row(
    stage: &Stage,
    r: &ef::table::TableRow,
    pending_at: Option<u16>,
    tie_reason: Option<String>,
) -> Result<TableRow, ApiError> {
    Ok(TableRow {
        team_id: stage
            .entrant(r.participant)
            .map(|e| e.id)
            .ok_or_else(ApiError::internal)?,
        position: pending_at.unwrap_or(r.position),
        tie_pending: pending_at.is_some(),
        tie_reason,
        points: r.points,
        series_diff: r.series_diff,
        played: r.played,
        wins: r.wins,
        draws: r.draws,
        losses: r.losses,
        byes: r.byes,
        maps_won: r.maps_won,
        maps_lost: r.maps_lost,
        map_diff: r.map_diff,
        frags_for: r.frags_for,
        frags_against: r.frags_against,
        frag_diff: r.frag_diff,
        frags_complete: r.frags_complete,
        buchholz: r.buchholz,
        rating: r.rating.map(|c| f64::from(c) / 100.0),
        ahead_by: r.ahead_by.map(|c| match c {
            Criterion::SeriesDiff => TableCriterion::SeriesDiff,
            Criterion::Points => TableCriterion::Points,
            Criterion::Buchholz => TableCriterion::Buchholz,
            Criterion::HeadToHead => TableCriterion::HeadToHead,
            Criterion::MapDiff => TableCriterion::MapDiff,
            Criterion::FragDiff => TableCriterion::FragDiff,
            Criterion::LowerRating => TableCriterion::LowerRating,
            Criterion::Seed => TableCriterion::Seed,
            Criterion::Organizer => TableCriterion::Organizer,
        }),
    })
}

/// Os dois lados que a partida tem agora, como o `engine` os vê.
pub fn occupants(stage: &Stage, m: &Match) -> (Occupant, Occupant) {
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

/// A partida `m` na chave; `playing`, os times com uma série em jogo
/// (`progress::playing`), que quem monta a chave inteira calcula uma vez só.
pub fn summary(
    status: TournamentStatus,
    stage: &Stage,
    m: &Match,
    viewer: &Viewer,
    playing: &BTreeSet<ParticipantId>,
) -> BracketMatch {
    let (oa, ob) = occupants(stage, m);
    let mine = viewer_side(stage, m, viewer);
    let games = stage.games_of(m.id);
    let (a_maps, b_maps) = confirmed_wins(&games);
    let count = |s: GameStatus| games.iter().filter(|g| g.status == s).count() as u8;
    let (a_frags, b_frags) = stage.series_frags(m.id).unzip();
    let task = tasks::match_task(status, stage, m, mine, viewer);
    BracketMatch {
        key: key::public(stage.number, m.id),
        part: match m.id.side {
            ec::BracketSide::Winners => BracketPart::Winners,
            ec::BracketSide::Losers => BracketPart::Losers,
            ec::BracketSide::GrandFinal => BracketPart::GrandFinal,
            ec::BracketSide::RoundRobin => BracketPart::RoundRobin,
            ec::BracketSide::Swiss => BracketPart::Swiss,
            ec::BracketSide::Placement => BracketPart::Placement,
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
        a_frags,
        b_frags,
        pending_maps: count(GameStatus::Pending),
        contested_maps: count(GameStatus::Contested),
        yours: mine.is_some(),
        task,
        veto: veto::status(stage, m),
        next_map: veto::next_map(status, stage, m, &games),
        in_progress: progress::in_progress(stage, m),
        opponent_playing: progress::opponent_playing(stage, m, mine, playing),
        task_detail: tasks::task_detail(status, stage, m, task, mine, viewer),
    }
}

fn side(stage: &Stage, slot: Slot, occupant: Occupant) -> MatchSide {
    let from = |outcome, id| {
        Some(SideSource {
            outcome,
            match_key: key::public(stage.number, id),
        })
    };
    MatchSide {
        team_id: match occupant {
            Occupant::Participant(p) => stage.entrant(p).map(|e| e.id),
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

/// O que a página da série mostra de uma partida além da linha gravada: o
/// que ela espera de quem vê, se ele pode contestar e quem contestou.
pub struct GameExtra {
    pub task: Option<ViewerTask>,
    pub can_contest: bool,
    pub contested_by: Option<String>,
}

/// A partida `g` como a página da série mostra, com a escalação de cada lado
/// (`lineup`: as linhas da partida).
pub fn game(g: &StoredGame, extra: GameExtra, lineup: &[LineupRow]) -> GameDetail {
    let side = |s: &str| -> Vec<TeamMember> {
        lineup
            .iter()
            .filter(|r| r.game_id == g.id && r.side == s)
            .map(|r| TeamMember {
                user_id: r.user_id,
                nickname: r.nickname.clone(),
                avatar_url: r.avatar_url.clone(),
                left: r.left,
            })
            .collect()
    };
    GameDetail {
        lineup_a: side("a"),
        lineup_b: side("b"),
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
        contested_by: extra.contested_by,
        contest_proposal: g.contest_proposal.as_ref().map(|p| ContestProposal {
            map: MapRef {
                slug: p.map.slug.clone(),
                name: p.map.name.clone(),
            },
            a_frags: p.frags.0,
            b_frags: p.frags.1,
        }),
        can_contest: extra.can_contest,
        confirmed_via: g.confirmed_via,
        confirmed_at: g.confirmed_at,
        task: extra.task,
    }
}

pub fn event(stage: &Stage, e: EventRow) -> Result<MatchEvent, ApiError> {
    let maps = |n: Option<i16>| n.map(u8::try_from).transpose();
    let winner = e
        .winner
        .map(|w| {
            stage
                .entrant(ParticipantId(w))
                .map(StoredTeam::view)
                .ok_or_else(ApiError::internal)
        })
        .transpose()?;
    // Sem vencedor, só a série empatada tem o que mostrar: o placar.
    let outcome = (winner.is_some() || e.winner_maps.is_some())
        .then(|| {
            Ok::<_, ApiError>(EventOutcome {
                walkover: winner.is_some() && e.game.is_none() && e.winner_maps.is_none(),
                winner,
                winner_maps: maps(e.winner_maps).map_err(|_| ApiError::internal())?,
                loser_maps: maps(e.loser_maps).map_err(|_| ApiError::internal())?,
            })
        })
        .transpose()?;
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
