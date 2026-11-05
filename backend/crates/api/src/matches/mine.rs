//! Onde quem vê está na chave de um torneio e qual é a próxima partida dele,
//! para a lista dos torneios dele em todas as comunidades (`/me/tournaments`)
//! e para a entrada de um torneio só; onde ele terminou nos encerrados, de
//! vários de uma vez (`final_places`); e onde outro inscrito está, sem a
//! próxima partida, para o perfil do membro na comunidade (`place_of`).

use chrono::Utc;
use engine::{
    core::{self as ec, Occupant, ParticipantId, Slot},
    formats::{self as ef, StageBracket, elimination::EliminationKind},
};
use sqlx::PgConnection;

use super::{
    due, key, progress,
    result::Side,
    stage::{self, Stage},
    tasks,
    types::{MatchState, NextMatch, SideSource, SourceOutcome, ViewerTask},
    veto,
    view::{self, Viewer},
};
use crate::{
    shared::ApiError,
    teams::StoredTeam,
    tenants::TenantMember,
    tournaments::{StoredTournament, TournamentId, TournamentStatus, find_visible},
};

/// Onde o inscrito está na chave. A colocação é a faixa (`rank` 5 com
/// `tied` 4 é 5º–8º).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// Inscrito sem seed: o torneio começou sem ele (não fez check-in).
    Missed,
    /// Na chave e ainda vivo (no round robin e no suíço, até a fase terminar:
    /// a colocação sai da tabela final).
    Playing,
    /// Desistiu; com a colocação quando ela já se definiu.
    Withdrawn(Option<(u16, u16)>),
    /// Eliminado, ou fora do 1º lugar na tabela final, com a colocação.
    Eliminated(u16, u16),
    Champion,
}

pub struct BracketPlace {
    /// O torneio depois das confirmações por tempo (pode ter encerrado).
    pub tournament: StoredTournament,
    pub place: Place,
    /// Só de quem ainda joga, e só em `place` (quem vê).
    pub next: Option<NextMatch>,
}

/// O lugar de `member` (inscrito) na chave do torneio `slug`, em andamento ou
/// encerrado, depois das confirmações por tempo vencidas.
pub async fn place<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
) -> Result<BracketPlace, ApiError> {
    let tournament = find_visible(conn, member, slug).await?;
    place_from(conn, member, tournament).await
}

/// Como `place`, com o torneio já lido por `find_visible`.
pub async fn place_from<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    tournament: StoredTournament,
) -> Result<BracketPlace, ApiError> {
    let (tournament, stage) = due::current_from(conn, member, tournament).await?;
    let viewer = Viewer::new(member, &stage);
    let place = locate(&stage.bracket, &stage.entrants, viewer.participant);
    let next = match place {
        Place::Playing => next_match(tournament.status, &stage, &viewer),
        _ => None,
    };
    Ok(BracketPlace {
        tournament,
        place,
        next,
    })
}

/// Onde `user_id` (inscrito) terminou em cada torneio encerrado de
/// `finished`, na ordem: só a chave, sem o veto, e duas idas ao banco para
/// todos (`stage::brackets`). `None` no torneio com confirmação por tempo
/// vencida: a chave ainda pode mudar, e ele vai por `place` (que aplica e
/// trava).
pub async fn final_places(
    conn: &mut PgConnection,
    finished: &[TournamentId],
    user_id: i64,
) -> Result<Vec<Option<Place>>, ApiError> {
    let now = Utc::now();
    Ok(stage::brackets(conn, finished)
        .await?
        .iter()
        .map(|s| {
            let me = stage::participant_of(&s.entrants, user_id);
            (!due::confirmations_due(&s.games, now)).then(|| locate(&s.bracket, &s.entrants, me))
        })
        .collect())
}

/// Como `place`, para outro inscrito, `user_id`, no torneio `slug` como
/// `member` o vê: sem a próxima partida, que é só de quem joga.
pub async fn place_of<R>(
    conn: &mut PgConnection,
    member: &TenantMember<R>,
    slug: &str,
    user_id: i64,
) -> Result<BracketPlace, ApiError> {
    let (tournament, stage) = due::current(conn, member, slug).await?;
    let place = locate(
        &stage.bracket,
        &stage.entrants,
        stage.participant_of(user_id),
    );
    Ok(BracketPlace {
        tournament,
        place,
        next: None,
    })
}

/// O lugar de `participant` na chave `bracket` dos times `entrants`; `None`
/// é o inscrito sem seed.
fn locate(
    bracket: &StageBracket,
    entrants: &[StoredTeam],
    participant: Option<ParticipantId>,
) -> Place {
    let Some(me) = participant else {
        return Place::Missed;
    };
    let standing = bracket
        .placements()
        .into_iter()
        .find(|p| p.participant == me)
        .map(|p| (p.rank, p.tied));
    let withdrawn = entrants.iter().any(|e| e.id == me.0 && e.withdrawn);
    match standing {
        Some((1, _)) => Place::Champion,
        _ if withdrawn => Place::Withdrawn(standing),
        Some((rank, tied)) => Place::Eliminated(rank, tied),
        None => Place::Playing,
    }
}

/// Round robin: a série em que o adversário enviou um mapa, que tem prazo, ou que
/// tem uma contestação para ele aceitar, ou a sugerida (`progress`). `None` nos
/// outros formatos.
fn league_next<'a>(
    status: TournamentStatus,
    stage: &'a Stage,
    viewer: &Viewer,
) -> Option<&'a ec::Match> {
    let suggested = progress::suggestions(stage);
    let me = suggested.get(&viewer.participant?)?;
    let answer = stage.bracket.matches().iter().find(|m| {
        let side = view::viewer_side(stage, m, viewer);
        side.is_some()
            && matches!(
                tasks::match_task(status, stage, m, side, viewer),
                Some(ViewerTask::Respond | ViewerTask::AcceptContest)
            )
    });
    answer.or_else(|| stage.bracket.get(*me))
}

/// A partida pronta de quem vê ou, sem nenhuma, a primeira em que ele já
/// está e que espera o adversário sair de outra.
fn next_match(status: TournamentStatus, stage: &Stage, viewer: &Viewer) -> Option<NextMatch> {
    let mine = |ready: bool| {
        stage.bracket.matches().iter().find(|m| {
            let wanted = match m.state {
                ec::MatchState::Ready { .. } => ready,
                ec::MatchState::Pending => !ready,
                _ => false,
            };
            wanted && view::viewer_side(stage, m, viewer).is_some()
        })
    };
    let m = league_next(status, stage, viewer)
        .or_else(|| mine(true))
        .or_else(|| mine(false))?;
    let side = view::viewer_side(stage, m, viewer)?;
    let (oa, ob) = view::occupants(stage, m);
    let (opponent, slot) = match side {
        Side::A => (ob, m.b),
        Side::B => (oa, m.a),
    };
    Some(NextMatch {
        key: key::format(m.id),
        state: match m.state {
            ec::MatchState::Ready { .. } => MatchState::Ready,
            _ => MatchState::Pending,
        },
        single: matches!(
            stage.format,
            ef::StageFormat::Elimination(c) if c.kind == EliminationKind::Single
        ),
        opponent: match opponent {
            Occupant::Participant(p) => stage.entrant(p).map(StoredTeam::view),
            _ => None,
        },
        opponent_from: match (opponent, slot) {
            (Occupant::Unknown, Slot::WinnerOf(id)) => Some(SideSource {
                outcome: SourceOutcome::Winner,
                match_key: key::format(id),
            }),
            (Occupant::Unknown, Slot::LoserOf(id)) => Some(SideSource {
                outcome: SourceOutcome::Loser,
                match_key: key::format(id),
            }),
            _ => None,
        },
        task: tasks::match_task(status, stage, m, Some(side), viewer),
        veto: veto::status(stage, m),
        next_map: veto::next_map(status, stage, m, &stage.games_of(m.id)),
        opponent_playing: progress::opponent_playing(
            stage,
            m,
            Some(side),
            &progress::playing(stage),
        ),
    })
}
